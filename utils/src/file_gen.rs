use anyhow::Result;
use aws_lc_rs::rsa::{OaepPublicEncryptingKey, PrivateDecryptingKey, OAEP_SHA1_MGF1SHA1};
use rand::prelude::*;

/// # Errors
/// this will fail whenever the encryption fails. Example cases might be different padding/mode.
pub fn encrypt_file(data: &[u8], private_key: &[u8]) -> Result<Vec<u8>> {
    let private_key = PrivateDecryptingKey::from_pkcs8(private_key)?;
    let public_key = OaepPublicEncryptingKey::new(private_key.public_key())?;

    let mut ciphertext = vec![0u8; public_key.ciphertext_size()];

    let max_plaintext_size = public_key.max_plaintext_size(&OAEP_SHA1_MGF1SHA1);

    let data = &data[0..data.len().min(max_plaintext_size)];

    public_key.encrypt(&OAEP_SHA1_MGF1SHA1, data, &mut ciphertext, None)?;

    Ok(ciphertext)
}

#[must_use]
pub fn gen_sample(length: Option<usize>) -> Vec<u8> {
    let mut sample: Vec<u8> = Vec::with_capacity(length.unwrap_or(1024));
    for _ in 0..sample.capacity() {
        sample.push(random::<char>() as u8);
    }
    sample
}
