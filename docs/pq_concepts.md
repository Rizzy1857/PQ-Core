# PQ Concepts and Post-Quantum Messaging Deep Dive

This document is a technical deep dive describing a decentralized, post-quantum messaging design built on the new NIST post-quantum standards (ML-KEM and ML-DSA). It covers recommended parameter choices, handshake and ratchet design, message formats, key/state management, implementation guidance, and a short threat-model analysis.

--

## 1. Foundational Building Blocks: The Standardized Primitives

This design uses the NIST post-quantum standards:

- **FIPS 203 (ML-KEM)** — module-lattice KEM (CRYSTALS-Kyber family evolution)
- **FIPS 204 (ML-DSA)** — module-lattice digital signatures (CRYSTALS-Dilithium family evolution)

We use ML-KEM for key encapsulation and ML-DSA for authentication. The design intentionally excludes the hash-based FIPS 205 to simplify long-term key handling and signatures.

### 1.1 ML-KEM: Quantum-Safe Key Encapsulation

ML-KEM provides KeyGen, Encapsulate, Decapsulate.

- Parameter sets: ML-KEM-512, ML-KEM-768, ML-KEM-1024. For maximum long-term security, ML-KEM-1024 (Level 5) is recommended.
- Key sizes (ML-KEM-1024 approx):
	- Public Key: ~1,568 bytes
	- Private Key (expanded): ~3,168 bytes (or 64-byte seed)
	- Ciphertext: ~1,568 bytes
	- Shared secret output: 32 bytes

Design note: support both seed and expanded private-key formats for FIPS compatibility and hardware-backed modules.

### 1.2 ML-DSA: Quantum-Safe Digital Signatures

- Parameter sets: ML-DSA-44, ML-DSA-65, ML-DSA-87. For maximum security choose ML-DSA-87 (Level 5).
- Typical sizes (ML-DSA-87): public key ~2,592 bytes, private key ~4,896 bytes, signature ~4,627 bytes.

Like ML-KEM, ML-DSA supports seed vs expanded private key formats; store keys with flexible formats for hardware compatibility.

--

## 2. Core Protocol Design: The Handshake

High-level approach: do a one-time out-of-band static public-key exchange at installation, then perform mutual-authenticated ephemeral KEM handshakes to create sessions. This mirrors the "installation" then "calling" steps of a decentralized messaging app.

### Phase 0: Installation (Out-of-Band Static Key Exchange)

- Each user generates two static keypairs:
	- Signature pair: `(sigPK, sigSK)` using ML-DSA-87
	- KEM pair: `(kemPK, kemSK)` using ML-KEM-1024
- The static public keys are exchanged out-of-band (QR, NFC, secure file transfer). These static keys are used to authenticate subsequent ephemeral exchanges.

### Phase 1: Mutual-Authenticated Session Establishment (Noise-XX-like)

The handshake is inspired by Noise `XX` pattern but uses ML-KEM for key exchange and ML-DSA for static authentication.

1. Alice generates an ephemeral KEM keypair `(kemPK_ephA, kemSK_ephA)`.
2. Alice encapsulates to Bob's static `kemPK_B` producing `(ciphertext_A, shared_secret_A)` and signs the handshake transcript with `sigSK_A`:
	 `signature_A = ML-DSA.Sign(sigSK_A, kemPK_A || kemPK_ephA || ciphertext_A || session_id)`
3. Alice sends `{ kemPK_ephA, ciphertext_A, signature_A }` to Bob.
4. Bob verifies `signature_A` using `sigPK_A` (from local contact storage). Bob decapsulates using `kemSK_B` to recover `shared_secret_A`.
5. Bob performs his own ephemeral encapsulation to Alice creating `{ kemPK_ephB, ciphertext_B, signature_B }` signed with `sigSK_B`.
6. Alice verifies `signature_B` and decapsulates `ciphertext_B` with her `kemSK_A`. Both sides derive a root shared secret using a KDF over the concatenation of relevant public keys, ciphertexts, and session identifiers to prevent re-encapsulation or key-misbinding attacks.

Security notes:
- Domain-separate HKDF inputs ("handshake-root-v1") must include all public keys and ciphertexts.
- Signatures bind static identity to ephemeral values to prevent impersonation.
- Using ephemeral KEM keys ensures forward secrecy against later compromise of static keys.

--

## 3. The Ratchet: Ongoing Conversation Security

Replace classic DH ratchet with a post-quantum ratchet that uses ML-KEM for epoch key agreement (SCKA-style). Combine an asymmetric PQ ratchet with a symmetric chain ratchet (HKDF-based) to produce message keys.

High-level steps:

