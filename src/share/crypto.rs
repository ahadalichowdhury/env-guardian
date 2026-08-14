use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::Sha256;
use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};

use crate::error::{AppError, Result};

const PK_HEADER: &str = "ENVGUARDIAN_PK1";
const SK_HEADER: &str = "ENVGUARDIAN_SK1";
const SHARE_HEADER: &str = "ENVGUARDIAN_SHARE1";
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

pub struct KeyPair {
    pub public: String,
    pub private: String,
}

pub fn generate_keypair() -> Result<KeyPair> {
    let secret = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);

    let public_file = format!(
        "{}\n{}",
        PK_HEADER,
        BASE64.encode(public.as_bytes())
    );
    let private_file = format!(
        "{}\n{}",
        SK_HEADER,
        BASE64.encode(secret.as_bytes())
    );

    Ok(KeyPair {
        public: public_file,
        private: private_file,
    })
}

pub fn load_public_key(contents: &str) -> Result<PublicKey> {
    let lines: Vec<&str> = contents.lines().collect();
    if lines.first().map(|l| l.trim()) != Some(PK_HEADER) {
        return Err(AppError::Crypto("invalid public key file".to_string()));
    }
    let bytes = BASE64
        .decode(lines.get(1).unwrap_or(&""))
        .map_err(|e| AppError::Crypto(format!("public key decode: {}", e)))?;
    if bytes.len() != 32 {
        return Err(AppError::Crypto("public key must be 32 bytes".to_string()));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(PublicKey::from(arr))
}

pub fn load_private_key(contents: &str) -> Result<StaticSecret> {
    let lines: Vec<&str> = contents.lines().collect();
    if lines.first().map(|l| l.trim()) != Some(SK_HEADER) {
        return Err(AppError::Crypto("invalid private key file".to_string()));
    }
    let bytes = BASE64
        .decode(lines.get(1).unwrap_or(&""))
        .map_err(|e| AppError::Crypto(format!("private key decode: {}", e)))?;
    if bytes.len() != 32 {
        return Err(AppError::Crypto("private key must be 32 bytes".to_string()));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(StaticSecret::from(arr))
}

/// Create E2E encrypted share package for recipient.
pub fn create_share(plaintext: &[u8], recipient_public: &PublicKey, label: &str) -> Result<String> {
    let ephemeral = EphemeralSecret::random_from_rng(OsRng);
    let ephemeral_public = PublicKey::from(&ephemeral);

    let shared = ephemeral.diffie_hellman(recipient_public);
    let wrap_key = derive_key(shared.as_bytes(), b"env-guardian-wrap")?;

    let mut data_key = [0u8; KEY_LEN];
    OsRng.fill_bytes(&mut data_key);

    let (data_nonce, data_ct) = encrypt_with_key(plaintext, &data_key)?;
    let (wrap_nonce, wrapped_key) = encrypt_with_key(&data_key, &wrap_key)?;

    Ok(format!(
        "{}\nephemeral_pub={}\nwrap_nonce={}\nwrapped_key={}\ndata_nonce={}\ndata={}\nlabel={}",
        SHARE_HEADER,
        BASE64.encode(ephemeral_public.as_bytes()),
        BASE64.encode(wrap_nonce),
        BASE64.encode(wrapped_key),
        BASE64.encode(data_nonce),
        BASE64.encode(data_ct),
        label
    ))
}

/// Decrypt share package with recipient private key.
pub fn open_share(encoded: &str, recipient_secret: &StaticSecret) -> Result<Vec<u8>> {
    let (ephemeral_pub_b64, wrap_nonce_b64, wrapped_key_b64, data_nonce_b64, data_b64) =
        parse_share(encoded)?;

    let ephemeral_bytes = BASE64
        .decode(ephemeral_pub_b64)
        .map_err(|e| AppError::Crypto(format!("ephemeral pub: {}", e)))?;
    if ephemeral_bytes.len() != 32 {
        return Err(AppError::Crypto("invalid ephemeral public key".to_string()));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&ephemeral_bytes);
    let ephemeral_public = PublicKey::from(arr);

    let shared = recipient_secret.diffie_hellman(&ephemeral_public);
    let wrap_key = derive_key(shared.as_bytes(), b"env-guardian-wrap")?;

    let wrap_nonce = BASE64
        .decode(wrap_nonce_b64)
        .map_err(|e| AppError::Crypto(format!("wrap nonce: {}", e)))?;
    let wrapped_key = BASE64
        .decode(wrapped_key_b64)
        .map_err(|e| AppError::Crypto(format!("wrapped key: {}", e)))?;

    let data_key_vec = decrypt_with_key(&wrapped_key, &wrap_nonce, &wrap_key)?;

    let data_nonce = BASE64
        .decode(data_nonce_b64)
        .map_err(|e| AppError::Crypto(format!("data nonce: {}", e)))?;
    let data_ct = BASE64
        .decode(data_b64)
        .map_err(|e| AppError::Crypto(format!("data: {}", e)))?;

    let data_key = vec_to_key32(&data_key_vec)?;
    decrypt_with_key(&data_ct, &data_nonce, &data_key)
}

fn vec_to_key32(v: &[u8]) -> Result<[u8; KEY_LEN]> {
    if v.len() != KEY_LEN {
        return Err(AppError::Crypto("invalid key length".to_string()));
    }
    let mut arr = [0u8; KEY_LEN];
    arr.copy_from_slice(v);
    Ok(arr)
}

fn derive_key(shared: &[u8], info: &[u8]) -> Result<[u8; KEY_LEN]> {
    let hk = Hkdf::<Sha256>::new(None, shared);
    let mut okm = [0u8; KEY_LEN];
    hk.expand(info, &mut okm)
        .map_err(|e| AppError::Crypto(format!("hkdf failed: {}", e)))?;
    Ok(okm)
}

fn encrypt_with_key(plaintext: &[u8], key: &[u8; KEY_LEN]) -> Result<(Vec<u8>, Vec<u8>)> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| AppError::Crypto(format!("cipher init: {}", e)))?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| AppError::Crypto(format!("encrypt failed: {}", e)))?;
    Ok((nonce_bytes.to_vec(), ct))
}

