use anyhow::{anyhow, bail, Context, Result};
use base64::Engine;
use clap::Parser;
use utils::file_gen::{encrypt_file, gen_sample};

async fn upload_encrypted_to_s3(file: &[u8], bucket: &str, key: &str) -> Result<()> {
    let config = aws_config::from_env().load().await;
    let client = aws_sdk_s3::Client::new(&config);

    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(file.to_owned().into())
        .send()
        .await?;

    Ok(())
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CliArgs {
    /// Num of samples to generate
    num_samples: usize,
    /// path to private key
    private_key: String,

    /// target bucket(s), at least one is required
    target_buckets: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();

    if args.target_buckets.is_empty() {
        bail!("you must specify at least one target bucket")
    }

    let private_key: Vec<_> = tokio::fs::read(args.private_key)
        .await?
        .strip_prefix(b"-----BEGIN PRIVATE KEY-----")
        .ok_or(anyhow!(
            "private key must begin with -----BEGIN PRIVATE KEY-----"
        ))?
        .strip_suffix(b"-----END PRIVATE KEY-----\n")
        .ok_or(anyhow!(
            "private key must end with -----END PRIVATE KEY-----"
        ))?
        .iter()
        .filter(|ch| **ch != b'\n')
        .copied()
        .collect();

    let private_key = base64::prelude::BASE64_STANDARD.decode(private_key)?;

    let mut sample_length = 10usize;

    for sample_num in 0..args.num_samples {
        println!("Generating sample #{sample_num}, sample length = {sample_length}");
        let encrypted_file_name = format!("sample_{sample_num}_encrypted");
        let raw_file_name = format!("sample_{sample_num}_raw");

        let sample = gen_sample(Some(sample_length));
        tokio::fs::write(&raw_file_name, sample.as_slice()).await?;

        println!("encrypting and uploading to s3");

        let encrypted_sample = encrypt_file(sample.as_slice(), private_key.as_slice())?;
        tokio::fs::write(&encrypted_file_name, encrypted_sample.as_slice()).await?;

        for bucket in &args.target_buckets {
            println!("uploading sample to {bucket}");
            upload_encrypted_to_s3(
                encrypted_sample.as_slice(),
                bucket.as_str(),
                &format!("sample_{sample_num}"),
            )
            .await?;
        }

        tokio::fs::remove_file(&raw_file_name)
            .await
            .context("could not delete raw file")?;
        tokio::fs::remove_file(&encrypted_file_name)
            .await
            .context("could not delete encrypted file")?;

        sample_length *= 2;
        sample_length = sample_length.min(1024 * 1024 * 1024);
    }

    Ok(())
}
