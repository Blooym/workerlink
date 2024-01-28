pub const UNAUTHORIZED_REQUEST_RESPONSE: &str = "Unauthorized";
pub const FORBIDDEN_REQUEST_RESPONSE: &str = "Forbidden";
pub const INVALID_PAYLOAD_RESPONSE: &str = "Invalid Payload";
pub const LINK_DOESNT_EXIST_RESPONSE: &str =
    "A link with that ID was not found, it may have been removed by its owner or expired.";
pub const NO_LINK_OWN_DOMAIN_RESPONSE: &str =
    "Cannot make a link redirect to the same domain as where link is hosted as this could cause an infinite redirect.";
pub const NOT_INITIALISED_WITH_AUTHTOKEN_RESPONSE: &str = "The link worker was initialised with no AUTH_TOKEN, all authenticated requests will be rejected until it has been set.";
pub const LINK_ALREADY_EXISTS_NO_OVERWRITE: &str =
    "A link with the given ID already exists and overwriting was not enabled.";
pub const GENERIC_LINK_CREATE_ERROR_RESPONSE: &str =
    "Something went wrong while trying to create a link.";
pub const GENERIC_LINK_DELETE_ERROR_RESPONSE: &str =
    "Something went wrong while trying to delete a link.";
pub const LINK_DELETE_SUCCESS_RESPONSE: &str = "link successfully deleted.";
