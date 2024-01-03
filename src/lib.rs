mod api;
mod authentication;
mod interface;
mod models;
mod storage;

use api::{requests::CreateShortlinkRequestBody, responses::CreateShortlinkResponse};
use authentication::authorized_guard;
use interface::INDEX_HTML;
use models::shortlink::{ShortlinkCreateArgs, ShortlinkModel};
use storage::{
    cloudflare_kv_driver::{CloudflareKVDriver, CLOUDFLARE_KV_BINDING},
    StorageDriver,
};
use validator::Validate;
use worker::{event, Context, Date, Env, Request, Response, RouteContext, Router, Url};

const SHORTLINK_NOT_EXIST_MESSAGE: &str =
    "Not found: The requested Shortlink does not exist, it may have been removed by its owner or expired.\nIf you believe this is an error, contact the link owner.";

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> worker::Result<Response> {
    Router::new()
        .get("/", |_, _| Response::from_html(INDEX_HTML))
        .get_async("/:id", do_shortlink_redirect)
        .get_async("/:id/details", get_shortlink_details)
        .get_async("/:id/exists", shortlink_exists)
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

    // Get the Shortlink and apply validation.
    let shortlink = match storage.get_from_json::<ShortlinkModel>(&key).await {
        Some(mut value) => {
            if value.disabled {
                return Response::error(SHORTLINK_NOT_EXIST_MESSAGE, 404);
            }

            // If the Shortlink has expired due to time.
            if let Some(expires_at_ms) = value.expiry_timestamp {
                if Date::now().as_millis() > expires_at_ms {
                    storage.delete(&key).await;
                    return Response::error(SHORTLINK_NOT_EXIST_MESSAGE, 404);
                }
            }

            // If the Shortlink has reached it's maximum number of views.
            if let Some(max_views) = value.max_views {
                if value.views >= max_views {
                    storage.delete(&key).await;
                    return Response::error(SHORTLINK_NOT_EXIST_MESSAGE, 404);
                }
            }

            value.increment_visits();
            value
        }
        None => return Response::error(SHORTLINK_NOT_EXIST_MESSAGE, 404),
    };

    storage.set_as_json(&key, &shortlink).await;
    Response::redirect(shortlink.url)
}

/// Gets a Shortlink and returns its detailed.
async fn get_shortlink_details(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let auth_guard = authorized_guard(&req, &ctx);
    if auth_guard.is_err() {
        return auth_guard.unwrap_err();
    }

    let storage = CloudflareKVDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);
    let key = get_shortlink_key_from_request(&req)?;

    // Get the Shortlink and apply validation.
    let shortlink = match storage.get_from_json::<ShortlinkModel>(&key).await {
        Some(value) => {
            if value.disabled {
                return Response::error(SHORTLINK_NOT_EXIST_MESSAGE, 404);
            }

            // If the Shortlink has expired due to time.
            if let Some(expires_at_ms) = value.expiry_timestamp {
                if Date::now().as_millis() > expires_at_ms {
                    storage.delete(&key).await;
                    return Response::error(SHORTLINK_NOT_EXIST_MESSAGE, 404);
                }
            }

            // If the Shortlink has reached it's maximum number of views.
            if let Some(max_views) = value.max_views {
                if value.views >= max_views {
                    storage.delete(&key).await;
                    return Response::error(SHORTLINK_NOT_EXIST_MESSAGE, 404);
                }
            }

            value
        }
        None => return Response::error(SHORTLINK_NOT_EXIST_MESSAGE, 404),
    };
    Response::from_json(&shortlink)
}

/// Creates or updates a Shortlink.
async fn create_or_update_shortlink(
    mut req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    let auth_guard = authorized_guard(&req, &ctx);
    if auth_guard.is_err() {
        return auth_guard.unwrap_err();
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

    let existing_model = storage.get_from_json::<ShortlinkModel>(&key).await;
    if !body.overwrite && existing_model.is_some() {
        return Response::error(
            "A link with the given ID already exists and overwriting was not enabled.",
            409,
        );
    }

    let model = match existing_model {
        Some(model) => model.update(ShortlinkCreateArgs {
            url,
            max_views: body.max_views,
            disabled: body.disabled,
            expiry_timestamp: body
                .expire_in
                .map(|time| Date::now().as_millis() + time.as_millis() as u64),
        }),
        None => ShortlinkModel::new(ShortlinkCreateArgs {
            url,
            max_views: body.max_views,
            disabled: body.disabled,
            expiry_timestamp: body
                .expire_in
                .map(|time| Date::now().as_millis() + time.as_millis() as u64),
        }),
    };

    if !storage.set_as_json::<&ShortlinkModel>(&key, &model).await {
        return Response::error(
            "Something went wrong while trying to create shortlink.",
            500,
        );
    }

    Response::from_json(&CreateShortlinkResponse::from_model(&model, req.url()?))
}

/// Deletes a Shortlink.
async fn delete_shortlink(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let auth_guard = authorized_guard(&req, &ctx);
    if auth_guard.is_err() {
        return auth_guard.unwrap_err();
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
