mod error;

pub use self::error::{Error, Result};

use hmac::{Hmac, Mac};
use sha2::Sha512;

pub struct EncryptContent {
    pub content: String,
    pub salt: String,
}

/// Not used for creating encrypted / hashed content due to b64
///
/// This is only meant to be used for passing the content around using a safe character set,
/// and provides more versatility
pub fn encrypt_into_b64u(key: &[u8], enc_content: &EncryptContent) -> Result<String> {
    let EncryptContent { content, salt } = enc_content;
    let mut hmac_sha512 = Hmac::<Sha512>::new_from_slice(key).map_err(|_| Error::KeyFailHmac)?;
    hmac_sha512.update(content.as_bytes());
    hmac_sha512.update(salt.as_bytes());

    let hmac_result = hmac_sha512.finalize();
    let result_bytes = hmac_result.into_bytes();
    let result = base64_url::encode(&result_bytes);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use rand::RngCore;

    #[test]
    fn test_encrypt_into_b64u_ok() -> Result<()> {
        let mut t_key = [0u8; 64]; // Alloc 64 bytes of memory; 512 bits = 64 bytes
        rand::thread_rng().fill_bytes(&mut t_key);
        let t_enc_content = EncryptContent {
            content: "hello world".to_string(),
            salt: "pepper".to_string(),
        };

        // TODO: For test consistency, the key should be static
        let t_res = encrypt_into_b64u(&t_key, &t_enc_content)?;
        let res = encrypt_into_b64u(&t_key, &t_enc_content)?;
        println!("{res}");

        assert_eq!(res, t_res);

        Ok(())
    }
}
