use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image(String);

impl Image {
    pub fn from_base64(base64: impl Into<String>) -> Self {
        Self(base64.into())
    }

    pub fn to_base64(&self) -> &str {
        &self.0
    }
}
