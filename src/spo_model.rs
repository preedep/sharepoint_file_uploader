use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct SPOEndpoint {
    share_point_domain: String,
    share_point_site: String,
    path: Option<String>,
    file_name: Option<String>,
    offset: Option<u64>,
    uuid: Option<String>,
}

impl SPOEndpoint {
    pub fn new(share_point_domain: &String, share_point_site: &String) -> SPOEndpoint {
        SPOEndpoint {
            share_point_domain: share_point_domain.clone(),
            share_point_site: share_point_site.clone(),
            path: None,
            file_name: None,
            offset: None,
            uuid: None,
        }
    }

    pub fn set_path(&mut self, path: &String) -> SPOEndpoint {
        self.path = Some(path.clone());
        self.clone()
    }
    pub fn set_file_name(&mut self, file_name: &String) -> SPOEndpoint {
        self.file_name = Some(file_name.clone());
        self.clone()
    }
    pub fn set_offset(&mut self, offset: &u64) -> SPOEndpoint {
        self.offset = Some(offset.clone());
        self.clone()
    }
    pub fn set_uuid(&mut self, uuid: &String) -> SPOEndpoint {
        self.uuid = Some(uuid.clone());
        self.clone()
    }
    pub fn to_spo_web_url(&self) -> String {
        format!("https://{share_point_domain}.sharepoint.com/sites/{share_point_site}/_api",
                share_point_domain = self.share_point_domain,
                share_point_site = self.share_point_site)
    }
    pub fn to_spo_digest_url(&self) -> String {
        format!("{web_url}/ContextInfo",
                web_url = self.to_spo_web_url())
    }
    pub fn to_spo_one_time_save_endpoint(&self) -> String {
        let url = format!("{web_url}/web/GetFolderByServerRelativeUrl('{path}')/Files/add(url='{file_name}',overwrite=true)",
                          web_url = self.to_spo_web_url(),
                          path = self.path.clone().unwrap(),
                          file_name = self.file_name.clone().unwrap());
        url
    }
    pub fn to_spo_start_upload_endpoint(&self) -> String {
        let url = format!("{web_url}/web/GetFileByServerRelativeUrl('{path}/{file_name}')/StartUpload(uploadId='{uuid}')",
                          web_url = self.to_spo_web_url(),
                          path = self.path.clone().unwrap(),
                          file_name = self.file_name.clone().unwrap(),
                          uuid = self.uuid.clone().unwrap()
        );
        url
    }
    pub fn to_spo_start_continue_endpoint(&self) -> String {
        let url = format!("{web_url}/web/GetFileByServerRelativeUrl('{path}/{file_name}')/ContinueUpload(uploadId='{uuid}',fileOffset={offset})",
                          web_url = self.to_spo_web_url(),
                          path = self.path.clone().unwrap(),
                          file_name = self.file_name.clone().unwrap(),
                          uuid = self.uuid.clone().unwrap(),
                          offset = self.offset.clone().unwrap()
        );
        url
    }
    pub fn to_spo_start_finish_endpoint(&self) -> String {
        let url = format!("{web_url}/GetFileByServerRelativeUrl('{path}/{file_name}')/FinishUpload(uploadId='{uuid}',fileOffset={offset})",
                          web_url = self.to_spo_web_url(),
                          path = self.path.clone().unwrap(),
                          file_name = self.file_name.clone().unwrap(),
                          uuid = self.uuid.clone().unwrap(),
                          offset = self.offset.clone().unwrap()
        );
        url
    }
}


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


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SPOTokenResponse {
    #[serde(rename = "token_type")]
    pub token_type: Option<String>,
    #[serde(rename = "expires_in")]
    pub expires_in: Option<String>,
    #[serde(rename = "not_before")]
    pub not_before: Option<String>,
    #[serde(rename = "expires_on")]
    pub expires_on: Option<String>,
    pub resource: Option<String>,
    #[serde(rename = "access_token")]
    pub access_token: Option<String>,
}

