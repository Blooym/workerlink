mod api;
mod authentication;
mod interface;
mod storage;

use api::{requests::CreateShortlinkRequestBody, responses::CreateShortlinkResponse};
use authentication::is_request_authorized;
use interface::bundle::INDEX_HTML;
use storage::{
    cloudflare_storage_driver::{CloudflareKVDriver, CLOUDFLARE_KV_BINDING},
    ShortlinkModel, StorageDriver,
};
use validator::Validate;
use worker::{event, Context, Date, Env, Request, Response, RouteContext, Router, Url};

// API Mapping:
//  [GET] /:id - Handle redirect to link (302/404/500)
//    RESPONSE -> 302 REDIRECT to url

//  [GET] /:id/exists - Check to see if an a link exists or not. (201, 404, 500)
//    RESPONSE -> 200 with empty body or error code.

//  AUTHED: [GET] /:id/details - Get the underlying struct as JSON for the link (200, 401, 404, 500)
//    RESPONSE ->
// {
//   url: Url,
//   views: u64,
//   max_views: Option<u64>,
//   expiry_timestamp: Option<u64>,
//   created_at_timestamp: u64,
//   last_visited_timestamp: Option<u64>,
// }
//  AUTHED: [PUT] /:id - Create or modify an existing link (201, 401, 500)
//    RESPONSE ->
// {
//   url: Url,
//   views: u64,
//   max_views: Option<u64>,
//   expiry_timestamp: Option<u64>,
//   created_at_timestamp: u64,
//   last_visited_timestamp: Option<u64>,
// }
//  AUTHED: [DELETE] /:id - Delete an existing link if it exists. (200, 401, 500)
//    RESPONSE -> 200 OK for deletion

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> worker::Result<Response> {
    Router::new()
        .get("/", |_, _| Response::from_html(INDEX_HTML))
        .get_async("/:id", do_shortlink_redirect)
        .get_async("/:id/details", get_shortlink_details)
        .head_async("/:id", shortlink_exists)
        .post_async("/:id", create_or_update_shortlink)
        .delete_async("/:id", delete_shortlink)
        .run(req, env)
        .await
}

/// Get the Shortlink key from a request.
fn get_shortlink_key_from_request(req: &Request) -> worker::Result<String> {
    let path = req.path();
    let Some(id) = path.split('/').nth(1) else {
        Err("Unable to find shortlink id for request.")?
    };
    Ok(id.to_string())
}

/// Checks if a Shortlink exists.
async fn shortlink_exists(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let storage = CloudflareKVDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);
    let key = get_shortlink_key_from_request(&req)?;

    if !storage.exists(&key).await {
        return Ok(Response::empty()?.with_status(404));
    }

    Ok(Response::empty()?.with_status(302))
}

/// Gets a Shortlink and handles the redirect for it's linked URL.
async fn do_shortlink_redirect(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let storage = CloudflareKVDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);
    let key = get_shortlink_key_from_request(&req)?;

    // Get the shortlink and apply validation.
    let shortlink = match storage.get_from_json::<ShortlinkModel>(&key).await {
        Some(mut value) => {
            if value.disabled {
                return Response::error("Shortlink does not exist or has been removed.", 404);
            }

            // If the shortlink has expired due to time.
            if let Some(expires_at_ms) = value.expiry_timestamp {
                if Date::now().as_millis() > expires_at_ms {
                    storage.delete(&key).await;
                    return Response::error("Shortlink does not exist or has been removed.", 404);
                }
            }

            // If the shortlink has reached it's maximum number of views.
            if let Some(max_views) = value.max_views {
                if value.views >= max_views {
                    storage.delete(&key).await;
                    return Response::error("Shortlink does not exist or has been removed.", 404);
                }
            }

            value.views += 1;
            value.last_visited_timestamp = Some(Date::now().as_millis());
            value
        }
        None => return Response::error("Shortlink does not exist or has been removed.", 404),
    };

    storage.set_as_json(&key, &shortlink).await;
    Response::redirect(shortlink.url)
}

/// Gets a Shortlink and returns its detailed.
async fn get_shortlink_details(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    if !is_request_authorized(&req, &ctx)? {
        return Response::error("Unauthorized", 401);
    }

    let storage = CloudflareKVDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);
    let key = get_shortlink_key_from_request(&req)?;

    // Get the shortlink and apply validation.
    let shortlink = match storage.get_from_json::<ShortlinkModel>(&key).await {
        Some(value) => {
            if value.disabled {
                return Response::error("Shortlink does not exist or has been removed.", 404);
            }

            // If the shortlink has expired due to time.
            if let Some(expires_at_ms) = value.expiry_timestamp {
                if Date::now().as_millis() > expires_at_ms {
                    storage.delete(&key).await;
                    return Response::error("Shortlink does not exist or has been removed.", 404);
                }
            }

            // If the shortlink has reached it's maximum number of views.
            if let Some(max_views) = value.max_views {
                if value.views >= max_views {
                    storage.delete(&key).await;
                    return Response::error("Shortlink does not exist or has been removed.", 404);
                }
            }

            value
        }
        None => return Response::error("Shortlink does not exist or has been removed.", 404),
    };
    Response::from_json(&shortlink)
}

/// Creates or updates a Shortlink.
async fn create_or_update_shortlink(
    mut req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    if !is_request_authorized(&req, &ctx)? {
        return Response::error("Unauthorized", 401);
    }

    let storage = CloudflareKVDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);
    let key = get_shortlink_key_from_request(&req)?;
    let body = req.json::<CreateShortlinkRequestBody>().await?;

    if body.validate().is_err() {
        return Response::error("Invalid payload", 400);
    }

    let url: Url = match Url::parse(&body.url) {
        Ok(url) => url,
        Err(_) => {
            return Response::error(
                "Unable to parse given URL, please ensure that it is valid.",
                400,
            )
        }
    };

    // Prevent making a Shortlink that recurses forever on the same domain.
    if req.url()?.domain() == url.domain() {
        return Response::error(
            "Cannot make a Shortlink to the same domain where Shortlink is hosted.",
            400,
        );
    }

    //
    let already_exists = storage.exists(&key).await;
    if !body.overwrite && already_exists {
        return Response::error(
            "A link with the given ID already exists and overwriting was not enabled.",
            409,
        );
    }

    let model = ShortlinkModel {
        url,
        max_views: body.max_views,
        views: 0,
        last_visited_timestamp: None,
        created_at_timestamp: Date::now().as_millis(),
        disabled: body.disabled,
        expiry_timestamp: body
            .expire_in
            .map(|time| Date::now().as_millis() + time.as_millis() as u64),
    };

    if !storage.set_as_json::<&ShortlinkModel>(&key, &model).await {
        return Response::error(
            "Something went wrong while trying to create shortlink.",
            500,
        );
    }

    Response::from_json(&CreateShortlinkResponse::from_shortlink_model(
        &model,
        req.url()?,
        already_exists,
    ))
}

/// Deletes a Shortlink.
async fn delete_shortlink(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    if !is_request_authorized(&req, &ctx)? {
        return Response::error("Unauthorized.", 401);
    }

    let storage = CloudflareKVDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);

    let key = get_shortlink_key_from_request(&req)?;
    match storage.get(&key).await {
        Some(_) => (),
        None => return Response::error("A shortlink with that ID does not exist.", 404),
    };

    if !storage.delete(&key).await {
        return Response::error("Something went wrong whilst deleting that shortlink.", 500);
    }

    Response::ok("Successfully deleted shortlink.")
}
