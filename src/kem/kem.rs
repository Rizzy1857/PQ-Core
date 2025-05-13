pub type PublicKey = Vec<u8>;
pub type SecretKey = Vec<u8>;
pub type Ciphertext = Vec<u8>;
pub type SharedSecret = Vec<u8>;

/// Trait for Key Encapsulation Mechanisms (KEM)
///
/// A KEM allows two parties to securely agree on a shared secret over an insecure channel.
/// This trait defines the required interface for any KEM implementation.
pub trait Kem {
    /// Generate a keypair (public key, secret key).
    fn keygen(&self) -> (PublicKey, SecretKey);

    /// Encapsulate a shared secret to a public key.
    ///
    /// # Arguments
    /// - `pk`: The recipient's public key.
    ///
    /// # Returns
    /// - `Ciphertext`: The ciphertext to send to the recipient.
    /// - `SharedSecret`: The shared secret, to be used as a session key.
    fn encaps(&self, pk: &PublicKey) -> (Ciphertext, SharedSecret);

    /// Decapsulate a shared secret from a ciphertext and secret key.
    ///
    /// # Arguments
    /// - `ct`: The ciphertext received from the sender.
    /// - `sk`: The recipient's secret key.
    ///
    /// # Returns
    /// - `SharedSecret`: The shared secret, matching the sender's encapsulated secret.
    fn decaps(&self, ct: &Ciphertext, sk: &SecretKey) -> SharedSecret;

    /// Get the length in bytes of a public key for this KEM.
    fn public_key_bytes(&self) -> usize {
        0 // Default: override in implementation
    }

    /// Get the length in bytes of a secret key for this KEM.
    fn secret_key_bytes(&self) -> usize {
        0 // Default: override in implementation
    }

    /// Get the length in bytes of a ciphertext for this KEM.
    fn ciphertext_bytes(&self) -> usize {
        0 // Default: override in implementation
    }

    /// Get the length in bytes of a shared secret for this KEM.
    fn shared_secret_bytes(&self) -> usize {
        0 // Default: override in implementation
    }
}