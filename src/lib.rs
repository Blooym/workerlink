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
use worker::{event, Context, Date, Env, Request, Response, RouteContext, Router, Url};

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> worker::Result<Response> {
    Router::new()
        .get("/", index_handler)
        .get("/favicon.ico", favicon_handler)
        .get_async("/:id", link_redirect_handler)
        .get_async("/:id/details", link_details_handler)
        .get_async("/:id/where", link_where_handler)
        .post_async("/:id", create_or_update_link_handler)
        .delete_async("/:id", delete_link_handler)
        .run(req, env)
        .await
}

/// Handler for the index (/) route.
fn index_handler(_req: Request, _ctx: RouteContext<()>) -> worker::Result<Response> {
    Response::from_html(include_str!("../static/index.html"))
}

/// Handler for the favicon (/favicon,ico) route.
fn favicon_handler(_req: Request, _ctx: RouteContext<()>) -> worker::Result<Response> {
    Response::from_bytes(include_bytes!("../static/favicon.ico").to_vec())
}

/// Get the link ID from a request.
fn get_link_id_from_request(req: &Request) -> worker::Result<String> {
    let path = req.path();
    let Some(id) = path.split('/').nth(1) else {
        Err("Unable to find link ID from request URL.")?
    };
    Ok(id.to_string())
}

/// Get the underlying redirect from a link ID.
async fn link_where_handler(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let storage = CloudflareKVDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);
    let key = get_link_id_from_request(&req)?;

    match storage.get_deserialized_json::<LinkModel>(&key).await {
        Some(link) => Response::ok(link.url.to_string()),
        None => Response::error(LINK_DOESNT_EXIST_RESPONSE, 404),
    }
}

/// Get a link and handle the redirect for it's linked URL.
async fn link_redirect_handler(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let storage = CloudflareKVDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);
    let key = get_link_id_from_request(&req)?;
    match storage.get_deserialized_json::<LinkModel>(&key).await {
        Some(mut value) => {
            // If the link has been disabled, act as it doesn't exist.
            if value.disabled {
                return Response::error(LINK_DOESNT_EXIST_RESPONSE, 404);
            }

            // If the link has expired due to time.
            if let Some(expires_at_ms) = value.expiry_timestamp {
                if Date::now().as_millis() > expires_at_ms {
                    storage.delete(&key).await;
                    return Response::error(LINK_DOESNT_EXIST_RESPONSE, 404);
                }
            }

            // If the link has reached it's maximum number of views.
            if let Some(max_views) = value.max_views {
                if value.views >= max_views {
                    storage.delete(&key).await;
                    return Response::error(LINK_DOESNT_EXIST_RESPONSE, 404);
                }
            }

            value.increment_visits();
            storage.set_serialized_json(&key, &value).await;
            Response::redirect(value.url)
        }
        None => return Response::error(LINK_DOESNT_EXIST_RESPONSE, 404),
    }
}

/// Get a link and return its details.
async fn link_details_handler(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let auth_guard = authorized_guard(&req, &ctx);
    if auth_guard.is_err() {
        return auth_guard.unwrap_err();
    }

    let storage = CloudflareKVDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);
    let key = get_link_id_from_request(&req)?;
    match storage.get_deserialized_json::<LinkModel>(&key).await {
        Some(value) => {
            // If the link has been disabled, act as it doesn't exist.
            if value.disabled {
                return Response::error(LINK_DOESNT_EXIST_RESPONSE, 404);
            }

            // If the Link has expired due to time.
            if let Some(expires_at_ms) = value.expiry_timestamp {
                if Date::now().as_millis() > expires_at_ms {
                    storage.delete(&key).await;
                    return Response::error(LINK_DOESNT_EXIST_RESPONSE, 404);
                }
            }

            // If the link has reached it's maximum number of views.
            if let Some(max_views) = value.max_views {
                if value.views >= max_views {
                    storage.delete(&key).await;
                    return Response::error(LINK_DOESNT_EXIST_RESPONSE, 404);
                }
            }
            Response::from_json(&value)
        }
        None => return Response::error(LINK_DOESNT_EXIST_RESPONSE, 404),
    }
}

/// Creates or updates a link.
async fn create_or_update_link_handler(
    mut req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    let auth_guard = authorized_guard(&req, &ctx);
    if auth_guard.is_err() {
        return auth_guard.unwrap_err();
    }

    let storage = CloudflareKVDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);
    let key = get_link_id_from_request(&req)?;
    let body = req.json::<CreateLinkRequestBody>().await?;

    if body.validate().is_err() {
        return Response::error(INVALID_PAYLOAD_RESPONSE, 400);
    }

    // Parse the given URL.
    let url: Url = match Url::parse(&body.url) {
        Ok(url) => url,
        Err(_) => return Response::error(UNABLE_TO_PARSE_URL_RESPONSE, 400),
    };

    // Prevent making a link that recurses forever on the same domain.
    if req.url()?.domain() == url.domain() {
        return Response::error(NO_LINK_OWN_DOMAIN_RESPONSE, 400);
    }

    // Grab existing model and check if we can overwrite it if it exists.
    let existing_model = storage.get_deserialized_json::<LinkModel>(&key).await;
    if !body.overwrite && existing_model.is_some() {
        return Response::error(LINK_ALREADY_EXISTS_NO_OVERWRITE, 409);
    }

    let model = match existing_model {
        Some(model) => model.modify(LinkBuilderArgs {
            url,
            max_views: body.max_views,
            disabled: body.disabled,
            expiry_timestamp: body
                .expire_in
                .map(|time| Date::now().as_millis() + time.as_millis() as u64),
        }),
        None => LinkModel::new(LinkBuilderArgs {
            url,
            max_views: body.max_views,
            disabled: body.disabled,
            expiry_timestamp: body
                .expire_in
                .map(|time| Date::now().as_millis() + time.as_millis() as u64),
        }),
    };

    if !storage
        .set_serialized_json::<&LinkModel>(&key, &model)
        .await
    {
        return Response::error(GENERIC_LINK_CREATE_ERROR_RESPONSE, 500);
    }

    Response::from_json(&CreateLinkResponse::from_model(&model, req.url()?))
}

/// Deletes a link.
async fn delete_link_handler(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let auth_guard = authorized_guard(&req, &ctx);
    if auth_guard.is_err() {
        return auth_guard.unwrap_err();
    }

    let storage = CloudflareKVDriver::new(ctx.kv(CLOUDFLARE_KV_BINDING)?);

    let key = get_link_id_from_request(&req)?;
    match storage.get(&key).await {
        Some(_) => (),
        None => return Response::error(LINK_DOESNT_EXIST_RESPONSE, 404),
    };

    if !storage.delete(&key).await {
        return Response::error(GENERIC_LINK_DELETE_ERROR_RESPONSE, 500);
    }

    Response::ok(LINK_DELETE_SUCCESS_RESPONSE)
}
