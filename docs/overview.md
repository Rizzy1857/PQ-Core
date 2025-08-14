# 🧬 PQ-Core: Architecture Overview

## What is PQ-Core?

**PQ-Core** is a professional Rust library that implements a complete post-quantum secure communication protocol. It combines NIST-standardized quantum-resistant primitives with modern cryptographic best practices to provide a production-ready foundation for secure messaging applications.

PQ-Core integrates real cryptographic implementations from the Rust ecosystem, providing a fully functional protocol stack that's ready for research, extension, and real-world deployment.

---

## Architecture Design

### Core Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   KEM Module    │    │   Signature     │    │   Protocol      │
│   (Kyber512)    │    │   (Dilithium2)  │    │   (PQSession)   │
│                 │    │                 │    │                 │
│ - Key Exchange  │    │ - Authentication│    │ - Handshake     │
│ - Encapsulation │    │ - Identity      │    │ - Encryption    │
│ - Decapsulation │    │ - Non-repudiate │    │ - Key Derivation│
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │   Math Module   │
                    │     (NTT)       │
                    │                 │
                    │ - Number Theory │
                    │ - Optimizations │
                    └─────────────────┘
```

### Cryptographic Stack

1. **Key Encapsulation Mechanism (KEM)**
   - **Algorithm:** Kyber512 (NIST PQC finalist)
   - **Implementation:** `pqcrypto-kyber` crate
   - **Purpose:** Quantum-resistant key exchange
   - **Security Level:** NIST Level 1 (~128-bit classical security)

2. **Digital Signatures**
   - **Algorithm:** Dilithium2 (NIST PQC finalist)
   - **Implementation:** `pqcrypto-dilithium` crate
   - **Purpose:** Authentication and non-repudiation
   - **Security Level:** NIST Level 2 (~192-bit classical security)

3. **Symmetric Encryption**
   - **Algorithm:** AES-256-GCM
   - **Implementation:** `aes-gcm` crate
   - **Purpose:** Authenticated encryption of messages
   - **Key Derivation:** HKDF with SHA-256

4. **Mathematical Primitives**
   - **NTT (Number Theoretic Transform):** For polynomial arithmetic
   - **Modular arithmetic:** Constant-time operations
   - **Custom implementation:** Educational and optimization purposes

---

## Protocol Flow

### Phase 1: Handshake
```
Alice                                    Bob
  │                                       │
  │ 1. Generate KEM/Sig keypairs          │
  │ 2. Create handshake message           │
  │────────────────────────────────────►  │
  │                                       │ 3. Verify signature
  │                                       │ 4. Encapsulate to Alice's PK
  │                                       │ 5. Derive shared secret
  │                                       │ 6. Generate response
  │  ◄────────────────────────────────────│
  │ 7. Complete handshake                 │
  │ 8. Derive shared secret               │
  │                                       │
```

### Phase 2: Secure Communication
```
Alice                                    Bob
  │                                       │
  │ 9. Encrypt with AES-256-GCM           │
  │────────────────────────────────────►  │
  │                                       │ 10. Decrypt and verify
  │                                       │
  │  ◄────────────────────────────────────│ 11. Send encrypted response
  │ 12. Decrypt and verify                │
  │                                       │
```

---

## Security Properties

### Quantum Resistance
- **Kyber512:** Based on Module-LWE (lattice) problem
- **Dilithium2:** Based on Module-LWE and Module-SIS problems
- **Forward Secrecy:** HKDF ensures key evolution
- **Post-Quantum Security:** Resistant to both classical and quantum attacks

### Implementation Security
- **Memory Safety:** Rust's ownership model prevents memory vulnerabilities
- **Secret Zeroization:** All cryptographic secrets are securely erased
- **Constant-Time Operations:** Critical paths avoid timing side-channels
- **No Unsafe Code:** Pure safe Rust throughout the protocol layer

### Protocol Security
- **Mutual Authentication:** Both parties verify each other's signatures
- **Replay Protection:** Nonce-based message ordering
- **Perfect Forward Secrecy:** Compromise of long-term keys doesn't affect past sessions
- **Authenticated Encryption:** AES-GCM provides both confidentiality and integrity

---

## Development Status

### ✅ Completed (Phase 1)
- Complete KEM implementation with Kyber512
- Complete signature implementation with Dilithium2
- Full protocol handshake and message encryption
- Comprehensive test suite
- Professional error handling
- Memory safety and secret zeroization
- Documentation and examples

### 🔄 Future Phases
- **Phase 2:** Additional algorithms (Kyber768/1024, Dilithium3/5)
- **Phase 3:** Hybrid classical+PQ schemes
- **Phase 4:** FFI bindings for other languages
- **Phase 5:** Hardware acceleration and optimization
- **Phase 6:** Formal verification and security proofs

---

## Conclusion

PQ-Core represents a professional, production-ready approach to post-quantum cryptography. By combining NIST-standardized algorithms with modern Rust engineering practices, it provides a solid foundation for building quantum-safe applications.

The library is designed for both immediate use and long-term evolution, with a modular architecture that can adapt to the changing landscape of post-quantum cryptography.

**PQ-Core: Ready for the quantum future, today.**
