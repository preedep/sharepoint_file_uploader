use std::collections::HashMap;
use std::env;
use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};
use warp::{http::Response, Filter};

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

async fn copy_file_blob_to_spo(req: UploadFileToSPORequest) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&req))
}
fn json_body() -> impl Filter<Extract = (UploadFileToSPORequest,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

#[tokio::main]
async fn main() {
    let  blob2spo_endpoint = warp::post()
        .and(warp::path("api"))
        .and(warp::path("HttpTriggerCopyBlob2SPO"))
        .and(warp::path::end())
        .and(json_body())
        .and_then(copy_file_blob_to_spo);

    let port_key = "FUNCTIONS_CUSTOMHANDLER_PORT";
    let port: u16 = match env::var(port_key) {
        Ok(val) => val.parse().expect("Custom Handler port is not a number!"),
        Err(_) => 3000,
    };

    warp::serve(blob2spo_endpoint).run((Ipv4Addr::LOCALHOST, port)).await
}
