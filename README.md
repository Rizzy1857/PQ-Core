
# PQ-Core [![Rust](https://img.shields.io/badge/Rust-1.70+-blue.svg)](https://www.rust-lang.org/)

---

## Overview

PQ-Core is a Rust library for post-quantum secure communication. It implements a protocol stack using NIST-standardized quantum-resistant primitives:

- **Kyber512** (KEM) for key exchange
- **Dilithium2** for digital signatures
- **AES-256-GCM** for authenticated symmetric encryption
- **HKDF (SHA-256)** for key derivation

All cryptographic operations use audited Rust crates. Secrets are zeroized, and the architecture is modular and extensible.

---

## Features

- **Post-Quantum Security:** Kyber and Dilithium (NIST PQC algorithms)
- **Authenticated Encryption:** AES-256-GCM with HKDF
- **Memory Safety:** Zeroization and constant-time operations
- **Modular Design:** Easy to extend and audit
- **Testing:** Unit and integration tests
- **Demo Example:** See `examples/demo.rs` for a full protocol run

---

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
pq-core = { path = "./PQ-Core" }
```

### Example: End-to-end quantum-safe communication

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

## Security

- **Key Exchange:** Kyber512 (quantum-resistant)
- **Authentication:** Dilithium2 (quantum-resistant)
- **Encryption:** AES-256-GCM (forward secrecy)
- **Key Derivation:** HKDF (SHA-256)
- **Memory Hygiene:** All secrets zeroized

---

## Testing and demo

Run all tests:

```powershell
cargo test
```

Run the demo:

```powershell
cargo run --example demo
```

---

## Documentation

- See `docs/overview.md` for architecture and protocol details
- See `docs/pq_concepts.md` for post-quantum background
- See `docs/protocol_flow.md` for protocol flow
- See `docs/roadmap.md` for project status and future plans

---

## Status

- Phase 1 complete: real cryptography, protocol implemented, tests passing.

---

## Authors

- Maintainer: Rizzy
- Contributors: see Git history

---

## 📜 License

MIT
