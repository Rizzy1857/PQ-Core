# PQ-Core 🔒 [![Rust](https://img.shields.io/badge/Rust-1.70+-blue.svg)](https://www.rust-lang.org/)

**A from-scratch implementation of post-quantum secure communication protocols**

## ⚠️ Project Status

Experimental - Under active development. Not yet suitable for production use. (or any use for that matter)

## 🚀 Features

- **Pure-Rust cryptographic primitives**
  - Lattice-based KEM (Kyber512)
  - Lattice-based signatures (Dilithium-style)
  - NTT-accelerated polynomial arithmetic
- **Zero-dependency core** (except randomness/security libs)
- **Memory-safe by design**
  - Zeroization of secrets
  - Constant-time operations where critical
- **Testable/auditable architecture**
