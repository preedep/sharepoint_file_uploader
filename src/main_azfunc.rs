use std::collections::HashMap;
use std::env;
use std::net::Ipv4Addr;

use crate::blob::blob2spo::do_copy_file_to_spo;
use serde::{Deserialize, Serialize};
use warp::{ Filter };
use warp::reject::Reject;
use crate::spo::spo_engine::SPOError;

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

#[derive(Debug,Clone,Serialize,Deserialize)]
struct UploadFileToSPOReject {
    error: SPOError
}
impl Reject for UploadFileToSPOReject {}
impl UploadFileToSPOReject {
    fn new(error: SPOError) -> Self {
        UploadFileToSPOReject {
            error
        }
    }
}

async fn copy_file_blob_to_spo(
    req: UploadFileToSPORequest,
) -> Result<impl warp::Reply, warp::Rejection> {
    do_copy_file_to_spo(
        &req.tenant_id,
        &req.client_id,
        &req.client_secret,
        &req.share_point_domain,
        &req.share_point_site,
        &req.share_point_path,
        &req.account,
        &req.container,
        &req.blob_name,
        None,
        None,
    )
    .await.map(|r|{
        warp::reply::json(&r)
    }).map_err(|e|{
        warp::reject::custom(crate::UploadFileToSPOReject::new(e))
    })
}
fn json_body() -> impl Filter<Extract = (UploadFileToSPORequest,), Error = warp::Rejection> + Clone
{
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
async fn recover(err: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(e) = err.find::<UploadFileToSPOReject>() {
        let json = warp::reply::json(&e);
        Ok(warp::reply::with_status(json, warp::http::StatusCode::INTERNAL_SERVER_ERROR))
    } else {
        Err(warp::reject::not_found())
    }
}
#[tokio::main]
async fn main() {
    let blob2spo_endpoint = warp::post()
        .and(warp::path("api"))
        .and(warp::path("HttpTriggerCopyBlob2SPO"))
        .and(warp::path::end())
        .and(json_body())
        .and_then(copy_file_blob_to_spo)
        .recover(recover);

    let port_key = "FUNCTIONS_CUSTOMHANDLER_PORT";
    let port: u16 = match env::var(port_key) {
        Ok(val) => val.parse().expect("Custom Handler port is not a number!"),
        Err(_) => 3000,
    };

    warp::serve(blob2spo_endpoint)
        .run((Ipv4Addr::LOCALHOST, port))
        .await
}
