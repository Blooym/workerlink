use worker::{Request, Result, RouteContext};

/// The binding name for the Authorization token variable.
const AUTH_TOKEN_BINDING: &str = "AUTH_TOKEN";

/// Checks if the request is authorized.
pub fn is_authorized(req: &Request, ctx: &RouteContext<()>) -> Result<bool> {
    let auth_token = ctx.var(AUTH_TOKEN_BINDING)?.to_string();
    let auth_header = match req.headers().get("Authorization")? {
        Some(header) => header,
        None => return Ok(false),
    };

    Ok(auth_header == auth_token)
}
