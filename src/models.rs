use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct PreLoginRequest {
    #[schema(format="email")]
    #[validate(email)]
    pub email: String,
}
