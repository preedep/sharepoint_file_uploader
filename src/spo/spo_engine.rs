use std::fmt::{Display, Formatter};

use log::debug;
use oauth2::http::HeaderMap;
use reqwest::Client;
use uuid::Uuid;

use crate::spo::spo_endpoint::SPOEndpoint;
use crate::spo::spo_model::{SPOContextInfoResponse, SPOErrorResponse, SPOTokenResponse};

#[derive(Debug)]
pub struct SPOError {
    message: String,
}

impl SPOError {
    pub fn new(message: &String) -> SPOError {
        SPOError {
            message: message.clone(),
        }
    }
}

impl Display for SPOError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SPOError: {}", self.message)
    }
}

pub struct SPOEngine {
    tenant_id: String,
    client_id: String,
    client_secret: String,
    share_point_domain: String,
    end_point: SPOEndpoint,

    uuid: Option<String>,
    token: Option<SPOTokenResponse>,
    context_info: Option<SPOContextInfoResponse>,
}

impl SPOEngine {
    pub fn new(
        tenant_id: &String,
        client_id: &String,
        client_secret: &String,
        share_point_domain: &String,
    ) -> SPOEngine {
        SPOEngine {
            tenant_id: tenant_id.to_owned(),
            client_id: client_id.to_owned(),
            client_secret: client_secret.to_owned(),
            share_point_domain: share_point_domain.to_owned(),
            end_point: SPOEndpoint::new(share_point_domain),
            uuid: None,
            token: None,
            context_info: None,
        }
    }
    //
    //  Upload One Time , Upload one time to Share point online
    //
    pub async fn upload_one_time(
        &mut self,
        site: &String,
        path: &String,
        file_name: &String,
        data: &[u8],
    ) -> Result<(), SPOError> {
        let mut end_point = self
            .end_point
            .set_site(site)
            .set_path(path)
            .set_file_name(file_name);
        self.end_point = end_point.clone();

        let token = get_spo_token(
            &self.tenant_id,
            &self.client_id,
            &self.client_secret,
            &self.share_point_domain,
        )
            .await;
        match token {
            Ok(t) => {
                debug!("token: {:#?}", t);
                self.token = Some(t);
            }
            Err(e) => {
                return Err(SPOError::new(&format!("get_spo_token error: {}", e)));
            }
        }
        let context_info = get_spo_digest_value(
            &self.end_point.to_spo_digest_url(),
            &self.token.clone().unwrap().access_token.unwrap(),
        )
            .await;
        match context_info {
            Ok(d) => {
                debug!("context_info: {:#?}", d);
                self.context_info = Some(d);
            }
            Err(e) => {
                return Err(SPOError::new(&format!("get_spo_digest_value error: {}", e)));
            }
        }
        transfer_data_to_spo(
            &self.end_point.to_file_one_time_upload_endpoint(),
            &self.context_info.clone().unwrap(),
            &self.token.clone().unwrap().access_token.unwrap(),
            &data.to_vec(),
        )
            .await
            .map_err(|e| SPOError::new(&format!("to_file_one_time_upload_endpoint error: {:#?}", e)))
    }
    //
    //  Upload Start , Start for upload multiple chunk to Share point online
    //
    pub async fn upload_start(
        &mut self,
        site: &String,
        path: &String,
        file_name: &String,
        data: &[u8],
    ) -> Result<(), SPOError> {
        let uuid = Uuid::new_v4();
        let mut end_point = self.end_point.set_uuid(&uuid.to_string());
        self.end_point = end_point.clone();

        //save empty file first
        //if not save empty file first, will get error , file not found from share point online
        let empty_data = vec![];
        let rs = self
            .upload_one_time(site, path, file_name, empty_data.as_slice())
            .await;
        match rs {
            Ok(_) => {
                debug!("Create empty file success");
            }
            Err(e) => {
                return Err(SPOError::new(&format!("upload_one_time error: {}", e)));
            }
        }
        //upload file
        transfer_data_to_spo(
            &self.end_point.to_file_start_upload_endpoint(),
            &self.context_info.clone().unwrap(),
            &self.token.clone().unwrap().access_token.unwrap(),
            &data.to_vec(),
        )
            .await
            .map_err(|e| SPOError::new(&format!("transfer_data_to_spo error: {:#?}", e)))
    }
    //
    //  Upload Continue , Continue for upload multiple chunk to Share point online
    //
    pub async fn upload_continue(
        &mut self,
        data: &[u8],
        file_offset: &u64,
    ) -> Result<(), SPOError> {
        let end_point = self.end_point.set_offset(file_offset);
        self.end_point = end_point.clone();

        transfer_data_to_spo(
            &self.end_point.to_file_continue_upload_endpoint(),
            &self.context_info.clone().unwrap(),
            &self.token.clone().unwrap().access_token.unwrap(),
            &data.to_vec(),
        )
            .await
            .map_err(|e| SPOError::new(&format!("transfer_data_to_spo error: {:#?}", e)))
    }
    //
    //  Upload  Finish,  Finish for upload multiple chunk to Share point online
    //
    pub async fn upload_finish(&mut self, data: &[u8], file_offset: &u64) -> Result<(), SPOError> {
        let end_point = self.end_point.set_offset(file_offset);
        self.end_point = end_point.clone();

        transfer_data_to_spo(
            &self.end_point.to_file_finish_upload_endpoint(),
            &self.context_info.clone().unwrap(),
            &self.token.clone().unwrap().access_token.unwrap(),
            &data.to_vec(),
        )
            .await
            .map_err(|e| SPOError::new(&format!("transfer_data_to_spo error: {:#?}", e)))
    }
}

