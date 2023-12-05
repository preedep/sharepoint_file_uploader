use serde::{Deserialize, Serialize};

///
/// Open ID Configuration
///
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenIDConfigurationV2 {
    #[serde(rename = "token_endpoint")]
    pub token_endpoint: Option<String>,
    #[serde(rename = "token_endpoint_auth_methods_supported")]
    pub token_endpoint_auth_methods_supported: Option<Vec<String>>,
    #[serde(rename = "jwks_uri")]
    pub jwks_uri: Option<String>,
    #[serde(rename = "response_modes_supported")]
    pub response_modes_supported: Option<Vec<String>>,
    #[serde(rename = "subject_types_supported")]
    pub subject_types_supported: Option<Vec<String>>,
    #[serde(rename = "id_token_signing_alg_values_supported")]
    pub id_token_signing_alg_values_supported: Option<Vec<String>>,
    #[serde(rename = "response_types_supported")]
    pub response_types_supported: Option<Vec<String>>,
    #[serde(rename = "scopes_supported")]
    pub scopes_supported: Option<Vec<String>>,
    pub issuer: Option<String>,
    #[serde(rename = "request_uri_parameter_supported")]
    pub request_uri_parameter_supported: Option<bool>,
    #[serde(rename = "userinfo_endpoint")]
    pub userinfo_endpoint: Option<String>,
    #[serde(rename = "authorization_endpoint")]
    pub authorization_endpoint: Option<String>,
    #[serde(rename = "device_authorization_endpoint")]
    pub device_authorization_endpoint: Option<String>,
    #[serde(rename = "http_logout_supported")]
    pub http_logout_supported: Option<bool>,
    #[serde(rename = "frontchannel_logout_supported")]
    pub frontchannel_logout_supported: Option<bool>,
    #[serde(rename = "end_session_endpoint")]
    pub end_session_endpoint: Option<String>,
    #[serde(rename = "claims_supported")]
    pub claims_supported: Option<Vec<String>>,
    #[serde(rename = "kerberos_endpoint")]
    pub kerberos_endpoint: Option<String>,
    #[serde(rename = "tenant_region_scope")]
    pub tenant_region_scope: Option<String>,
    #[serde(rename = "cloud_instance_name")]
    pub cloud_instance_name: Option<String>,
    #[serde(rename = "cloud_graph_host_name")]
    pub cloud_graph_host_name: Option<String>,
    #[serde(rename = "msgraph_host")]
    pub msgraph_host: Option<String>,
    #[serde(rename = "rbac_url")]
    pub rbac_url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JWKS {
    pub keys: Option<Vec<JWKSKeyItem>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JWKSKeyItem {
    pub kty: Option<String>,
    #[serde(rename = "use")]
    pub use_field: Option<String>,
    pub kid: Option<String>,
    pub x5t: Option<String>,
    pub n: Option<String>,
    pub e: Option<String>,
    pub x5c: Option<Vec<String>>,
    pub issuer: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IDToken {
    pub aud: Option<String>,
    pub iss: Option<String>,
    pub iat: Option<i64>,
    pub nbf: Option<i64>,
    pub exp: Option<i64>,
    pub acct: Option<i64>,
    pub acrs: Option<Vec<String>>,
    pub aio: Option<String>,
    #[serde(rename = "auth_time")]
    pub auth_time: Option<i64>,
    pub ctry: Option<String>,
    pub email: Option<String>,
    #[serde(rename = "family_name")]
    pub family_name: Option<String>,
    #[serde(rename = "given_name")]
    pub given_name: Option<String>,
    pub groups: Option<Vec<String>>,
    pub idp: Option<String>,
    pub ipaddr: Option<String>,
    #[serde(rename = "login_hint")]
    pub login_hint: Option<String>,
    pub name: Option<String>,
    pub nonce: Option<String>,
    pub oid: Option<String>,
    #[serde(rename = "preferred_username")]
    pub preferred_username: Option<String>,
    pub rh: Option<String>,
    pub sid: Option<String>,
    pub sub: Option<String>,
    #[serde(rename = "tenant_ctry")]
    pub tenant_ctry: Option<String>,
    #[serde(rename = "tenant_region_scope")]
    pub tenant_region_scope: Option<String>,
    pub tid: Option<String>,
    pub uti: Option<String>,
    pub ver: Option<String>,
    pub wids: Option<Vec<String>>,
    #[serde(rename = "xms_pl")]
    pub xms_pl: Option<String>,
    #[serde(rename = "xms_tpl")]
    pub xms_tpl: Option<String>,
    #[serde(rename = "employee_id")]
    pub employee_id: Option<String>,
    #[serde(rename = "department")]
    pub department: Option<String>,
    #[serde(rename = "companyname")]
    pub companyname: Option<String>,
    #[serde(rename = "officelocation")]
    pub officelocation: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResponseAuthorized {
    #[serde(rename(deserialize = "code"))]
    pub code: Option<String>,
    #[serde(rename(deserialize = "session_state"))]
    pub session_state: Option<String>,
    #[serde(rename(deserialize = "state"))]
    pub state: Option<String>,
    #[serde(rename(deserialize = "id_token"))]
    pub id_token: Option<String>,
    #[serde(rename(deserialize = "error"))]
    pub error: Option<String>,
    #[serde(rename(deserialize = "error_description"))]
    pub error_description: Option<String>,
    #[serde(rename(deserialize = "access_token"))]
    pub access_token: Option<String>,
    #[serde(rename(deserialize = "token_type"))]
    pub token_type: Option<String>,
    #[serde(rename(deserialize = "scope"))]
    pub scope: Option<String>,
    #[serde(rename(deserialize = "expires_in"))]
    pub expires_in: Option<i64>,
}
