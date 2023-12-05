use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SPOErrorResponse {
    pub error: Error,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Error {
    pub code: String,
    pub message: Message,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub lang: String,
    pub value: String,
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SPOContextInfoResponse {
    pub d: D,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct D {
    #[serde(rename = "GetContextWebInformation")]
    pub get_context_web_information: GetContextWebInformation,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetContextWebInformation {
    #[serde(rename = "__metadata")]
    pub metadata: Metadata,
    #[serde(rename = "FormDigestTimeoutSeconds")]
    pub form_digest_timeout_seconds: i64,
    #[serde(rename = "FormDigestValue")]
    pub form_digest_value: String,
    #[serde(rename = "LibraryVersion")]
    pub library_version: String,
    #[serde(rename = "SiteFullUrl")]
    pub site_full_url: String,
    #[serde(rename = "SupportedSchemaVersions")]
    pub supported_schema_versions: SupportedSchemaVersions,
    #[serde(rename = "WebFullUrl")]
    pub web_full_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportedSchemaVersions {
    #[serde(rename = "__metadata")]
    pub metadata: Metadata2,
    pub results: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata2 {
    #[serde(rename = "type")]
    pub type_field: String,
}