async fn get_spo_token(
    tenant_id: &String,
    client_id: &String,
    client_secret: &String,
    share_point_domain: &String,
) -> Result<SPOTokenResponse, reqwest::Error> {
    //https://accounts.accesscontrol.windows.net/5612aad0-a1b7-4391-87a7-389e38e63b73/tokens/OAuth/2
    let url = format!(
        "https://accounts.accesscontrol.windows.net/{tenant_id}/tokens/OAuth/2",
        tenant_id = tenant_id,
    );
    let body = format!(
        r#"grant_type=client_credentials&client_id={client_id}@{tenant_id}&client_secret={client_secret}&resource=00000003-0000-0ff1-ce00-000000000000/{share_point_domain}.sharepoint.com@{tenant_id}"#,
        client_id = client_id,
        tenant_id = tenant_id,
        client_secret = client_secret,
        share_point_domain = share_point_domain
    );
    let mut headers = HeaderMap::new();
    headers.append(
        "Content-Type",
        "application/x-www-form-urlencoded".parse().unwrap(),
    );
    let res_post = Client::new()
        .post(url)
        .headers(headers)
        .body(body)
        .send()
        .await?
        .json::<SPOTokenResponse>()
        .await
        .map_err(|e| e);
    res_post
}

async fn transfer_data_to_spo(
    spo_save_endpoint: &String,
    digest: &SPOContextInfoResponse,
    spo_access_token: &String,
    data: &[u8],
) -> Result<(), reqwest::Error> {
    debug!("transfer_data_to_spo with url : {:?}", spo_save_endpoint);

    let mut headers = HeaderMap::new();
    headers.append(
        "Authorization",
        format!("Bearer {}", spo_access_token).parse().unwrap(),
    );
    headers.append(
        "Content-Type",
        "application/json;odata=verbose".parse().unwrap(),
    );
    headers.append("Accept", "application/json;odata=verbose".parse().unwrap());
    headers.append("Content-Length", data.len().to_string().parse().unwrap());
    headers.append(
        "X-RequestDigest",
        digest
            .d
            .get_context_web_information
            .form_digest_value
            .parse()
            .unwrap(),
    );
    let res = Client::new()
        .post(spo_save_endpoint)
        .headers(headers.clone())
        .body(data.to_owned())
        .send()
        .await;
    match res {
        Ok(r) => {
            if r.status().is_success() {
                debug!("Success Upload");
            } else {
                let res_json = r.json::<SPOErrorResponse>().await?;
                //error!("Error: {:#?}", res_json);
                panic!("Error: {:#?}", res_json);
            }
        }
        Err(e) => {
            panic!("url : {}\n{}", spo_save_endpoint, e);
        }
    };
    Ok(())
}

async fn get_spo_digest_value(
    spo_digest_endpoint: &String,
    spo_access_token: &String,
) -> Result<SPOContextInfoResponse, reqwest::Error> {
    debug!("spo_digest_endpoint: {:?}", spo_digest_endpoint);

    let mut headers = HeaderMap::new();
    headers.append(
        "Authorization",
        format!("Bearer {}", spo_access_token).parse().unwrap(),
    );
    headers.append("Accept", "application/json;odata=verbose".parse().unwrap());
    headers.append(
        "Content-Type",
        "application/json;odata=verbose".parse().unwrap(),
    );

    Client::new()
        .post(spo_digest_endpoint)
        .headers(headers.clone())
        .send()
        .await?
        .json::<SPOContextInfoResponse>()
        .await
        .map_err(|e| e)
}
