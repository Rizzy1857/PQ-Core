use pq_core::PQSession;

#[test]
fn test_full_session() {
    let mut alice = PQSession::new();
    let mut bob = PQSession::new();

    // Alice initiates
    let handshake = alice.initiate_handshake();

    // Bob processes handshake
    let response = bob.process_handshake(handshake).unwrap();

    // Alice completes handshake
    alice.complete_handshake(response).unwrap();

    // Now test message exchange
    let plaintext = b"PQ-Core test message";
    let ciphertext = alice.encrypt(plaintext);
    let decrypted = bob.decrypt(&ciphertext).unwrap();

    assert_eq!(plaintext, decrypted.as_slice());
}