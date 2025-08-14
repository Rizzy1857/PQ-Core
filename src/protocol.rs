use crate::kem::{Kyber512, Kem};
use crate::kem::kem::{PublicKey, SecretKey, Ciphertext};
use crate::sig::dilithium::{Dilithium, DilithiumPublicKey, DilithiumSecretKey, DilithiumSignature};
use rand::rngs::OsRng;
use rand::RngCore;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use hkdf::Hkdf;
use sha2::Sha256;

pub enum PQState {
    Init,
    HandshakeSent,
    HandshakeReceived,
    Established,
    Error,
}

pub struct HandshakeMessage {
    pub kem_pk: PublicKey,
    pub sig_pk: DilithiumPublicKey,
    pub signature: DilithiumSignature,
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
    sig_sk: DilithiumSecretKey,
    sig_pk: DilithiumPublicKey,
    tx_chain_key: [u8; 32],
    rx_chain_key: [u8; 32],
    nonce: u64,
}

impl PQSession {
    pub fn new() -> Self {
        let kem = Kyber512::new();
        let sig = Dilithium::new();
        
        // Generate KEM keys
        let (_, sk) = kem.keygen().expect("KEM keygen failed");
        
        // Generate signature keys
        let (sig_pk, sig_sk) = sig.keygen().expect("Signature keygen failed");
        
        PQSession {
            state: PQState::Init,
            kem,
            sig,
            sk,
            sig_sk,
            sig_pk,
            tx_chain_key: [0u8; 32],
            rx_chain_key: [0u8; 32],
            nonce: 0,
        }
    }

    pub fn initiate_handshake(&mut self) -> Result<HandshakeMessage, PQError> {
        let (pk, sk) = self.kem.keygen().map_err(|_| PQError::Other)?;
        self.sk = sk.clone();
        self.state = PQState::HandshakeSent;
        
        // Sign the public key with our signature key
        let signature = self.sig.sign(pk.as_ref(), &self.sig_sk).map_err(|_| PQError::InvalidSignature)?;
        let nonce = random_u64();
        let (ciphertext, _) = self.kem.encaps(&pk).map_err(|_| PQError::Other)?;

        Ok(HandshakeMessage {
            kem_pk: pk,
            sig_pk: self.sig_pk.clone(),
            signature,
            nonce,
            ciphertext,
        })
    }

    pub fn complete_handshake(&mut self, msg: HandshakeMessage) -> Result<(), PQError> {
        // Verify signature if present
        if !self.sig.verify(msg.kem_pk.as_ref(), &msg.signature, &msg.sig_pk).map_err(|_| PQError::InvalidSignature)? {
            return Err(PQError::InvalidSignature);
        }

        // KEM decapsulation
        let shared_secret = self.kem.decaps(&msg.ciphertext, &self.sk).map_err(|_| PQError::Other)?;

        // Derive chain keys using HKDF
        let hk = Hkdf::<Sha256>::new(None, shared_secret.as_ref());
        let mut tx_key = [0u8; 32];
        let mut rx_key = [0u8; 32];
        hk.expand(b"tx", &mut tx_key).map_err(|_| PQError::Other)?;
        hk.expand(b"rx", &mut rx_key).map_err(|_| PQError::Other)?;
        self.tx_chain_key = tx_key;
        self.rx_chain_key = rx_key;

        self.state = PQState::Established;
        Ok(())
    }

    pub fn process_handshake(&mut self, msg: HandshakeMessage) -> Result<HandshakeMessage, PQError> {
        // Verify the incoming signature
        if !self.sig.verify(msg.kem_pk.as_ref(), &msg.signature, &msg.sig_pk).map_err(|_| PQError::InvalidSignature)? {
            return Err(PQError::InvalidSignature);
        }

        // Encapsulate to the incoming public key to get shared secret
        let (ciphertext, shared_secret) = self.kem.encaps(&msg.kem_pk).map_err(|_| PQError::Other)?;

        // Derive chain keys using HKDF
        let hk = Hkdf::<Sha256>::new(None, shared_secret.as_ref());
        let mut tx_key = [0u8; 32];
        let mut rx_key = [0u8; 32];
        hk.expand(b"rx", &mut tx_key).map_err(|_| PQError::Other)?;  // Bob's tx is Alice's rx
        hk.expand(b"tx", &mut rx_key).map_err(|_| PQError::Other)?;  // Bob's rx is Alice's tx
        self.tx_chain_key = tx_key;
        self.rx_chain_key = rx_key;

        // Generate our own key pair for the response
        let (our_pk, our_sk) = self.kem.keygen().map_err(|_| PQError::Other)?;
        self.sk = our_sk;

        // Sign our public key
        let our_signature = self.sig.sign(our_pk.as_ref(), &self.sig_sk).map_err(|_| PQError::InvalidSignature)?;

        self.state = PQState::Established;

        Ok(HandshakeMessage {
            kem_pk: our_pk,
            sig_pk: self.sig_pk.clone(),
            signature: our_signature,
            nonce: msg.nonce + 1,  // Increment the nonce
            ciphertext,
        })
    }

    pub fn encrypt(&mut self, plaintext: &[u8]) -> Vec<u8> {
        let key = Key::<Aes256Gcm>::from_slice(&self.tx_chain_key);
        let cipher = Aes256Gcm::new(key);
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[..8].copy_from_slice(&self.nonce.to_le_bytes());
        let nonce = Nonce::from_slice(&nonce_bytes);
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
        let key = Key::<Aes256Gcm>::from_slice(&self.rx_chain_key);
        let cipher = Aes256Gcm::new(key);
        let mut nonce_full = [0u8; 12];
        nonce_full[..8].copy_from_slice(nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_full);
        let ct = &ciphertext[8..];
        let plaintext = cipher.decrypt(nonce, ct).map_err(|_| PQError::InvalidCiphertext)?;
        self.nonce = msg_nonce.wrapping_add(1);
        Ok(plaintext)
    }
}

fn random_u64() -> u64 {
    let mut bytes = [0u8; 8];
    OsRng.fill_bytes(&mut bytes);
    u64::from_le_bytes(bytes)
}