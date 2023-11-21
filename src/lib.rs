mod authentication;
mod storage;

use authentication::is_request_authorized;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use storage::{
    cloudflare_storage_driver::{CloudflareKVDriver, CLOUDFLARE_KV_BINDING},
    ShortlinkModel, StorageDriver,
};
use validator::Validate;
use worker::{event, Context, Date, Env, Request, Response, Result, RouteContext, Router, Url};

/// The HTML for the index page.
const INDEX_HTML_CONTENT: &str = include_str!("../static/index.html");

/// The request for creating/updating a shortlink.
#[derive(Validate, Deserialize)]
struct CreateShortlinkRequestBody {
    #[validate(url)]
    url: String,
    #[serde(default)]
    overwrite: bool,
    #[serde(default)]
    #[serde(with = "humantime_serde")]
    expire_in: Option<Duration>,
    #[serde(default)]
    #[validate(range(min = 1))]
    max_views: Option<u64>,
}

/// The response for successfully creating a Shortlink.
#[derive(Serialize)]
struct CreateLinkResponse {
    url: String,
    original_url: String,
    overwritten: bool,
    expiry_timestamp: Option<u64>,
    max_views: Option<u64>,
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get("/", |_, _| Response::from_html(INDEX_HTML_CONTENT))
        .get_async("/:id", do_shortlink_redirect)
        .head_async("/:id", shortlink_exists)
        .post_async("/:id", create_or_update_shortlink)
        .delete_async("/:id", delete_shortlink)
        .run(req, env)
        .await
}

/// Get the Shortlink key from a request.
fn get_shortlink_key_from_request(req: &Request) -> Result<String> {
    let path = req.path();
    let Some(id) = path.split('/').nth(1) else {
        Err("Unable to find shortlink id for request.")?
    };
    Ok(id.to_string())
}

/// Checks if a Shortlink exists.
async fn shortlink_exists(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let storage = CloudflareKVDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);
    let key = get_shortlink_key_from_request(&req)?;

    if !storage.exists(&key).await {
        return Ok(Response::empty()?.with_status(404));
    }

    Ok(Response::empty()?.with_status(302))
}

/// Gets a Shortlink and handles the redirect for it's linked URL.
async fn do_shortlink_redirect(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let storage = CloudflareKVDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);
    let key = get_shortlink_key_from_request(&req)?;

    // Get the shortlink and apply validation.
    let shortlink = match storage.get_from_json::<ShortlinkModel>(&key).await {
        Some(mut value) => {
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
    Response::redirect(shortlink.original_url)
}

/// Creates or updates a Shortlink.
async fn create_or_update_shortlink(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if !is_request_authorized(&req, &ctx)? {
        return Response::error("Unauthorized", 401);
    }

    let storage = CloudflareKVDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);
    let key = get_shortlink_key_from_request(&req)?;
    let body = req.json::<CreateShortlinkRequestBody>().await?;

    if body.validate().is_err() {
        return Response::error("Invalid payload", 400);
    }

    let original_url: Url = match Url::parse(&body.url) {
        Ok(url) => url,
        Err(_) => {
            return Response::error(
                "Unable to parse given URL, please ensure that it is valid.",
                400,
            )
        }
    };

    // Prevent making a Shortlink that recurses forever on the same domain.
    if req.url()?.domain() == original_url.domain() {
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
        original_url,
        max_views: body.max_views,
        views: 0,
        last_visited_timestamp: None,
        created_at_timestamp: Date::now().as_millis(),
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

    Response::from_json(&CreateLinkResponse {
        url: req.url()?.to_string(),
        original_url: model.original_url.to_string(),
        overwritten: already_exists,
        expiry_timestamp: model.expiry_timestamp,
        max_views: model.max_views,
    })
}

/// Deletes a Shortlink.
async fn delete_shortlink(req: Request, ctx: RouteContext<()>) -> Result<Response> {
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
