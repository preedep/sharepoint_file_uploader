use std::error::Error;
use std::time::SystemTime;

use clap::Parser;
use console::Style;
use futures::StreamExt;
use log::info;
use spinner::{SpinnerBuilder, SpinnerHandle};

use crate::blob::blob2spo::{do_copy_file_to_spo, ProcessStatus};

mod blob;
mod spo;

fn show_status(
    status: ProcessStatus,
    spinner: &SpinnerHandle,
    message: &String,
    chunks_size: &u64,
) {
    let cyan = Style::new().cyan().bold();

    match status {
        ProcessStatus::StartDownload => {
            //info!("Start download blob file [{}]", cyan.apply_to(message));
            spinner.message(message.clone().into());
            //spinner.update("".to_string());
        }
        ProcessStatus::Downloading => {
            //info!("Start copy file to share point online [{}]", cyan.apply_to(message));
            let message = format!("{} with {} bytes", message, chunks_size);
            spinner.update(cyan.apply_to(message).to_string());
        }
        ProcessStatus::DownloadComplete => {
            //info!("Download complete [{}]", cyan.apply_to(message));
            spinner.message(message.clone().into());
        }
        ProcessStatus::StartUpload => {
            //info!("Start upload file to share point online [{}]", cyan.apply_to(message));
            spinner.message(message.clone().into());
        }
        ProcessStatus::ContinueUpload => {
            //info!("Continue upload file to share point online [{}]", cyan.apply_to(message));
            spinner.message(message.clone().into());
        }
        ProcessStatus::FinishUpload => {
            //info!("Finish upload file to share point online [{}]", cyan.apply_to(message));
            spinner.message(message.clone().into());
        }
        ProcessStatus::UploadComplete => {
            //info!("Upload done [{}]", cyan.apply_to(message));
            //let message = format!("{} with {} bytes", message, chunks_size);
            spinner.message(message.clone().into());
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

    let sp = SpinnerBuilder::new("Copy file to SPO".into()).start();
    let start = SystemTime::now();

    let res = do_copy_file_to_spo(
        &tenant_id,
        &client_id,
        &client_secret,
        &share_point_domain,
        &share_point_site,
        &share_point_path,
        &account,
        &container,
        &blob_name,
        Some(show_status),
        Some(&sp),
    )
    .await;
    match res {
        Ok(_) => {
            info!("Copy file to SPO complete");
        }
        Err(e) => {
            info!("Copy file to SPO error : {}", e);
        }
    }

    let diff = SystemTime::now().duration_since(start).unwrap();
    info!("Executed complete : {:?} secs", diff.as_secs());

    Ok(())
}
