pub enum PQState {
    Init,
    HandshakeSent,
    HandshakeReceived,
    Established,
    Error
}

pub struct PQSession {
    state: PQState,
    kem: Kyber512,
    sig: Dilithium,
    tx_chain_key: [u8; 32],
    rx_chain_key: [u8; 32],
    nonce: u64
}

impl PQSession {
    pub fn new() -> Self {
        PQSession {
            state: PQState::Init,
            kem: Kyber512::new(),
            sig: Dilithium::new(),
            tx_chain_key: [0u8; 32],
            rx_chain_key: [0u8; 32],
            nonce: 0
        }
    }

    pub fn initiate_handshake(&mut self) -> HandshakeMessage {
        let (pk, sk) = self.kem.keygen();
        self.state = PQState::HandshakeSent;
        
        HandshakeMessage {
            kem_pk: pk,
            signature: self.sig.sign(&pk, &self.sig.keygen().1),
            nonce: rand::random()
        }
    }

    pub fn complete_handshake(&mut self, msg: HandshakeMessage) -> Result<(), PQError> {
        // Verify signature if present
        if !self.sig.verify(&msg.kem_pk, &msg.signature) {
            return Err(PQError::InvalidSignature);
        }

        // KEM decapsulation
        let shared_secret = self.kem.decaps(&msg.ciphertext, &self.sk);

        // Derive chain keys
        self.tx_chain_key = hkdf_sha3(&shared_secret, "tx");
        self.rx_chain_key = hkdf_sha3(&shared_secret, "rx");

        self.state = PQState::Established;
        Ok(())
    }
}