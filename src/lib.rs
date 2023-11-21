mod authentication;
mod storage;

use authentication::is_request_authorized;
use serde::{Deserialize, Serialize};
use storage::{
    cloudflare_storage_driver::{CloudflareStorageDriver, CLOUDFLARE_KV_BINDING},
    StorageDriver,
};
use worker::{event, Context, Env, Request, Response, Result, RouteContext, Router, Url};

/// The HTML for the index page.
const INDEX_HTML_CONTENT: &str = include_str!("../static/index.html");

/// The payload for creating a link.
#[derive(Deserialize)]
struct CreateLinkPayload {
    url: String,
    #[serde(default)]
    overwrite: bool,
}

/// The response returned when successfully creating a link.
#[derive(Serialize)]
struct CreateLinkResponse {
    url: String,
    overwritten: bool,
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get("/", |_, _| Response::from_html(INDEX_HTML_CONTENT))
        .get_async("/:id", get_link)
        .head_async("/:id", link_exists)
        .post_async("/:id", create_or_update_link)
        .delete_async("/:id", delete_link)
        .run(req, env)
        .await
}

/// Get the URL ID from a request.
fn get_shortlink_id_from_request(req: &Request) -> Result<String> {
    let path = req.path();
    let Some(id) = path.split('/').nth(1) else {
        Err("Unable to find shortlink id inside of request URL.")?
    };
    Ok(id.to_string())
}

/// Checks if a link exists.
async fn link_exists(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let kv_store = CloudflareStorageDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);
    let id = get_shortlink_id_from_request(&req)?;

    if kv_store.get_value(&id).await.is_some() {
        return Ok(Response::empty().unwrap().with_status(302));
    }

    Ok(Response::empty().unwrap().with_status(404))
}

/// Gets a link.
async fn get_link(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let kv_store: CloudflareStorageDriver =
        CloudflareStorageDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);

    let id = get_shortlink_id_from_request(&req)?;
    let value = match kv_store.get_value(&id).await {
        Some(value) => value,
        None => return Response::error("Shortlink does not exist or has been removed.", 404),
    };

    let url = match Url::parse(&value) {
        Ok(url) => url,
        Err(_) => {
            return Response::error(
                "Shortlink redirects to an invalid URL, unable to continue.",
                500,
            )
        }
    };

    Response::redirect(url)
}

/// Creates or updates link.
async fn create_or_update_link(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if !is_request_authorized(&req, &ctx)? {
        return Response::error("Unauthorized", 401);
    }

    let kv_store: CloudflareStorageDriver =
        CloudflareStorageDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);
    let short_url_name = get_shortlink_id_from_request(&req)?;
    let body = req.json::<CreateLinkPayload>().await?;

    let url = match Url::parse(&body.url) {
        Ok(url) => url,
        Err(_) => {
            return Response::error(
                "Unable to parse given URL, please ensure that it is valid.",
                500,
            )
        }
    };

    let already_exists = kv_store.get_value(&short_url_name).await.is_some();
    if !body.overwrite {
        return Response::error(
            "A link with the given ID already exists and overwriting was not enabled.",
            400,
        );
    }

    if !kv_store.set_value(&short_url_name, url.as_str()).await {
        return Response::error(
            "Something went wrong while trying to create shortlink.",
            500,
        );
    }

    Response::from_json(&CreateLinkResponse {
        url: req.url().unwrap().to_string(),
        overwritten: already_exists,
    })
}

/// Deletes a link.
async fn delete_link(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if !is_request_authorized(&req, &ctx)? {
        return Response::error("Unauthorized.", 401);
    }

    let kv_store: CloudflareStorageDriver =
        CloudflareStorageDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);

    let id = get_shortlink_id_from_request(&req)?;
    match kv_store.get_value(&id).await {
        Some(_) => (),
        None => return Response::error("A shortlink with that ID does not exist.", 404),
    };

    if !kv_store.delete_value(&id).await {
        return Response::error("Something went wrong whilst deleting that shortlink.", 500);
    }

    Response::ok("Successfully deleted shortlink.")
}
