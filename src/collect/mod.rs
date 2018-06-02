use failure;
use futures;
use rusoto_core;
use rusoto_s3;
use serde_json;

use futures::prelude::*;

pub mod schema;

#[async]
pub fn upload_states_to_s3(
    states: futures::sync::mpsc::Receiver<schema::State>,
) -> Result<(), failure::Error> {
    use rusoto_s3::S3;

    let s3_client = rusoto_s3::S3Client::simple(rusoto_core::region::Region::EuWest1);

    #[async]
    for state in states.map_err(|_| failure::err_msg("state channel poisoned")) {
        let state_json = serde_json::to_vec(&state)?;

        if let Err(e) = await!(s3_client.put_object(&rusoto_s3::PutObjectRequest {
            body: Some(state_json),
            bucket: "precip-stats".to_owned(),
            key: "data.json".to_owned(),
            content_type: Some("application/json".to_owned()),
            cache_control: Some("max-age=300".to_owned()),
            acl: Some("public-read".to_owned()),
            ..rusoto_s3::PutObjectRequest::default()
        })) {
            warn!("upload to S3 failed, will retry later: {}", e);
        } else {
            info!("uploaded state to S3");
        }
    }

    Ok(())
}
