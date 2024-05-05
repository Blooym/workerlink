mod api;
mod authentication;
mod messages;
mod models;
mod storage;

use api::{requests::CreateLinkRequestBody, responses::CreateLinkResponse};
use authentication::authorized_guard;
use messages::*;
use models::link::{LinkBuilderArgs, LinkModel};
use storage::{
    cloudflare_kv_driver::{CloudflareKVDriver, CLOUDFLARE_KV_BINDING},
    StorageDriver,
};
use validator::Validate;
use worker::{event, Context, Date, Env, Request, Response, RouteContext, Router};

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> worker::Result<Response> {
    Router::new()
        .get("/", index_handler)
        .get("/favicon.ico", favicon_handler)
        .get("/robots.txt", robots_handler)
        .get_async("/:id", link_redirect_handler)
        .post_async("/:id", create_or_update_link_handler)
        .delete_async("/:id", delete_link_handler)
        .get_async("/:id/where", link_where_handler)
        .get_async("/:id/details", link_details_handler)
        .run(req, env)
        .await
}

/// Handler to serve the index HTML.
fn index_handler(_req: Request, _ctx: RouteContext<()>) -> worker::Result<Response> {
    Response::from_html(include_str!("../static/index.html"))
}

/// Handler to serve the site favicon.
fn favicon_handler(_req: Request, _ctx: RouteContext<()>) -> worker::Result<Response> {
    let mut response =
        Response::from_bytes(include_bytes!("../static/favicon.ico").to_vec()).unwrap();
    response
        .headers_mut()
        .append("Content-Type", "image/vnd.microsoft.icon")
        .unwrap();

    Ok(response)
}

/// Handler to serve the robots.txt.
fn robots_handler(_req: Request, _ctx: RouteContext<()>) -> worker::Result<Response> {
    let mut response =
        Response::from_bytes(include_bytes!("../static/robots.txt").to_vec()).unwrap();
    response
        .headers_mut()
        .append("Content-Type", "text/plain")
        .unwrap();
    Ok(response)
}

/// Get the link ID from a request.
fn get_link_id_from_req(req: &Request) -> worker::Result<String> {
    let path = req.path();
    let Some(id) = path.split('/').nth(1) else {
        Err("Unable to find link ID from request URL.")?
    };
    Ok(id.to_string())
}

/// Handle a visit to /:id by attempting to find the key in storage and redirecting to the assigned url.
///
/// This handler will also deal with the following:
///     - Incrementing the visits count and storing the updated value
///     - Deleting the key from storage if it is no longer valid (exceeds max views, timed expiry, etc.)
async fn link_redirect_handler(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let storage = CloudflareKVDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);
    let id = get_link_id_from_req(&req)?;

    match storage.get_deserialized_json::<LinkModel>(&id).await {
        Some(mut link) => {
            if link.disabled {
                return Response::error(LINK_DOESNT_EXIST_RESPONSE, 404);
            }

            if !link.is_valid() {
                storage.delete(&id).await;
                return Response::error(LINK_DOESNT_EXIST_RESPONSE, 404);
            }

            link.increment_visits();
            storage.set_serialized_json(&id, &link).await;
            Response::redirect(link.url)
        }
        None => Response::error(LINK_DOESNT_EXIST_RESPONSE, 404),
    }
}

/// Get the underlying redirect from a link key.
async fn link_where_handler(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let storage = CloudflareKVDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);
    let id = get_link_id_from_req(&req)?;

    match storage.get_deserialized_json::<LinkModel>(&id).await {
        Some(link) => {
            if link.disabled {
                return Response::error(LINK_DOESNT_EXIST_RESPONSE, 404);
            }

            if !link.is_valid() {
                storage.delete(&id).await;
                return Response::error(LINK_DOESNT_EXIST_RESPONSE, 404);
            }

            Response::ok(link.url.to_string())
        }
        None => Response::error(LINK_DOESNT_EXIST_RESPONSE, 404),
    }
}

/// Get a link and return its details as JSON.
async fn link_details_handler(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let auth_guard = authorized_guard(&req, &ctx);
    if let Err(err) = auth_guard {
        return err;
    }

    let storage = CloudflareKVDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);
    let id = get_link_id_from_req(&req)?;

    match storage.get_deserialized_json::<LinkModel>(&id).await {
        Some(link) => {
            if !link.is_valid() {
                storage.delete(&id).await;
                return Response::error(LINK_DOESNT_EXIST_RESPONSE, 404);
            }

            Response::from_json(&link)
        }
        None => Response::error(LINK_DOESNT_EXIST_RESPONSE, 404),
    }
}

/// Create a new link or update an existing one.
async fn create_or_update_link_handler(
    mut req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    let auth_guard = authorized_guard(&req, &ctx);
    if let Err(err) = auth_guard {
        return err;
    }

    let storage = CloudflareKVDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);
    let id: String = get_link_id_from_req(&req)?;

    // Validate the JSON from the request can be deserialized.
    let Ok(body) = req.json::<CreateLinkRequestBody>().await else {
        return Response::error(INVALID_PAYLOAD_RESPONSE, 400);
    };

    // Validate that the struct is valid using the custom struct validator.
    if body.validate().is_err() {
        return Response::error(INVALID_PAYLOAD_RESPONSE, 400);
    }

    // Prevent making a link that recurses forever on the same domain.
    if req.url()?.domain() == body.url.domain() {
        return Response::error(NO_LINK_OWN_DOMAIN_RESPONSE, 400);
    }

    // Grab the existing model and check if we can overwrite it (if it exists).
    let existing_model = storage.get_deserialized_json::<LinkModel>(&id).await;
    if !body.overwrite && existing_model.is_some() {
        return Response::error(LINK_ALREADY_EXISTS_NO_OVERWRITE, 409);
    }

    let model = match existing_model {
        Some(model) => model.modify(LinkBuilderArgs {
            url: body.url,
            max_views: body.max_views,
            disabled: body.disabled,
            expiry_timestamp: body
                .expire_in
                .map(|time| Date::now().as_millis() + time.as_millis() as u64),
        }),
        None => LinkModel::new(LinkBuilderArgs {
            url: body.url,
            max_views: body.max_views,
            disabled: body.disabled,
            expiry_timestamp: body
                .expire_in
                .map(|time| Date::now().as_millis() + time.as_millis() as u64),
        }),
    };

    if !storage.set_serialized_json::<&LinkModel>(&id, &model).await {
        return Response::error(GENERIC_LINK_CREATE_ERROR_RESPONSE, 500);
    }

    Response::from_json(&CreateLinkResponse::from_model(&model, req.url()?))
}

/// Delete a link.
async fn delete_link_handler(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let auth_guard = authorized_guard(&req, &ctx);
    if let Err(err) = auth_guard {
        return err;
    }

    let storage = CloudflareKVDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);

    let id = get_link_id_from_req(&req)?;
    match storage.get(&id).await {
        Some(_) => (),
        None => return Response::error(LINK_DOESNT_EXIST_RESPONSE, 404),
    };

    if !storage.delete(&id).await {
        return Response::error(GENERIC_LINK_DELETE_ERROR_RESPONSE, 500);
    }

    Response::ok(LINK_DELETE_SUCCESS_RESPONSE)
}
