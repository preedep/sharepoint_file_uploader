use std::error::Error;
use std::sync::Arc;

use azure_core::auth::TokenCredential;
use azure_identity::DefaultAzureCredential;
use azure_storage::prelude::*;
use azure_storage_blobs::prelude::*;
use futures::StreamExt;
use log::debug;
use reqwest::Client;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::spo_model::{SPOContextInfoResponse, SPOErrorResponse};

mod entra_id_model;
mod spo_model;

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

const MAX_CHUNK_SIZE: usize = 100 * 1024 * 1024; // 100MB

fn get_spo_save_endpoint(
    share_point_domain: &String,
    share_point_site: &String,
    path: &String,
    file_name: &String,
) -> String {
    //
    //   https://nickdev001.sharepoint.com/sites/MVP/_api/web/GetFileByServerRelativeUrl('/sites/MVP/Shared%20Documents/@{linkedService().SPOFileName}')/$value
    //   http://test.sharepoint.com/sites/testsite/_api/web/GetFolderByServerRelativeUrl('/Library Name/Folder Name')/Files/add(url='a.txt',overwrite=true)
    //
    let url = format!(
        r#"https://{share_point_domain}.sharepoint.com/sites/{share_point_site}/_api/web/GetFolderByServerRelativeUrl('{path}')/Files/add(url='{file_name}',overwrite=true)"#,
        share_point_domain = share_point_domain,
        share_point_site = share_point_site,
        path = path,
        file_name = file_name
    );
    url
}

fn get_spo_save_endpoint_start_upload(
    share_point_domain: &String,
    share_point_site: &String,
    path: &String,
    file_name: &String,
    uuid: &String,
) -> String {
    let url = format!(
        r#"https://{share_point_domain}.sharepoint.com/sites/{share_point_site}/_api/web/GetFileByServerRelativeUrl('{path}/{file_name}')/StartUpload(uploadId='{uuid}')"#,
        share_point_domain = share_point_domain,
        share_point_site = share_point_site,
        path = path,
        file_name = file_name,
        uuid = uuid
    );
    url
}

fn get_spo_save_endpoint_continue_upload(
    share_point_domain: &String,
    share_point_site: &String,
    path: &String,
    file_name: &String,
    uuid: &String,
    offset: &u64,
) -> String {
    let url = format!(
        r#"https://{share_point_domain}.sharepoint.com/sites/{share_point_site}/_api/web/GetFileByServerRelativeUrl('{path}/{file_name}')/ContinueUpload(uploadId='{uuid}',fileOffset={offset})"#,
        share_point_domain = share_point_domain,
        share_point_site = share_point_site,
        path = path,
        file_name = file_name,
        uuid = uuid,
        offset = offset
    );
    url
}

fn get_spo_save_endpoint_finish_upload(
    share_point_domain: &String,
    share_point_site: &String,
    path: &String,
    file_name: &String,
    uuid: &String,
    offset: &u64,
) -> String {
    let url = format!(
        r#"https://{share_point_domain}.sharepoint.com/sites/{share_point_site}/_api/web/GetFileByServerRelativeUrl('{path}/{file_name}')/FinishUpload(uploadId='{uuid}',fileOffset={offset})"#,
        share_point_domain = share_point_domain,
        share_point_site = share_point_site,
        path = path,
        file_name = file_name,
        uuid = uuid,
        offset = offset
    );
    url
}

