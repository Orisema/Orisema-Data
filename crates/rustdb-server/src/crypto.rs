use crate::error::AppError;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng, Payload},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use uuid::Uuid;

pub struct Credentials(Aes256Gcm);
impl Credentials {
    pub fn new(key: &str) -> Result<Self, AppError> {
        let key = STANDARD
            .decode(key)
            .map_err(|_| AppError::Validation("Invalid encryption key"))?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|_| AppError::Validation("Encryption key must contain exactly 32 bytes"))?;
        Ok(Self(cipher))
    }
    pub fn encrypt(&self, id: Uuid, password: &str) -> Result<String, AppError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .0
            .encrypt(
                &nonce,
                Payload {
                    msg: password.as_bytes(),
                    aad: id.as_bytes(),
                },
            )
            .map_err(|_| AppError::Internal)?;
        let mut bytes = nonce.to_vec();
        bytes.extend(ciphertext);
        Ok(format!("v1:{}", STANDARD.encode(bytes)))
    }
    pub fn decrypt(&self, id: Uuid, encrypted: &str) -> Result<String, AppError> {
        let data = STANDARD
            .decode(encrypted.strip_prefix("v1:").ok_or(AppError::Internal)?)
            .map_err(|_| AppError::Internal)?;
        if data.len() < 28 {
            return Err(AppError::Internal);
        }
        let bytes = self
            .0
            .decrypt(
                Nonce::from_slice(&data[..12]),
                Payload {
                    msg: &data[12..],
                    aad: id.as_bytes(),
                },
            )
            .map_err(|_| AppError::Internal)?;
        String::from_utf8(bytes).map_err(|_| AppError::Internal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authenticated_randomized_roundtrip() -> Result<(), AppError> {
        let crypto = Credentials::new(&STANDARD.encode([7; 32]))?;
        let id = Uuid::new_v4();
        let a = crypto.encrypt(id, "sensitive-密码")?;
        let b = crypto.encrypt(id, "sensitive-密码")?;
        assert_ne!(a, b);
        assert!(!a.contains("sensitive"));
        assert_eq!(crypto.decrypt(id, &a)?, "sensitive-密码");
        assert!(crypto.decrypt(Uuid::new_v4(), &a).is_err());
        assert!(crypto.decrypt(id, "v1:AAAA").is_err());
        let other = Credentials::new(&STANDARD.encode([8; 32]))?;
        assert!(other.decrypt(id, &a).is_err());
        assert!(Credentials::new("bad").is_err());
        Ok(())
    }
}
