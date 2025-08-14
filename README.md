
# PQ-Core 🔒 [![Rust](https://img.shields.io/badge/Rust-1.70+-blue.svg)](https://www.rust-lang.org/)

**Quantum-Safe Secure Communication Protocols in Pure Rust**

---

## 🚀 Overview

PQ-Core is a professional, production-grade Rust library for post-quantum secure communication. It implements a full protocol stack using NIST-standardized quantum-resistant primitives:

- **Kyber512** (KEM) for key exchange
- **Dilithium2** for digital signatures
- **AES-256-GCM** for authenticated symmetric encryption
- **HKDF (SHA-256)** for key derivation

All cryptographic operations use real, production-ready Rust crates. Secrets are zeroized, and the architecture is modular, auditable, and extensible.

---

## ✨ Features

- **Post-Quantum Security:** Kyber and Dilithium (NIST PQC finalists)
- **Authenticated Encryption:** AES-256-GCM with HKDF
- **Memory Safety:** Zeroization and constant-time operations
- **Modular Design:** Easy to extend and audit
- **Comprehensive Testing:** Unit and integration tests
- **Demo Example:** See `examples/demo.rs` for a full protocol run

---

## 📦 Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
pq-core = { path = "./PQ-Core" }
```

### Example: End-to-End Quantum-Safe Communication

```rust
use pq_core::PQSession;

fn main() {
  let mut alice = PQSession::new();
  let mut bob = PQSession::new();

  // Handshake
  let handshake = alice.initiate_handshake().unwrap();
  let response = bob.process_handshake(handshake).unwrap();
  alice.complete_handshake(response).unwrap();

  // Secure message exchange
  let plaintext = b"Hello quantum world!";
  let ciphertext = alice.encrypt(plaintext);
  let decrypted = bob.decrypt(&ciphertext).unwrap();
  assert_eq!(plaintext, decrypted.as_slice());
}
```

---

## � Security

- **Key Exchange:** Kyber512 (quantum-resistant)
- **Authentication:** Dilithium2 (quantum-resistant)
- **Encryption:** AES-256-GCM (forward secrecy)
- **Key Derivation:** HKDF (SHA-256)
- **Memory Hygiene:** All secrets zeroized

---

## 🧪 Testing & Demo

Run all tests:

```sh
cargo test
```

Run the demo:

```sh
cargo run --example demo
```

---

## 📚 Documentation

- See `docs/overview.md` for architecture and protocol details
- See `docs/pq_concepts.md` for post-quantum background
- See `docs/protocol_flow.md` for protocol flow
- See `docs/roadmap.md` for project status and future plans

---

## 🏆 Status

- **Phase 1: Complete** — Real cryptography, full protocol, all tests passing
- **Ready for research, extension, and real-world quantum-safe applications**

---

## 👤 Authors & Credits

- Original author: Rizzy
- Professional refactoring, cryptographic integration, and review: [GitHub Copilot]

---

## 📜 License

MIT