fn get_spo_digest_endpoint(share_point_domain: &String, share_point_site: &String) -> String {
    let url = format!(
        "https://{share_point_domain}.sharepoint.com/sites/{share_point_site}/_api/contextinfo",
        share_point_domain = share_point_domain,
        share_point_site = share_point_site
    );
    url
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let tenant_id = std::env::var("AZURE_TENANT_ID").unwrap();
    let client_id = std::env::var("AZURE_CLIENT_ID").unwrap();
    let client_secret = std::env::var("AZURE_CLIENT_SECRET").unwrap();
    let share_point_domain = std::env::var("SHARE_POINT_DOMAIN").unwrap();

    let account = String::from("nickdevstorage002");
    let container = String::from("datas");
    let blob_name = String::from("test5.txt");

    /////
    let credential = Arc::new(DefaultAzureCredential::default());
    let storage_credentials = StorageCredentials::token_credential(credential);

    let blob_client =
        ClientBuilder::new(account, storage_credentials).blob_client(&container, &blob_name);
    //blob_client.put_block_blob("hello world").content_type("text/plain").await?;


    let mut result: Vec<u8> = vec![];
    // The stream is composed of individual calls to the get blob endpoint
    let mut chunk_buffer_size: u64 = 0;
    let mut offset: u64 = 0;
    let mut has_first_chunk = false;


    //Get SPO Access Token
    let spo_token =
        get_spo_token(&tenant_id, &client_id, &client_secret, &share_point_domain).await?;

    let spo_access_token = spo_token.access_token.unwrap();
    debug!("spo access token: {:?}", spo_access_token);

    //Create new file endpoint
    let spo_save_endpoint = get_spo_save_endpoint(
        &share_point_domain,
        &String::from("MVP"),
        &String::from("/sites/MVP/Shared%20Documents"),
        &blob_name,
    );
    debug!("spo_save_endpoint: {:?}", spo_save_endpoint);

    //Create digest endpoint
    let spo_digest_endpoint = get_spo_digest_endpoint(&share_point_domain, &String::from("MVP"));
    debug!("spo_digest_endpoint: {:?}", spo_digest_endpoint);

    //Get Digest Value
    let digest = get_spo_digest_value(&spo_digest_endpoint, &spo_access_token).await?;
    debug!("digest: {:#?}", digest);
    let uuid = Uuid::new_v4();
    let uuid = uuid.to_string();

    //delete file if exists

    //create new file
    transfer_data_to_spo(&spo_save_endpoint,
                         &digest,
                         &spo_access_token,
                         &String::from("")).await?;


    let mut stream = blob_client.get().into_stream();
    while let Some(value) = stream.next().await {
        let mut body = value?.data;
        // For each response, we stream the body instead of collecting it all
        // into one large allocation.
        while let Some(value) = body.next().await {
            let value = value?;
            //debug!("Value len : {:?}", value.len());
            chunk_buffer_size = chunk_buffer_size + value.len() as u64;

            if chunk_buffer_size < MAX_CHUNK_SIZE as u64 {
                result.extend(&value);
            } else {
                debug!("Next Chunk");
                //upload for previous chunk
                if !has_first_chunk {
                    debug!("Upload First Chunk");
                    let end_point_url = get_spo_save_endpoint_start_upload(
                        &share_point_domain,
                        &String::from("MVP"),
                        &String::from("/sites/MVP/Shared%20Documents"),
                        &blob_name,
                        &uuid);
                    debug!("start upload end point url: {:?}", end_point_url);
                    let res = transfer_data_to_spo(&end_point_url,
                                                   &digest,
                                                   &spo_access_token,
                                                   &String::from_utf8(result.clone()).unwrap()).await;
                    match res {
                        Ok(_) => {
                            has_first_chunk = true;
                            offset = offset + result.len() as u64;

                            debug!("Upload Chunk Success");
                            result = vec![];
                            chunk_buffer_size = value.len() as u64; //reset
                            result.extend(&value);
                        }
                        Err(e) => {
                            debug!("Upload Chunk Error: {:?}", e);
                            panic!("{}", e);
                        }
                    }
                } else {
                    //has first chunk already
                    let end_point_url = get_spo_save_endpoint_continue_upload(
                        &share_point_domain,
                        &String::from("MVP"),
                        &String::from("/sites/MVP/Shared%20Documents"),
                        &blob_name,
                        &uuid,
                        &offset);
                    debug!("continue upload end point url: {:?}", end_point_url);
                    let res = transfer_data_to_spo(&end_point_url,
                                                   &digest,
                                                   &spo_access_token,
                                                   &String::from_utf8(result.clone()).unwrap()).await;
                    match res {
                        Ok(_) => {
                            debug!("Upload Chunk Success");
                            offset = offset + result.len() as u64;

                            result = vec![];
                            chunk_buffer_size = value.len() as u64; //reset
                            result.extend(&value);
                        }
                        Err(e) => {
                            debug!("Upload Chunk Error: {:?}", e);
                            panic!("{}", e);
                        }
                    }
                }
            }
        }
    }
    if result.len() > 0 {
        if !has_first_chunk {
            //simple upload
            //small file
            debug!("Upload First Chunk");
            let end_point_url = get_spo_save_endpoint(
                &share_point_domain,
                &String::from("MVP"),
                &String::from("/sites/MVP/Shared%20Documents"),
                &blob_name);
            debug!("start upload end point url: {:?}", end_point_url);
            transfer_data_to_spo(&end_point_url,
                                 &digest,
                                 &spo_access_token,
                                 &String::from_utf8(result.clone()).unwrap()).await?;
        } else {
            debug!("Upload finish Chunk");
            let end_point_url = get_spo_save_endpoint_finish_upload(
                &share_point_domain,
                &String::from("MVP"),
                &String::from("/sites/MVP/Shared%20Documents"),
                &blob_name,
                &uuid,
                &offset);
            debug!("finish upload end point url: {:?}", end_point_url);
            transfer_data_to_spo(&end_point_url,
                                 &digest,
                                 &spo_access_token,
                                 &String::from_utf8(result.clone()).unwrap()).await?;
        }
    }
    Ok(())
}

async fn transfer_data_to_spo(
    spo_save_endpoint: &String,
    digest: &SPOContextInfoResponse,
    spo_access_token: &String,
    text: &String,
) -> Result<(), reqwest::Error> {
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
    headers.append("Content-Length", text.len().to_string().parse().unwrap());
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
        .body(text.clone())
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
            panic!("{}", e);
        }
    };
    Ok(())
}

async fn get_spo_digest_value(
    spo_digest_endpoint: &String,
    spo_access_token: &String,
) -> Result<SPOContextInfoResponse, reqwest::Error> {
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