1. Maintain a `root_key` and a per-epoch symmetric chain (`chain_key`).
2. On an asymmetric ratchet step, use ephemeral ML-KEM to produce a fresh shared secret and mix it into `root_key` via HKDF.
3. From the updated root_key derive a new `chain_key` and then derive per-message `message_key = HKDF(chain_key || "msg" || counter)`.
4. Encrypt each message with AEAD using `message_key` and include the ratchet header (either a KEM ciphertext or ephemeral public key depending on direction).

Out-of-order handling:
- The receiver must be able to derive keys for messages that arrive out-of-order, up to a reasonable window. Use per-message counters and chain indexes in the header.

Ratchet parameters recommendations:
- Asymmetric steps: ML-KEM-768 or ML-KEM-1024 for high security.
- Symmetric KDF: HKDF-SHA256 (domain-separated contexts).
- AEAD: AES-256-GCM (hardware-accelerated platforms) or XChaCha20-Poly1305 on constrained platforms.

--

## 4. Cryptographic Key & State Management

- **Session ID / Fingerprint**: Not secret. Use a short human-friendly fingerprint for contact indexing: `session_id = truncate(SHA3-256(kemPK || sigPK), 20)`.
- **Static Key Storage**: Encrypt long-term private keys at rest (AES-256-GCM) with a key derived from a local user secret (password/PIN) and optionally protected by hardware secure modules (TPM/Keychain).
- **Randomness**: Use system-backed CSPRNG (e.g., /dev/urandom, platform RNGs, or `liboqs` randomness) for ephemeral keys and KEM encapsulations.
- **Ephemeral Key Deletion**: Securely wipe ephemeral KEM secret keys from memory after use.

--

## 5. Message Format & Encryption

Message header must contain ratchet metadata so the receiver can derive the decryption key. Because KEM public keys/ciphertexts are large (~1.5 KB), the first message of a ratchet epoch will be large.

Suggested header fields:

- `epoch_id` (4 bytes)
- `message_counter` (4 bytes)
- `chain_key_index` (8 bytes)
- `ratchet_blob` (either `kem_ciphertext` or `kem_public_key`) (~1.5 KB)
- `payload_length` (2 bytes)
- `aead_tag` (16 bytes)

Payload encryption:
- Use AEAD with a per-message key derived from the ratchet's chain key: `message_key = HKDF(chain_key || "msg" || counter)`.

--

## 6. Implementation Guidance

- Prefer using battle-tested PQ crypto libraries rather than rolling your own:
	- **liboqs** (C) — widely used, test harnesses, and benchmarks.
	- **oqs-provider** for OpenSSL integration (if using TLS or OpenSSL stacks).
	- Language-specific crates: `fips203` / `fips204` Rust crates or `pqcrypto` families for immediate prototyping.

- For mobile apps, use FFI bindings to `liboqs` or stable Rust crates. Protect private keys using platform keystores when available.

- For ratchet logic, adopt an existing audited implementation (e.g., `pq-ratchet` or `clatter`) rather than coding the whole Double Ratchet from scratch.

--

## 7. Security & Threat Model

Properties provided:

- **Confidentiality**: Breaking messages requires breaking ML-KEM or symmetric AEAD; both are post-quantum resistant.
- **Authentication**: Static ML-DSA signatures ensure mutual authentication.
- **Forward Secrecy**: Ephemeral KEM keys for ratchet steps protect past messages if static keys are compromised later.
- **Post-Compromise Security**: Ratchet is designed to recover security after temporary compromises.

Pitfalls & mitigations:

- **KEM binding**: Always derive shared keys with KDF inputs that include all public keys and ciphertexts to avoid re-encapsulation attacks.
- **Domain separation**: Use distinct labels for KDF contexts, e.g., `handshake-root-v1`, `ratchet-msg-v1`.
- **Constant-time implementation**: Use vetted PQ libraries that minimize timing side-channels.
- **Key management**: Store static keys within platform-provided secure storage; encrypt backups.

--

## 8. Summary & Next Steps

This design describes a practical, secure, post-quantum messaging architecture using ML-KEM and ML-DSA at high-security parameter sets. Next implementation steps (incremental):

1. Integrate `liboqs` or `fips203`/`fips204` bindings and implement static key generation and storage.
2. Implement the mutual-authenticated ML-KEM handshake (Noise-XX-like) and test interoperability.
3. Implement the PQ ratchet (SCKA-like asymmetric steps + symmetric chain ratchet).
4. Implement message envelope and AEAD encryption/decryption with proper header parsing and out-of-order handling.
5. Add integration tests and interop tests between two peers.

If you want, I can start implementing these steps incrementally — tell me which step to begin and I'll create the code, tests, and demos.
