pub const UNAUTHORIZED_REQUEST_RESPONSE: &str = "Unauthorized";
pub const FORBIDDEN_REQUEST_RESPONSE: &str = "Forbidden";
pub const INVALID_PAYLOAD_RESPONSE: &str = "Invalid Payload";
pub const LINK_DOESNT_EXIST_RESPONSE: &str =
    "A Link with that ID was not found, it may have been removed by its owner or expired.";
pub const NO_LINK_OWN_DOMAIN_RESPONSE: &str =
    "Cannot make a Link redirect to the same domain as where Link is hosted as this could cause an infinite redirect.";
pub const UNABLE_TO_PARSE_URL_RESPONSE: &str =
    "Unable to parse the given URL, please ensure that it is valid.";
pub const NOT_INITIALISED_WITH_AUTHTOKEN_RESPONSE: &str = "The Link worker was initialised with no AUTH_TOKEN, all authenticated requests will be rejected until it has been set.";
pub const LINK_ALREADY_EXISTS_NO_OVERWRITE: &str =
    "A link with the given ID already exists and overwriting was not enabled.";
pub const GENERIC_LINK_CREATE_ERROR_RESPONSE: &str =
    "Something went wrong while trying to create a Link.";
pub const GENERIC_LINK_DELETE_ERROR_RESPONSE: &str =
    "Something went wrong while trying to delete a Link.";
pub const LINK_DELETE_SUCCESS_RESPONSE: &str = "Link successfully deleted.";
