use rand::Error as RngError;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey(Box<[u8]>);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretKey(Box<[u8]>);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ciphertext(Box<[u8]>);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedSecret(Box<[u8]>);

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] { &self.0 }
}
impl AsRef<[u8]> for SecretKey {
    fn as_ref(&self) -> &[u8] { &self.0 }
}
impl AsRef<[u8]> for Ciphertext {
    fn as_ref(&self) -> &[u8] { &self.0 }
}
impl AsRef<[u8]> for SharedSecret {
    fn as_ref(&self) -> &[u8] { &self.0 }
}

impl PublicKey {
    pub fn from_vec(v: Vec<u8>) -> Self { Self(v.into_boxed_slice()) }
    pub fn from_box(b: Box<[u8]>) -> Self { Self(b) }
}
impl SecretKey {
    pub fn from_vec(v: Vec<u8>) -> Self { Self(v.into_boxed_slice()) }
    pub fn from_box(b: Box<[u8]>) -> Self { Self(b) }
}
impl Ciphertext {
    pub fn from_vec(v: Vec<u8>) -> Self { Self(v.into_boxed_slice()) }
    pub fn from_box(b: Box<[u8]>) -> Self { Self(b) }
}
impl SharedSecret {
    pub fn from_vec(v: Vec<u8>) -> Self { Self(v.into_boxed_slice()) }
    pub fn from_box(b: Box<[u8]>) -> Self { Self(b) }
}

#[derive(Debug, Error)]
pub enum KemError {
    #[error("Invalid key size provided")]
    InvalidKeySize,
    #[error("Invalid ciphertext size")]
    InvalidCiphertextSize,
    #[error("Encapsulation operation failed")]
    EncapsulationError,
    #[error("Decapsulation operation failed")]
    DecapsulationError,
    #[error("Cryptographic RNG failure: {0}")]
    RandomError(#[from] RngError),
}

pub trait Kem: Send + Sync {
    fn keygen(&self) -> Result<(PublicKey, SecretKey), KemError>;
    fn encaps(&self, pk: &PublicKey) -> Result<(Ciphertext, SharedSecret), KemError>;
    fn decaps(&self, ct: &Ciphertext, sk: &SecretKey) -> Result<SharedSecret, KemError>;
    fn public_key_bytes(&self) -> usize;
    fn secret_key_bytes(&self) -> usize;
    fn ciphertext_bytes(&self) -> usize;
    fn shared_secret_bytes(&self) -> usize;
}