fn decrypt_with_key(
    ciphertext: &[u8],
    nonce_bytes: &[u8],
    key: &[u8; KEY_LEN],
) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| AppError::Crypto(format!("cipher init: {}", e)))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| AppError::Crypto("decryption failed — wrong key or corrupted share".to_string()))
}

fn parse_share(contents: &str) -> Result<(String, String, String, String, String)> {
    let lines: Vec<&str> = contents.lines().collect();
    if lines.first().map(|l| l.trim()) != Some(SHARE_HEADER) {
        return Err(AppError::Crypto(
            "invalid share format — expected ENVGUARDIAN_SHARE1".to_string(),
        ));
    }

    let mut ephemeral_pub = String::new();
    let mut wrap_nonce = String::new();
    let mut wrapped_key = String::new();
    let mut data_nonce = String::new();
    let mut data = String::new();

    for line in lines.iter().skip(1) {
        if let Some((k, v)) = line.split_once('=') {
            match k.trim() {
                "ephemeral_pub" => ephemeral_pub = v.trim().to_string(),
                "wrap_nonce" => wrap_nonce = v.trim().to_string(),
                "wrapped_key" => wrapped_key = v.trim().to_string(),
                "data_nonce" => data_nonce = v.trim().to_string(),
                "data" => data = v.trim().to_string(),
                _ => {}
            }
        }
    }

    if ephemeral_pub.is_empty() || data.is_empty() {
        return Err(AppError::Crypto("incomplete share package".to_string()));
    }

    Ok((ephemeral_pub, wrap_nonce, wrapped_key, data_nonce, data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn share_roundtrip() {
        let pair = generate_keypair().unwrap();
        let public = load_public_key(&pair.public).unwrap();
        let private = load_private_key(&pair.private).unwrap();

        let plain = b"DATABASE_URL=postgres://prod\nAPI_KEY=secret";
        let share = create_share(plain, &public, "production").unwrap();
        let opened = open_share(&share, &private).unwrap();
        assert_eq!(opened, plain);
    }

    #[test]
    fn wrong_key_fails() {
        let pair1 = generate_keypair().unwrap();
        let pair2 = generate_keypair().unwrap();
        let pub1 = load_public_key(&pair1.public).unwrap();
        let priv2 = load_private_key(&pair2.private).unwrap();

        let share = create_share(b"KEY=v", &pub1, "test").unwrap();
        let err = open_share(&share, &priv2).unwrap_err();
        assert!(err.to_string().contains("decryption failed"));
    }
}
