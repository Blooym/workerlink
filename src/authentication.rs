use worker::{Request, Response, RouteContext};

/// The binding name for the Authorization token variable set in Cloudflare.
const AUTH_TOKEN_BINDING: &str = "AUTH_TOKEN";

/// The header to check to find the Authorization token.
const AUTHORIZATION_HEADER: &str = "Authorization";

pub enum AuthStatus {
    Authorized,
    Unauthorized,
    NoAuthorizationSent,
}

/// Checks if the request is authorized by comparing the Authorization header to the [`AUTH_TOKEN_BINDING`] value.
pub fn is_request_authorized(req: &Request, ctx: &RouteContext<()>) -> worker::Result<AuthStatus> {
    let auth_token = ctx.var(AUTH_TOKEN_BINDING)?.to_string();
    let auth_header = match req.headers().get(AUTHORIZATION_HEADER)? {
        Some(header) => header,
        None => return Ok(AuthStatus::NoAuthorizationSent),
    };

    if auth_header == auth_token {
        Ok(AuthStatus::Authorized)
    } else {
        Ok(AuthStatus::Unauthorized)
    }
}

pub fn authorized_guard(
    req: &Request,
    ctx: &RouteContext<()>,
) -> Result<(), worker::Result<worker::Response>> {
    match is_request_authorized(&req, &ctx).unwrap() {
        AuthStatus::Authorized => return Ok(()),
        AuthStatus::Unauthorized => {
            return Err(Response::error("Forbidden", 403));
        }
        AuthStatus::NoAuthorizationSent => {
            return Err(Response::error("Unauthorized", 401));
        }
    }
}
