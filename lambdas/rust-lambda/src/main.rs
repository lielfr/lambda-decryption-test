use std::sync::Arc;

use anyhow::{anyhow, Result};
use aws_lambda_events::{event::s3::S3Event, s3::S3Entity};
use aws_sdk_s3::primitives::ByteStream;
use lambda_runtime::{
    run, service_fn,
    tracing::{self, info},
    Error, LambdaEvent,
};
use tokio::task::spawn_blocking;
mod decryptor;

const PRIVATE_KEY_PATH_ENV: &str = "PRIVATE_KEY_PATH";
const RESULT_BUCKET_PATH_ENV: &str = "RESULT_BUCKET_PATH";

async fn function_handler(event: LambdaEvent<S3Event>) -> Result<()> {
    let private_key_path = std::env::var(PRIVATE_KEY_PATH_ENV)?;
    let private_key = Arc::new(decryptor::get_decryption_key(private_key_path.as_str()).await?);
    let result_bucket_path = std::env::var(RESULT_BUCKET_PATH_ENV)?;
    let config = aws_config::from_env().load().await;
    let s3_client = aws_sdk_s3::Client::new(&config);

    for r in event.payload.records {
        let S3Entity { bucket, object, .. } = r.s3;
        info!("Got bucket: {bucket:?}, object: {object:?}, processing");
        let object_key = object.key.clone().ok_or(anyhow!("no key for {object:?}"))?;
        let result = s3_client
            .get_object()
            .bucket(
                bucket
                    .name
                    .clone()
                    .ok_or(anyhow!("no bucket name for {:?}", bucket.clone()))?,
            )
            .key(&object_key)
            .send()
            .await?;

        let result = result.body.collect().await?.to_vec();
        let inner_private_key = Arc::clone(&private_key);
        let result =
            spawn_blocking(move || decryptor::decrypt(&result, &inner_private_key)).await??;

        info!("performed decryption successfully, writing to target bucket");
        s3_client
            .put_object()
            .body(ByteStream::from(result))
            .bucket(&result_bucket_path)
            .key(&object_key)
            .send()
            .await?;
        info!("finished for {bucket:?}, object: {object:?}");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
