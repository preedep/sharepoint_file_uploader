use std::error::Error;
use std::sync::Arc;
use std::time::SystemTime;

use azure_identity::DefaultAzureCredential;
use azure_storage::StorageCredentials;
use azure_storage_blobs::prelude::ClientBuilder;
use clap::Parser;
use console::Style;
use futures::StreamExt;
use log::{debug, info};
use spinner::{SpinnerBuilder, SpinnerHandle};

use crate::spo::spo_engine::SPOEngine;

pub const MAX_CHUNK_SIZE: usize = 64 * 1024 * 1024; // 64MB

mod main_azfunc;
mod spo;

pub enum ProcessStatus {
    Start,
    Continue,
    Finish,
}

pub type ShowStatusFn = fn(status: ProcessStatus, spinner: &SpinnerHandle, message: &String);

fn show_status(status: ProcessStatus, spinner: &SpinnerHandle, message: &String) {
    let cyan = Style::new().cyan().bold();

    match status {
        ProcessStatus::Start => {
            info!("Start[{}]", cyan.apply_to(message));
            spinner.update(message.clone().into());
        }
        ProcessStatus::Continue => {
            info!("Continue[{}]", cyan.apply_to(message));
            spinner.update(message.clone().into());
        }
        ProcessStatus::Finish => {
            info!("Finish[{}]", cyan.apply_to(message));
            spinner.update(message.clone().into());
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Storage account for copy file to share point online
    #[arg(long)]
    storage_account: String,
    /// Container name in storage account for copy file to share point online
    #[arg(long)]
    container_name: String,
    /// Blob name or File name for copy file to share point online
    #[arg(long)]
    blob_name: String,
    /// Share point domain ex. [share_point_domain].sharepoint.com
    #[arg(long)]
    spo_domain: String,
    /// Share point domain ex. [share_point_domain].sharepoint.com/sites/[share_point_site]
    #[arg(long)]
    spo_site: String,
    /// Share point domain ex. [share_point_domain].sharepoint.com/sites/[share_point_site]/_api/web/GetFileByServerRelativeUrl('[spo_path]')
    #[arg(long)]
    spo_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let cli = Cli::parse();

    // Common parameters for uses authentication for Storage Account , Share Point Online
    // Client Secret got from App Registration in Azure Active Directory
    let tenant_id = std::env::var("AZURE_TENANT_ID").unwrap();
    let client_id = std::env::var("AZURE_CLIENT_ID").unwrap();
    let client_secret = std::env::var("AZURE_CLIENT_SECRET").unwrap();

    // Parameters for blob storage
    let account = cli.storage_account;
    let container = cli.container_name;
    let blob_name = cli.blob_name;

    // Parameters for share point online
    let share_point_domain = cli.spo_domain;
    let share_point_site = cli.spo_site;
    let share_point_path = cli.spo_path;

    let sp = SpinnerBuilder::new("Uploading....".into()).start();
    let start = SystemTime::now();

    do_upload_file_to_spo(
        &tenant_id,
        &client_id,
        &client_secret,
        &share_point_domain,
        &share_point_site,
        &share_point_path,
        &account,
        &container,
        &blob_name,
        show_status,
        &sp,
    )
    .await?;

    let diff = SystemTime::now().duration_since(start).unwrap();
    info!("Executed complete : {:?} secs", diff.as_secs());

    Ok(())
}

//
//  Read file from azure blob storage and upload chunk file to share point online
//
async fn do_upload_file_to_spo(
    tenant_id: &String,
    client_id: &String,
    client_secret: &String,
    share_point_domain: &String,
    share_point_site: &String,
    share_point_pah: &String,
    account: &String,
    container: &String,
    blob_name: &String,
    callback: ShowStatusFn,
    spinner: &SpinnerHandle,
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
                spinner.update(format!("Downloading... {} bytes", chunk_buffer_size));
            } else {
                debug!("Next Chunk");
                //upload for previous chunk
                if !has_first_chunk {
                    debug!("Upload First Chunk");
                    //spinner.update(format!("Downloaded... {} bytes", chunk_buffer_size));
                    callback(ProcessStatus::Start, spinner, &String::from("Upload Start"));
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
                            spinner.update(format!("Updated {} bytes", chunk_buffer_size));
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
                    //spinner.update(format!("Downloaded... {} bytes", chunk_buffer_size));
                    callback(
                        ProcessStatus::Continue,
                        &spinner,
                        &String::from("Upload Continue"),
                    );

                    let r = spo_engine.upload_continue(result.as_slice(), &offset).await;
                    match r {
                        Ok(_) => {
                            //debug!("continue upload end point url: {:?}", end_point_url);
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
                }
                Err(e) => {
                    debug!("Upload Chunk Error: {:?}", e);
                    panic!("{}", e);
                }
            }
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
            let r = spo_engine.upload_finish(result.as_slice(), &offset).await;
            match r {
                Ok(_) => {
                    debug!("Upload Chunk Success");
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
