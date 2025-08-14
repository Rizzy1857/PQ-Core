use pq_core::PQSession;

fn main() {
    println!("🔮 PQ-Core Demo: Post-Quantum Cryptography in Action");
    println!("=====================================================");
    
    // Create two parties: Alice and Bob
    let mut alice = PQSession::new();
    let mut bob = PQSession::new();

    println!("\n🎭 Setting up Alice and Bob...");
    
    // Phase 1: Handshake
    println!("\n🤝 Phase 1: Post-Quantum Handshake");
    println!("   📤 Alice initiating handshake...");
    let handshake = alice.initiate_handshake().expect("Alice handshake failed");
    
    println!("   📥 Bob processing handshake...");
    let response = bob.process_handshake(handshake).expect("Bob handshake processing failed");
    
    println!("   ✅ Alice completing handshake...");
    alice.complete_handshake(response).expect("Alice handshake completion failed");
    
    println!("   🎉 Handshake complete! Secure channel established.");

    // Phase 2: Encrypted Communication
    println!("\n🔐 Phase 2: Secure Communication");
    
    let messages = [
        "Hello from the post-quantum future!",
        "Kyber KEM + Dilithium signatures = quantum-safe crypto",
        "AES-256-GCM provides forward secrecy",
        "🚀 Ready for the quantum computer era!"
    ];
    
    for (i, &message) in messages.iter().enumerate() {
        println!("   📝 Message {}: {}", i + 1, message);
        
        // Alice encrypts
        let ciphertext = alice.encrypt(message.as_bytes());
        println!("      🔒 Encrypted: {} bytes", ciphertext.len());
        
        // Bob decrypts
        let decrypted = bob.decrypt(&ciphertext).expect("Decryption failed");
        let decrypted_str = String::from_utf8(decrypted).expect("Invalid UTF-8");
        println!("      🔓 Decrypted: {}", decrypted_str);
        
        assert_eq!(message, decrypted_str);
        println!("      ✅ Message integrity verified!");
    }
    
    println!("\n🎊 Demo complete! PQ-Core successfully demonstrated:");
    println!("   ✅ Kyber512 Key Encapsulation Mechanism (quantum-resistant)");
    println!("   ✅ Dilithium2 Digital Signatures (quantum-resistant)");  
    println!("   ✅ AES-256-GCM Symmetric Encryption (with HKDF key derivation)");
    println!("   ✅ Full end-to-end post-quantum cryptographic protocol");
    println!("\n🛡️  Your communications are now quantum-computer proof! 🛡️");
}
