//! PQ-Core: Post-Quantum Cryptography Core Library
//! Provides KEM, signature, and protocol primitives for post-quantum secure applications.

pub mod kem;
pub mod math;
pub mod sig;
pub mod protocol;
pub mod bidirectional;

pub use protocol::PQSession;
pub use bidirectional::{BidirectionalSession, MessageEnvelope, MessageType};