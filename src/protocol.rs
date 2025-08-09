use crate::kem::{Kyber512, Kem};
use crate::kem::kem::{PublicKey, SecretKey, Ciphertext};
use crate::sig::dilithium::Dilithium;
use rand::rngs::OsRng;
use rand::RngCore;
use aes_gcm::Aes256Gcm;
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::aead::generic_array::typenum::{U12, U32};
use aes_gcm::aead::generic_array::GenericArray;

pub enum PQState {
    Init,
    HandshakeSent,
    HandshakeReceived,
    Established,
    Error,
}

pub struct HandshakeMessage {
    pub kem_pk: PublicKey,
    pub signature: Vec<u8>,
    pub nonce: u64,
    pub ciphertext: Ciphertext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PQError {
    InvalidSignature,
    InvalidCiphertext,
    Other,
}

pub struct PQSession {
    state: PQState,
    kem: Kyber512,
    sig: Dilithium,
    sk: SecretKey,
    tx_chain_key: [u8; 32],
    rx_chain_key: [u8; 32],
    nonce: u64,
}

impl PQSession {
    pub fn new() -> Self {
        PQSession {
            state: PQState::Init,
            kem: Kyber512::new(),
            sig: Dilithium::new(4, 4, 1 << 17, 95232), // Example params, adjust as needed
            sk: SecretKey::from_vec(vec![0u8; 32]),
            tx_chain_key: [0u8; 32],
            rx_chain_key: [0u8; 32],
            nonce: 0,
        }
    }

    pub fn initiate_handshake(&mut self) -> Result<HandshakeMessage, PQError> {
        let (pk, sk) = self.kem.keygen().map_err(|_| PQError::Other)?;
        self.sk = sk.clone();
        self.state = PQState::HandshakeSent;
        let dummy_sig_sk = vec![0u8; 32];
        let signature = self.sig.sign(pk.as_ref(), &dummy_sig_sk);
        let nonce = random_u64();
        let (ciphertext, _) = self.kem.encaps(&pk).map_err(|_| PQError::Other)?;

        Ok(HandshakeMessage {
            kem_pk: pk,
            signature,
            nonce,
            ciphertext,
        })
    }

    pub fn complete_handshake(&mut self, msg: HandshakeMessage) -> Result<(), PQError> {
        // Verify signature if present
        if !self.sig.verify(msg.kem_pk.as_ref(), &msg.signature) {
            return Err(PQError::InvalidSignature);
        }

        // KEM decapsulation
        let shared_secret = self.kem.decaps(&msg.ciphertext, &self.sk).map_err(|_| PQError::Other)?;

        // Derive chain keys (placeholder HKDF)
        self.tx_chain_key = hkdf_sha3(shared_secret.as_ref(), b"tx");
        self.rx_chain_key = hkdf_sha3(shared_secret.as_ref(), b"rx");

        self.state = PQState::Established;
        Ok(())
    }

    pub fn process_handshake(&mut self, msg: HandshakeMessage) -> Result<HandshakeMessage, PQError> {
        // Stub: echo the handshake for test
        Ok(msg)
    }

    pub fn encrypt(&mut self, plaintext: &[u8]) -> Vec<u8> {
        let key = GenericArray::<u8, U32>::from_slice(&self.tx_chain_key);
        let cipher = Aes256Gcm::new(key);
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[..8].copy_from_slice(&self.nonce.to_le_bytes());
        let nonce = GenericArray::<u8, U12>::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, plaintext).expect("encryption failure!");
        let mut out = Vec::with_capacity(8 + ciphertext.len());
        out.extend_from_slice(&self.nonce.to_le_bytes());
        out.extend_from_slice(&ciphertext);
        self.nonce = self.nonce.wrapping_add(1);
        out
    }

    pub fn decrypt(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>, PQError> {
        if ciphertext.len() < 8 {
            return Err(PQError::InvalidCiphertext);
        }
        let nonce_bytes = &ciphertext[..8];
        let msg_nonce = match nonce_bytes.try_into().ok().and_then(|b: [u8;8]| Some(u64::from_le_bytes(b))) {
            Some(n) => n,
            None => return Err(PQError::InvalidCiphertext),
        };
        if msg_nonce < self.nonce {
            return Err(PQError::Other); // replay detected
        }
        let key = GenericArray::<u8, U32>::from_slice(&self.rx_chain_key);
        let cipher = Aes256Gcm::new(key);
        let mut nonce_full = [0u8; 12];
        nonce_full[..8].copy_from_slice(nonce_bytes);
        let nonce = GenericArray::<u8, U12>::from_slice(&nonce_full);
        let ct = &ciphertext[8..];
        let plaintext = cipher.decrypt(nonce, ct).map_err(|_| PQError::InvalidCiphertext)?;
        self.nonce = msg_nonce.wrapping_add(1);
        Ok(plaintext)
    }
}

/// Placeholder HKDF using SHA3 (replace with real implementation)
fn hkdf_sha3(secret: &[u8], info: &[u8]) -> [u8; 32] {
    use sha3::{Digest, Sha3_256};
    let mut hasher = Sha3_256::default();
    hasher.update(secret);
    hasher.update(info);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result[..32]);
    out
}

fn random_u64() -> u64 {
    let mut bytes = [0u8; 8];
    OsRng.fill_bytes(&mut bytes);
    u64::from_le_bytes(bytes)
}

// Add a stub for Dilithium::verify for compilation
impl Dilithium {
    pub fn verify(&self, _msg: &[u8], _sig: &[u8]) -> bool {
        // TODO: Implement real signature verification
        true
    }
}