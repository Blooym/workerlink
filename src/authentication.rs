use crate::messages::{
    FORBIDDEN_REQUEST_RESPONSE, NOT_INITIALISED_WITH_AUTHTOKEN_RESPONSE,
    UNAUTHORIZED_REQUEST_RESPONSE,
};
use worker::{Request, Response, RouteContext};

/// The binding name for the Authorization token variable set in the Cloudflare Worker env vars.
const AUTH_TOKEN_BINDING: &str = "AUTH_TOKEN";

/// The header to check to find the Authorization token.
const AUTHORIZATION_HEADER: &str = "Authorization";

/// Represents a requests authorization state.
enum AuthorizationState {
    Authorized,
    Unauthorized,
    NoAuthorizationSent,
    InternalNoTokenSet,
}

/// Checks if the request is authorized by comparing the Authorization header to the [`AUTH_TOKEN_BINDING`] value.
fn is_request_authorized(
    req: &Request,
    ctx: &RouteContext<()>,
) -> worker::Result<AuthorizationState> {
    let auth_token = ctx.var(AUTH_TOKEN_BINDING)?.to_string();

    // It's better to play it safe and assume no token being set is user-error
    // and deny authenticated requests than to allow someone to not set one and get screwed over.
    if auth_token.is_empty() {
        return Ok(AuthorizationState::InternalNoTokenSet);
    }

    let auth_header = match req.headers().get(AUTHORIZATION_HEADER)? {
        Some(header) => header,
        None => return Ok(AuthorizationState::NoAuthorizationSent),
    };

    if auth_header == auth_token {
        Ok(AuthorizationState::Authorized)
    } else {
        Ok(AuthorizationState::Unauthorized)
    }
}

/// Guards a request by checking if it's authorized and returning a response value with an error if it isn't.
pub fn authorized_guard(
    req: &Request,
    ctx: &RouteContext<()>,
) -> Result<(), worker::Result<worker::Response>> {
    match is_request_authorized(&req, &ctx).unwrap() {
        AuthorizationState::Authorized => return Ok(()),
        AuthorizationState::Unauthorized => {
            return Err(Response::error(FORBIDDEN_REQUEST_RESPONSE, 403));
        }
        AuthorizationState::NoAuthorizationSent => {
            return Err(Response::error(UNAUTHORIZED_REQUEST_RESPONSE, 401));
        }
        AuthorizationState::InternalNoTokenSet => {
            return Err(Response::error(
                NOT_INITIALISED_WITH_AUTHTOKEN_RESPONSE,
                500,
            ));
        }
    }
}
