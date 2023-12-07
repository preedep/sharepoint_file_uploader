use std::error::Error;


use console::Style;
use log::info;
use spinner::{SpinnerBuilder, SpinnerHandle};

use crate::spo_engine::{do_upload_file_to_spo, ProcessStatus};

mod entra_id_model;
mod spo_model;
mod spo_engine;


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

    let sp = SpinnerBuilder::new("Uploading....".into()).start();

    do_upload_file_to_spo(
        &tenant_id,
        &client_id,
        &client_secret,
        &share_point_domain,
        &account,
        &container,
        &blob_name,
        show_status,
        &sp,
    )
        .await?;
    Ok(())
}
