use anyhow::{anyhow, Context, Result};
use lambda_runtime::tracing::info;
use openssl::{
    pkey::Private,
    rsa::{Padding, Rsa},
};

pub async fn get_decryption_key(secret_path: &str) -> Result<Rsa<Private>> {
    let config = aws_config::from_env().load().await;
    let sm = aws_sdk_secretsmanager::Client::new(&config);

    info!("Trying to get {secret_path} from secrets manager");
    let secret = sm
        .get_secret_value()
        .secret_id(secret_path)
        .send()
        .await?
        .secret_string()
        .ok_or(anyhow!("could not get secret binary for {secret_path}"))?
        .to_owned();
    info!("got secret successfully");

    let secret: Rsa<Private> =
        Rsa::private_key_from_pem(secret.as_bytes()).context("could not parse key")?;

    Ok(secret)
}

pub fn decrypt(input: &[u8], key: &Rsa<Private>) -> Result<Vec<u8>> {
    let mut result = vec![0u8; input.len()];
    // try both modes, and only fail if both fail
    let result_size = match key.private_decrypt(input, &mut result, Padding::PKCS1_OAEP) {
        Ok(r) => r,
        Err(_) => key.private_decrypt(input, &mut result, Padding::PKCS1)?,
    };
    let result = result.into_iter().take(result_size).collect();
    Ok(result)
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use openssl::rsa::Rsa;

    use super::decrypt;

    #[test]
    fn test_decryption_with_test_key_new_padding() {
        // encrypted.enc was generated with: echo "test text" | openssl pkeyutl -encrypt -pkeyopt rsa_padding_mode:oaep -pubin -inkey a_pub.pem -in - -out encrypted.enc
        let key = include_str!("./test_keys/a.pem");
        let key = Arc::new(Rsa::private_key_from_pem(key.as_bytes()).unwrap());
        let input = include_bytes!("./test_keys/encrypted.enc").to_vec();
        let output = decrypt(&input, &key).unwrap();
        assert_eq!(output, b"test text\n");
    }

    #[test]
    fn test_decryption_with_test_key_old_padding() {
        // encrypted2.enc was generated with Ruby, default parameters to public_encrypt
        let key = include_str!("./test_keys/a.pem");
        let key = Arc::new(Rsa::private_key_from_pem(key.as_bytes()).unwrap());
        let input = include_bytes!("./test_keys/encrypted2.enc").to_vec();
        let output = decrypt(&input, &key).unwrap();
        assert_eq!(output, b"test text\n");
    }
}
