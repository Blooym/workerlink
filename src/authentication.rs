use worker::{Request, Result, RouteContext};

/// The binding name for the Authorization token variable set in Cloudflare.
const AUTH_TOKEN_BINDING: &str = "AUTH_TOKEN";

/// The header to check to find the Authorization token.
const AUTHORIZATION_HEADER: &str = "Authorization";

/// Checks if the request is authorized by comparing the Authorization header to the [`AUTH_TOKEN_BINDING`] value.
pub fn is_request_authorized(req: &Request, ctx: &RouteContext<()>) -> Result<bool> {
    let auth_token = ctx.var(AUTH_TOKEN_BINDING)?.to_string();
    let auth_header = match req.headers().get(AUTHORIZATION_HEADER)? {
        Some(header) => header,
        None => return Ok(false),
    };
    Ok(auth_header == auth_token)
}
