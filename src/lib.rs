pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
// src/lib.rs
pub mod kem;
pub mod math;
pub mod sig;
pub mod protocol;  // This contains PQSession

// Re-export important types at crate root
pub use protocol::PQSession;