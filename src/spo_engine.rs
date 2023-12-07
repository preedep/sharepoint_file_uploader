use std::error::Error;
use std::sync::Arc;

use azure_identity::DefaultAzureCredential;
use azure_storage::StorageCredentials;
use azure_storage_blobs::prelude::ClientBuilder;
use futures::StreamExt;
use log::{debug, info};
use oauth2::http::HeaderMap;
use reqwest::Client;
use spinner::SpinnerHandle;
use uuid::Uuid;

use crate::spo_model::{SPOContextInfoResponse, SPOEndpoint, SPOErrorResponse, SPOTokenResponse};

pub const MAX_CHUNK_SIZE: usize = 64 * 1024 * 1024; // 100MB

pub enum ProcessStatus {
    Start,
    Continue,
    Finish,
}


pub type ShowStatusFn = fn(status: ProcessStatus, spinner: &SpinnerHandle, message: &String);


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

//
// Upload blob name to SPOnline
//
pub async fn do_upload_file_to_spo(
    tenant_id: &String,
    client_id: &String,
    client_secret: &String,
    share_point_domain: &String,
    account: &String,
    container: &String,
    blob_name: &String,
    callback: ShowStatusFn,
    spinner: &SpinnerHandle,
) -> Result<(), Box<dyn Error>> {
    /////
    let credential = Arc::new(DefaultAzureCredential::default());
    let storage_credentials = StorageCredentials::token_credential(credential);

    let blob_client =
        ClientBuilder::new(account, storage_credentials).blob_client(container, blob_name);
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
    let mut endpoint = SPOEndpoint::new(&share_point_domain, &String::from("MVP"))
        .set_path(&String::from("/sites/MVP/Shared%20Documents"))
        .set_file_name(blob_name);

    //Create digest endpoint
    let spo_digest_endpoint = endpoint.to_spo_digest_url();
    //get_spo_digest_endpoint(&share_point_domain, &String::from("MVP"));
    debug!("spo_digest_endpoint: {:?}", spo_digest_endpoint);
    //Get Digest Value
    let digest = get_spo_digest_value(&spo_digest_endpoint, &spo_access_token).await?;
    debug!("digest: {:#?}", digest);
    let uuid = Uuid::new_v4();
    let uuid = uuid.to_string();
    //delete file if exists

    //create new file
    transfer_data_to_spo(
        &endpoint.to_spo_one_time_save_endpoint(),
        &digest,
        &spo_access_token,
        &String::from(""),
    )
        .await?;

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
                spinner.update(format!("Downloading... {} bytes", chunk_buffer_size));
            } else {
                debug!("Next Chunk");
                //upload for previous chunk
                if !has_first_chunk {
                    debug!("Upload First Chunk");
                    //spinner.update(format!("Downloaded... {} bytes", chunk_buffer_size));
                    callback(ProcessStatus::Start, spinner, &String::from("Upload Start"));

                    let end_point_url = endpoint.set_uuid(&uuid).to_spo_start_upload_endpoint();
                    //debug!("start upload end point url: {:?}", end_point_url);
                    let res = transfer_data_to_spo(
                        &end_point_url,
                        &digest,
                        &spo_access_token,
                        &String::from_utf8(result.clone()).unwrap(),
                    )
                        .await;
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
                    //spinner.update(format!("Downloaded... {} bytes", chunk_buffer_size));
                    let end_point_url = endpoint
                        .set_uuid(&uuid)
                        .set_offset(&offset)
                        .to_spo_start_continue_endpoint();
                    callback(
                        ProcessStatus::Continue,
                        &spinner,
                        &String::from("Upload Continue"),
                    );
                    //debug!("continue upload end point url: {:?}", end_point_url);

                    let res = transfer_data_to_spo(
                        &end_point_url,
                        &digest,
                        &spo_access_token,
                        &String::from_utf8(result.clone()).unwrap(),
                    )
                        .await;
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
            callback(
                ProcessStatus::Start,
                &spinner,
                &String::from("Upload Start"),
            );
            debug!("Upload First Chunk");
            let end_point_url = endpoint.set_uuid(&uuid).to_spo_one_time_save_endpoint();
            //debug!("start upload end point url: {:?}", end_point_url);
            transfer_data_to_spo(
                &end_point_url,
                &digest,
                &spo_access_token,
                &String::from_utf8(result.clone()).unwrap(),
            )
                .await?;
            callback(
                ProcessStatus::Finish,
                &spinner,
                &String::from("Upload Finish"),
            );


        } else {
            debug!("Upload finish Chunk");
            //spinner.update(format!("Downloaded... {} bytes", chunk_buffer_size));
            callback(
                ProcessStatus::Finish,
                &spinner,
                &String::from("Upload Finish"),
            );
            let end_point_url = endpoint
                .set_uuid(&uuid)
                .set_offset(&offset)
                .to_spo_start_finish_endpoint();
            //debug!("finish upload end point url: {:?}", end_point_url);
            transfer_data_to_spo(
                &end_point_url,
                &digest,
                &spo_access_token,
                &String::from_utf8(result.clone()).unwrap(),
            )
                .await?;
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
