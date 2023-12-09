use std::collections::HashMap;
use std::env;
use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};
use warp::{Filter, http::Response};

mod blob;
mod spo;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UploadFileToSPORequest {
    tenant_id: String,
    client_id: String,
    client_secret: String,
    share_point_domain: String,
    share_point_site: String,
    share_point_path: String,
    account: String,
    container: String,
    blob_name: String,
}


#[tokio::main]
async fn main() {
    let example1 = warp::get()
        .and(warp::path("api"))
        .and(warp::path("HttpTriggerCopyBlob2SPO"))
        .and(warp::query::<HashMap<String, String>>())
        .map(|p: HashMap<String, String>| match p.get("name") {
            Some(name) => Response::builder().body(format!("Hello, {}. This HTTP triggered function executed successfully.", name)),
            None => Response::builder().body(String::from("This HTTP triggered function executed successfully. Pass a name in the query string for a personalized response.")),
        });

    let port_key = "FUNCTIONS_CUSTOMHANDLER_PORT";
    let port: u16 = match env::var(port_key) {
        Ok(val) => val.parse().expect("Custom Handler port is not a number!"),
        Err(_) => 3000,
    };

    warp::serve(example1).run((Ipv4Addr::LOCALHOST, port)).await
}
