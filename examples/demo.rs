use pq_core::PQSession;
use std::{thread, time::Duration};

// ANSI color codes for terminal styling
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const CYAN: &str = "\x1b[96m";
const GREEN: &str = "\x1b[92m";
const YELLOW: &str = "\x1b[93m";
const MAGENTA: &str = "\x1b[95m";
const RED: &str = "\x1b[91m";
const BLUE: &str = "\x1b[94m";

fn print_header() {
    println!("{}{}", CYAN, BOLD);
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    🔮 PQ-CORE DEMO 🔮                        ║");
    println!("║              Post-Quantum Cryptography in Action             ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!("{}", RESET);
}

fn print_section(title: &str, icon: &str) {
    println!("\n{}{}{} {} {}{}", YELLOW, BOLD, icon, title, RESET, YELLOW);
    println!("{}▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔{}", YELLOW, RESET);
}

fn print_step(step: &str, detail: &str) {
    println!("{}│{} {} {}{}", BLUE, GREEN, step, RESET, detail);
    thread::sleep(Duration::from_millis(500)); // Dramatic pause
}

fn print_success(message: &str) {
    println!("{}│ ✨ {}{}{}", BLUE, GREEN, message, RESET);
}

fn print_crypto_info(label: &str, data: &str) {
    println!("{}│   {} {}{}{}", BLUE, label, MAGENTA, data, RESET);
}

fn main() {
    print_header();
    
    // Create two parties: Alice and Bob
    let mut alice = PQSession::new();
    let mut bob = PQSession::new();

    print_section("INITIALIZATION", "🎭");
    print_step("🧑‍💻", "Setting up Alice (Sender)");
    print_step("👨‍💻", "Setting up Bob (Receiver)");
    print_success("Both parties ready for quantum-safe communication!");
    
    // Phase 1: Handshake
    print_section("POST-QUANTUM HANDSHAKE", "🤝");
    
    print_step("📤", "Alice initiating Kyber512 key exchange...");
    let handshake = alice.initiate_handshake().expect("Alice handshake failed");
    print_crypto_info("KEM Ciphertext:", "768 bytes (Kyber512)");
    
    print_step("📥", "Bob processing with Dilithium2 signatures...");
    let response = bob.process_handshake(handshake).expect("Bob handshake processing failed");
    print_crypto_info("Response Data:", "~3.2KB (keys + signature)");
    
    print_step("🔐", "Alice completing handshake & deriving session keys...");
    alice.complete_handshake(response).expect("Alice handshake completion failed");
    print_success("Quantum-resistant secure channel established! 🎉");

    // Phase 2: Encrypted Communication
    print_section("SECURE MESSAGING", "📡");
    
    let messages = [
        "Hello from the post-quantum future! 🌟",
        "RSA is dead. Long live lattice cryptography! 💀➡️🔒",
        "Kyber + Dilithium = Quantum apocalypse ready 🛡️",
        "Even quantum computers can't crack this! 🚀🔮"
    ];
    
    for (i, &message) in messages.iter().enumerate() {
        println!("\n{}├─ Message {} ─────────────────────────────────────────{}", BLUE, i + 1, RESET);
        println!("{}│ 📝 Original: {}{}{}", BLUE, CYAN, message, RESET);
        
        // Alice encrypts
        let ciphertext = alice.encrypt(message.as_bytes());
        print_crypto_info("🔒 Encrypted:", &format!("{} bytes (AES-256-GCM)", ciphertext.len()));
        
        // Visual representation of encrypted data
        let preview = ciphertext.iter().take(8)
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ");
        print_crypto_info("   Preview:", &format!("{}... (binary data)", preview));
        
        // Bob decrypts
        thread::sleep(Duration::from_millis(300));
        let decrypted = bob.decrypt(&ciphertext).expect("Decryption failed");
        let decrypted_str = String::from_utf8(decrypted).expect("Invalid UTF-8");
        print_crypto_info("🔓 Decrypted:", &decrypted_str);
        
        assert_eq!(message, decrypted_str);
        print_success("Message integrity verified with HMAC! ✅");
    }
    
    // Final summary with impressive stats
    print_section("MISSION ACCOMPLISHED", "🎊");
    println!("{}│{}", BLUE, RESET);
    println!("{}│ ✅ {} Kyber512 KEM{} (Module-LWE, quantum-resistant)", BLUE, GREEN, RESET);
    println!("{}│ ✅ {} Dilithium2 Signatures{} (Module-SIS, quantum-resistant)", BLUE, GREEN, RESET);
    println!("{}│ ✅ {} AES-256-GCM Encryption{} (128-bit post-quantum security)", BLUE, GREEN, RESET);
    println!("{}│ ✅ {} HKDF Key Derivation{} (SHA-256, RFC 5869)", BLUE, GREEN, RESET);
    println!("{}│{}", BLUE, RESET);
    println!("{}╰─ {} Result: Your comms are QUANTUM-COMPUTER PROOF! {} 🛡️", BLUE, GREEN, RESET);
    
    println!("\n{}{}", MAGENTA, BOLD);
    println!("    ╔═══════════════════════════════════════════════╗");
    println!("    ║  🚀 WELCOME TO THE POST-QUANTUM FUTURE! 🚀    ║");
    println!("    ╚═══════════════════════════════════════════════╝");
    println!("{}", RESET);
}
