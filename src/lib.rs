mod auth;
mod kv;

use auth::is_authorized;
use kv::get_kv_store;
use serde::Deserialize;
use worker::{event, Context, Env, Request, Response, Result, RouteContext, Router, Url};

/// The HTML for the index page.
const INDEX_HTML: &str = include_str!("../static/index.html");

/// The payload for creating a link.
#[derive(Deserialize)]
struct CreateLinkPayload {
    url: String,
}

/// The payload for updating a link.
#[derive(Deserialize)]
struct UpdateLinkPayload {
    url: String,
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new()
        .get("/", |_, _| Response::from_html(INDEX_HTML))
        .get_async("/:id", get_link)
        .head_async("/:id", link_exists)
        .post_async("/:id", create_link)
        .put_async("/:id", update_link)
        .delete_async("/:id", delete_link);
    router.run(req, env).await
}

fn get_id_from_request(req: &Request) -> Result<String> {
    let path = req.path();
    let Some(id) = path.split("/").nth(1) else {
        return Err("ID was not provided".into());
    };

    Ok(id.to_string())
}

/// Checks if a link exists.
async fn link_exists(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let kv_store = get_kv_store(ctx)?;
    let id = get_id_from_request(&req)?;
    let value = match kv_store.get(&id).text().await? {
        Some(value) => value,
        None => return Response::error("A link with that ID was not found", 404),
    };
    Response::ok(value)
}

/// Gets a link.
async fn get_link(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let kv_store = get_kv_store(ctx)?;

    let id = get_id_from_request(&req)?;
    let value = match kv_store.get(&id).text().await? {
        Some(value) => value,
        None => return Response::error("A link with that ID was not found", 404),
    };
    let url = match Url::parse(&value) {
        Ok(url) => url,
        Err(_) => return Response::error("Unable to parse URL", 500),
    };

    Response::redirect(url)
}

/// Creates a link.
async fn create_link(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if !is_authorized(&req, &ctx)? {
        return Response::error("Unauthorized", 401);
    }

    let kv_store = get_kv_store(ctx)?;

    let id = get_id_from_request(&req)?;
    match kv_store.get(&id).text().await? {
        Some(_) => return Response::error("A link with that ID already exists", 409),
        None => (),
    };

    let body = req.json::<CreateLinkPayload>().await?;
    let url = match Url::parse(&body.url) {
        Ok(url) => url,
        Err(_) => return Response::error("Unable to parse URL", 500),
    };
    kv_store.put(&id, url.as_str())?.execute().await?;

    Response::ok("Successfully created link")
}

/// Deletes a link.
async fn delete_link(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if !is_authorized(&req, &ctx)? {
        return Response::error("Unauthorized", 401);
    }

    let kv_store = get_kv_store(ctx)?;

    let id = get_id_from_request(&req)?;
    match kv_store.get(&id).text().await? {
        Some(_) => (),
        None => return Response::error("A link with that ID was not found", 404),
    };
    kv_store.delete(&id).await?;

    Response::ok("Successfully deleted link")
}

/// Updates a link.
async fn update_link(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if !is_authorized(&req, &ctx)? {
        return Response::error("Unauthorized", 401);
    }

    let kv_store = get_kv_store(ctx)?;

    let id = get_id_from_request(&req)?;
    match kv_store.get(&id).text().await? {
        Some(_) => (),
        None => return Response::error("A link with that ID was not found", 404),
    };
    let body = req.json::<UpdateLinkPayload>().await?;
    let url = match Url::parse(&body.url) {
        Ok(url) => url,
        Err(_) => return Response::error("Unable to parse URL", 500),
    };
    kv_store.put(&id, url.as_str())?.execute().await?;

    Response::ok("Successfully updated link")
}
