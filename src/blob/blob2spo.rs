use std::error::Error;
use std::sync::Arc;

use azure_identity::DefaultAzureCredential;
use azure_storage::StorageCredentials;
use azure_storage_blobs::prelude::ClientBuilder;
use futures::StreamExt;
use log::debug;
use spinner::SpinnerHandle;

use crate::spo::spo_engine::SPOEngine;

pub const MAX_CHUNK_SIZE: usize = 32 * 1024 * 1024; // 64MB

pub enum ProcessStatus {
    StartDownload,
    Downloading,
    DownloadComplete,
    StartUpload,
    ContinueUpload,
    FinishUpload,
    UploadComplete,
}

pub type ShowStatusFn =
    fn(status: ProcessStatus, spinner: &SpinnerHandle, message: &String, chunks_size: &u64);

//
//  Read file from azure blob storage and upload chunk file to share point online
//
pub async fn do_copy_file_to_spo(
    tenant_id: &String,
    client_id: &String,
    client_secret: &String,
    share_point_domain: &String,
    share_point_site: &String,
    share_point_pah: &String,
    account: &String,
    container: &String,
    blob_name: &String,
    callback: Option<ShowStatusFn>,
    spinner: Option<&SpinnerHandle>,
) -> Result<(), Box<dyn Error>> {
    let credential = Arc::new(DefaultAzureCredential::default());
    let storage_credentials = StorageCredentials::token_credential(credential);

    let blob_client =
        ClientBuilder::new(account, storage_credentials).blob_client(container, blob_name);

    let mut result: Vec<u8> = vec![];
    // The stream is composed of individual calls to the get blob endpoint
    let mut chunk_buffer_size: u64 = 0;
    let mut offset: u64 = 0;
    let mut has_first_chunk = false;

    let mut spo_engine =
        SPOEngine::new(&tenant_id, &client_id, &client_secret, &share_point_domain);

    if let Some(callback) = callback {
        callback(
            ProcessStatus::StartDownload,
            spinner.unwrap(),
            &String::from("Downloading"),
            &chunk_buffer_size,
        );
    }
    //
    //  Read file from azure blob storage and upload chunk file to share point online
    //
    let mut stream = blob_client.get().into_stream();
    while let Some(value) = stream.next().await {
        let mut body = value?.data;
        // For each response, we stream the body instead of collecting it all
        // into one large allocation.
        while let Some(value) = body.next().await {
            let value = value?;
            //debug!("Value len : {:?}", value.len());
            chunk_buffer_size = chunk_buffer_size + value.len() as u64;
            //
            //  Check chunk buffer size
            //
            if chunk_buffer_size < MAX_CHUNK_SIZE as u64 {
                result.extend(&value);
                //spinner.update(format!("Downloading... {} bytes", chunk_buffer_size));
                if let Some(callback) = callback {
                    callback(
                        ProcessStatus::Downloading,
                        spinner.unwrap(),
                        &String::from("Downloading"),
                        &chunk_buffer_size,
                    );
                }
            } else {
                debug!("Next Chunk");
                //Download completed
                if let Some(callback) = callback {
                    callback(
                        ProcessStatus::DownloadComplete,
                        spinner.unwrap(),
                        &String::from("Download Complete"),
                        &chunk_buffer_size,
                    );
                }
                //upload for previous chunk
                if !has_first_chunk {
                    debug!("Upload First Chunk");
                    //spinner.update(format!("Downloaded... {} bytes", chunk_buffer_size));
                    //callback(ProcessStatus::Start, spinner, &String::from("Upload Start"));
                    if let Some(callback) = callback {
                        callback(
                            ProcessStatus::StartUpload,
                            spinner.unwrap(),
                            &String::from("Upload Start"),
                            &chunk_buffer_size,
                        );
                    }
                    let r = spo_engine
                        .upload_start(
                            share_point_site,
                            share_point_pah,
                            blob_name,
                            result.as_slice(),
                        )
                        .await;
                    match r {
                        Ok(_) => {
                            debug!("Upload Chunk Success");
                            if let Some(callback) = callback {
                                callback(
                                    ProcessStatus::UploadComplete,
                                    spinner.unwrap(),
                                    &String::from("Upload Complete[StartUpload]"),
                                    &chunk_buffer_size,
                                );
                            }
                            //spinner.update(format!("Updated {} bytes", chunk_buffer_size));
                            //setup flag and resetup
                            has_first_chunk = true;
                            offset = offset + result.len() as u64;
                            chunk_buffer_size = value.len() as u64; //reset
                            result = vec![];
                            result.extend(&value);
                        }
                        Err(e) => {
                            debug!("Upload Chunk Error: {:?}", e);
                            panic!("{}", e);
                        }
                    }
                } else {
                    //has first chunk already

                    if let Some(callback) = callback {
                        callback(
                            ProcessStatus::ContinueUpload,
                            spinner.unwrap(),
                            &String::from("Upload Continue"),
                            &chunk_buffer_size,
                        );
                    }

                    let r = spo_engine.upload_continue(result.as_slice(), &offset).await;
                    match r {
                        Ok(_) => {
                            //debug!("continue upload end point url: {:?}", end_point_url);
                            debug!("Upload Chunk Success");
                            if let Some(callback) = callback {
                                callback(
                                    ProcessStatus::UploadComplete,
                                    spinner.unwrap(),
                                    &String::from("Upload Complete[ContinueUpload]"),
                                    &chunk_buffer_size,
                                );
                            }

                            offset = offset + result.len() as u64;
                            chunk_buffer_size = value.len() as u64; //reset
                            result = vec![];
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
            debug!("Upload First Chunk");
            if let Some(callback) = callback {
                callback(
                    ProcessStatus::StartUpload,
                    spinner.unwrap(),
                    &String::from("Upload Start"),
                    &chunk_buffer_size,
                );
            }
            let r = spo_engine
                .upload_one_time(
                    &String::from("MVP"),
                    &String::from("/sites/MVP/Shared Documents"),
                    &String::from(blob_name),
                    result.as_slice(),
                )
                .await;

            match r {
                Ok(_) => {
                    debug!("Upload Chunk Success");
                    if let Some(callback) = callback {
                        callback(
                            ProcessStatus::UploadComplete,
                            spinner.unwrap(),
                            &String::from("Upload Complete"),
                            &chunk_buffer_size,
                        );
                    }
                }
                Err(e) => {
                    debug!("Upload Chunk Error: {:?}", e);
                    panic!("{}", e);
                }
            }
        } else {
            debug!("Upload finish Chunk");

            if let Some(callback) = callback {
                callback(
                    ProcessStatus::DownloadComplete,
                    spinner.unwrap(),
                    &String::from("Download Complete"),
                    &chunk_buffer_size,
                );

                callback(
                    ProcessStatus::FinishUpload,
                    spinner.unwrap(),
                    &String::from("Upload Finish"),
                    &chunk_buffer_size,
                );
            }
            let r = spo_engine.upload_finish(result.as_slice(), &offset).await;
            match r {
                Ok(_) => {
                    debug!("Upload Finish Chunk Success");
                    if let Some(callback) = callback {
                        callback(
                            ProcessStatus::UploadComplete,
                            spinner.unwrap(),
                            &String::from("Upload Complete[FinishUpload]"),
                            &chunk_buffer_size,
                        );
                    }
                }
                Err(e) => {
                    debug!("Upload Chunk Error: {:?}", e);
                    panic!("{}", e);
                }
            }
        }
    }
    Ok(())
}
