#[derive(Debug, Clone)]
pub struct SPOEndpoint {
    share_point_domain: String,
    share_point_site: Option<String>,
    path: Option<String>,
    file_name: Option<String>,
    offset: Option<u64>,
    uuid: Option<String>,
}

impl SPOEndpoint {
    pub fn new(share_point_domain: &String) -> SPOEndpoint {
        SPOEndpoint {
            share_point_domain: share_point_domain.to_owned(),
            share_point_site: None,
            path: None,
            file_name: None,
            offset: None,
            uuid: None,
        }
    }
    pub fn set_site(&mut self, site: &String) -> SPOEndpoint {
        self.share_point_site = Some(site.to_owned());
        self.to_owned()
    }
    pub fn set_path(&mut self, path: &String) -> SPOEndpoint {
        self.path = Some(path.to_owned());
        self.to_owned()
    }
    pub fn set_file_name(&mut self, file_name: &String) -> SPOEndpoint {
        self.file_name = Some(file_name.to_owned());
        self.to_owned()
    }
    pub fn set_offset(&mut self, offset: &u64) -> SPOEndpoint {
        self.offset = Some(offset.to_owned());
        self.to_owned()
    }
    pub fn set_uuid(&mut self, uuid: &String) -> SPOEndpoint {
        self.uuid = Some(uuid.to_owned());
        self.to_owned()
    }
    pub fn to_spo_web_url(&self) -> String {
        format!(
            "https://{share_point_domain}.sharepoint.com/sites/{share_point_site}",
            share_point_domain = self.share_point_domain,
            share_point_site = self.share_point_site.clone().unwrap()
        )
    }
    pub fn to_spo_domain_url(&self) -> String {
        format!(
            "https://{share_point_domain}.sharepoint.com",
            share_point_domain = self.share_point_domain,
        )
    }
    pub fn to_spo_digest_url(&self) -> String {
        format!(
            "{web_url}/_api/ContextInfo",
            web_url = self.to_spo_web_url()
        )
    }
    pub fn to_file_one_time_upload_endpoint(&self) -> String {
        format!("{web_url}/_api/web/GetFolderByServerRelativeUrl('{path}')/Files/add(url='{file_name}',overwrite=true)'",
                web_url = self.to_spo_web_url(),
                path = self.path.clone().unwrap(),
                file_name = self.file_name.clone().unwrap())
    }
    pub fn to_file_start_upload_endpoint(&self) -> String {
        format!("{web_url}/_api/web/GetFileByServerRelativeUrl('{path}/{file_name}')/StartUpload(uploadId=guid'{uuid}')",
                web_url = self.to_spo_web_url(),
                path = self.path.clone().unwrap(),
                file_name = self.file_name.clone().unwrap(),
                uuid = self.uuid.clone().unwrap()
        )
    }
    pub fn to_file_continue_upload_endpoint(&self) -> String {
        format!("{web_url}/_api/web/GetFileByServerRelativeUrl('{path}/{file_name}')/ContinueUpload(uploadId=guid'{uuid}',fileOffset={offset})",
                web_url = self.to_spo_web_url(),
                path = self.path.clone().unwrap(),
                file_name = self.file_name.clone().unwrap(),
                uuid = self.uuid.clone().unwrap(),
                offset = self.offset.clone().unwrap()
        )
    }
    pub fn to_file_finish_upload_endpoint(&self) -> String {
        format!("{web_url}/_api/web/GetFileByServerRelativeUrl('{path}/{file_name}')/FinishUpload(uploadId=guid'{uuid}',fileOffset={offset})",
                web_url = self.to_spo_web_url(),
                path = self.path.clone().unwrap(),
                file_name = self.file_name.clone().unwrap(),
                uuid = self.uuid.clone().unwrap(),
                offset = self.offset.clone().unwrap()
        )
    }
}
