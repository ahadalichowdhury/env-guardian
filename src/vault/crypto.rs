use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::Argon2;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::rngs::OsRng;
use rand::RngCore;

use crate::error::{AppError, Result};

const FORMAT_HEADER: &str = "ENVGUARDIAN1";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

/// Encrypt plaintext with master password; returns serialized .env.enc contents.
pub fn encrypt_bytes(plaintext: &[u8], password: &str) -> Result<String> {
    let mut salt_bytes = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt_bytes);

    let key = derive_key(password, &salt_bytes)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AppError::Crypto(format!("invalid key: {}", e)))?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| AppError::Crypto(format!("encryption failed: {}", e)))?;

    let salt_b64 = BASE64.encode(salt_bytes);
    let nonce_b64 = BASE64.encode(nonce_bytes);
    let data_b64 = BASE64.encode(ciphertext);

    Ok(format!(
        "{}\nsalt={}\nnonce={}\ndata={}",
        FORMAT_HEADER,
        salt_b64,
        nonce_b64,
        data_b64
    ))
}

/// Decrypt .env.enc file contents with master password.
pub fn decrypt_bytes(encoded: &str, password: &str) -> Result<Vec<u8>> {
    let (salt, nonce_bytes, ciphertext) = parse_encrypted_file(encoded)?;

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AppError::Crypto(format!("invalid key: {}", e)))?;

    let nonce = Nonce::from_slice(&nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| AppError::Crypto(
            "decryption failed — wrong password or corrupted file".to_string(),
        ))
}

fn derive_key(password: &str, salt_bytes: &[u8]) -> Result<[u8; KEY_LEN]> {
    let argon2 = Argon2::default();
    let mut key = [0u8; KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), salt_bytes, &mut key)
        .map_err(|e| AppError::Crypto(format!("key derivation failed: {}", e)))?;
    Ok(key)
}

fn parse_encrypted_file(contents: &str) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let lines: Vec<&str> = contents.lines().collect();
    if lines.is_empty() || lines[0].trim() != FORMAT_HEADER {
        return Err(AppError::Crypto(
            "invalid encrypted file format — expected ENVGUARDIAN1 header".to_string(),
        ));
    }

    let mut salt_b64 = None;
    let mut nonce_b64 = None;
    let mut data_b64 = None;

    for line in lines.iter().skip(1) {
        let line = line.trim();
        if let Some((k, v)) = line.split_once('=') {
            match k.trim() {
                "salt" => salt_b64 = Some(v.trim()),
                "nonce" => nonce_b64 = Some(v.trim()),
                "data" => data_b64 = Some(v.trim()),
                _ => {}
            }
        }
    }

    let salt = BASE64
        .decode(salt_b64.unwrap_or(""))
        .map_err(|e| AppError::Crypto(format!("invalid salt: {}", e)))?;
    let nonce = BASE64
        .decode(nonce_b64.unwrap_or(""))
        .map_err(|e| AppError::Crypto(format!("invalid nonce: {}", e)))?;
    let data = BASE64
        .decode(data_b64.unwrap_or(""))
        .map_err(|e| AppError::Crypto(format!("invalid data: {}", e)))?;

    Ok((salt, nonce, data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_encrypt_decrypt() {
        let plain = b"DATABASE_URL=postgres://localhost\nAPI_KEY=secret";
        let enc = encrypt_bytes(plain, "master-password-123").unwrap();
        let dec = decrypt_bytes(&enc, "master-password-123").unwrap();
        assert_eq!(dec, plain);
    }

    #[test]
    fn wrong_password_fails() {
        let enc = encrypt_bytes(b"KEY=value", "correct").unwrap();
        let err = decrypt_bytes(&enc, "wrong").unwrap_err();
        assert!(err.to_string().contains("decryption failed"));
    }
}
