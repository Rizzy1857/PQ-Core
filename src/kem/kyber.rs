use pqcrypto_kyber::kyber512;
use pqcrypto_traits::kem::{PublicKey as PQPublicKey, SecretKey as PQSecretKey, Ciphertext as PQCiphertext, SharedSecret as PQSharedSecret};
use super::kem::{Kem, KemError, PublicKey, SecretKey, Ciphertext, SharedSecret};

#[derive(Debug, Clone)]
pub struct Kyber512 {
    _phantom: std::marker::PhantomData<()>,
}

impl Kyber512 {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}


impl Kem for Kyber512 {
    fn keygen(&self) -> Result<(PublicKey, SecretKey), KemError> {
        let (pk_bytes, sk_bytes) = kyber512::keypair();
        Ok((
            PublicKey::from_vec(pk_bytes.as_bytes().to_vec()),
            SecretKey::from_vec(sk_bytes.as_bytes().to_vec()),
        ))
    }

    fn encaps(&self, pk: &PublicKey) -> Result<(Ciphertext, SharedSecret), KemError> {
        if pk.as_ref().len() != kyber512::public_key_bytes() {
            return Err(KemError::InvalidKeySize);
        }
        
        let pk_kyber = kyber512::PublicKey::from_bytes(pk.as_ref())
            .map_err(|_| KemError::InvalidKeySize)?;
        
        let (ss_bytes, ct_bytes) = kyber512::encapsulate(&pk_kyber);
        
        Ok((
            Ciphertext::from_vec(ct_bytes.as_bytes().to_vec()),
            SharedSecret::from_vec(ss_bytes.as_bytes().to_vec()),
        ))
    }

    fn decaps(&self, ct: &Ciphertext, sk: &SecretKey) -> Result<SharedSecret, KemError> {
        if ct.as_ref().len() != kyber512::ciphertext_bytes() {
            return Err(KemError::InvalidCiphertextSize);
        }
        if sk.as_ref().len() != kyber512::secret_key_bytes() {
            return Err(KemError::InvalidKeySize);
        }

        let ct_kyber = kyber512::Ciphertext::from_bytes(ct.as_ref())
            .map_err(|_| KemError::InvalidCiphertextSize)?;
        let sk_kyber = kyber512::SecretKey::from_bytes(sk.as_ref())
            .map_err(|_| KemError::InvalidKeySize)?;

        let ss_bytes = kyber512::decapsulate(&ct_kyber, &sk_kyber);
        
        Ok(SharedSecret::from_vec(ss_bytes.as_bytes().to_vec()))
    }

    fn public_key_bytes(&self) -> usize {
        kyber512::public_key_bytes()
    }

    fn secret_key_bytes(&self) -> usize {
        kyber512::secret_key_bytes()
    }

    fn ciphertext_bytes(&self) -> usize {
        kyber512::ciphertext_bytes()
    }

    fn shared_secret_bytes(&self) -> usize {
        kyber512::shared_secret_bytes()
    }
}