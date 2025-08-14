use pqcrypto_dilithium::dilithium2;
use pqcrypto_traits::sign::{PublicKey as PQPublicKey, SecretKey as PQSecretKey, SignedMessage as PQSignedMessage};
use zeroize::Zeroize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DilithiumError {
    InvalidPublicKey,
    InvalidSecretKey,
    InvalidSignature,
    SigningError,
    VerificationError,
}

impl std::fmt::Display for DilithiumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DilithiumError::InvalidPublicKey => write!(f, "Invalid public key"),
            DilithiumError::InvalidSecretKey => write!(f, "Invalid secret key"),
            DilithiumError::InvalidSignature => write!(f, "Invalid signature"),
            DilithiumError::SigningError => write!(f, "Signing operation failed"),
            DilithiumError::VerificationError => write!(f, "Verification failed"),
        }
    }
}

impl std::error::Error for DilithiumError {}

#[derive(Debug, Clone)]
pub struct DilithiumPublicKey {
    inner: Vec<u8>,
}

impl DilithiumPublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DilithiumError> {
        if bytes.len() != dilithium2::public_key_bytes() {
            return Err(DilithiumError::InvalidPublicKey);
        }
        Ok(Self {
            inner: bytes.to_vec(),
        })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }
}

#[derive(Debug, Clone)]
pub struct DilithiumSecretKey {
    inner: Vec<u8>,
}

impl DilithiumSecretKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DilithiumError> {
        if bytes.len() != dilithium2::secret_key_bytes() {
            return Err(DilithiumError::InvalidSecretKey);
        }
        Ok(Self {
            inner: bytes.to_vec(),
        })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }
}

impl Drop for DilithiumSecretKey {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

#[derive(Debug, Clone)]
pub struct DilithiumSignature {
    inner: Vec<u8>,
}

impl DilithiumSignature {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DilithiumError> {
        if bytes.len() != dilithium2::signature_bytes() {
            return Err(DilithiumError::InvalidSignature);
        }
        Ok(Self {
            inner: bytes.to_vec(),
        })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }
}

#[derive(Debug, Clone)]
pub struct Dilithium;

impl Dilithium {
    pub fn new() -> Self {
        Self
    }

    pub fn keygen(&self) -> Result<(DilithiumPublicKey, DilithiumSecretKey), DilithiumError> {
        let (pk_bytes, sk_bytes) = dilithium2::keypair();
        
        Ok((
            DilithiumPublicKey {
                inner: pk_bytes.as_bytes().to_vec(),
            },
            DilithiumSecretKey {
                inner: sk_bytes.as_bytes().to_vec(),
            },
        ))
    }

    pub fn sign(&self, message: &[u8], sk: &DilithiumSecretKey) -> Result<DilithiumSignature, DilithiumError> {
        let sk_dilithium = dilithium2::SecretKey::from_bytes(&sk.inner)
            .map_err(|_| DilithiumError::InvalidSecretKey)?;
        
        let sig_bytes = dilithium2::sign(message, &sk_dilithium);
        
        Ok(DilithiumSignature {
            inner: sig_bytes.as_bytes().to_vec(),
        })
    }

    pub fn verify(&self, message: &[u8], signature: &DilithiumSignature, pk: &DilithiumPublicKey) -> Result<bool, DilithiumError> {
        let pk_dilithium = dilithium2::PublicKey::from_bytes(&pk.inner)
            .map_err(|_| DilithiumError::InvalidPublicKey)?;
        
        // Create a SignedMessage from the signature bytes
        let signed_msg = dilithium2::SignedMessage::from_bytes(&signature.inner)
            .map_err(|_| DilithiumError::InvalidSignature)?;
        
        // Use open to verify and recover the message
        match dilithium2::open(&signed_msg, &pk_dilithium) {
            Ok(recovered_msg) => Ok(recovered_msg == message),
            Err(_) => Ok(false),
        }
    }

    pub fn public_key_bytes() -> usize {
        dilithium2::public_key_bytes()
    }

    pub fn secret_key_bytes() -> usize {
        dilithium2::secret_key_bytes()
    }

    pub fn signature_bytes() -> usize {
        dilithium2::signature_bytes()
    }
}