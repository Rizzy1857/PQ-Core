//! Number Theoretic Transform (NTT) and modular arithmetic utilities.
//! This module provides a simple, safe, and reusable NTT implementation for polynomials.

/// Computes the in-place Number Theoretic Transform (NTT) of a polynomial.
///
/// # Arguments
/// * `poly` - The polynomial coefficients (will be transformed in-place).
/// * `root` - A principal n-th root of unity modulo `modulus`.
/// * `modulus` - A prime modulus.
///
/// # Panics
/// Panics if `poly.len()` is not a power of two or if `modulus <= 1`.
///
/// # Example
/// ```
/// use pq_core::math::ntt::ntt;
/// let mut poly = vec![1, 2, 3, 4];
/// ntt(&mut poly, 17, 3329);
/// ```
pub fn ntt(poly: &mut [i32], root: i32, modulus: i32) {
    let n = poly.len();
    assert!(n.is_power_of_two(), "NTT input length must be a power of two");
    assert!(modulus > 1, "Modulus must be > 1");
    if n <= 1 { return; }
    let mut even = vec![0; n / 2];
    let mut odd = vec![0; n / 2];
    for i in 0..n / 2 {
        even[i] = poly[2 * i];
        odd[i] = poly[2 * i + 1];
    }
    let root_sq = modpow(root, 2, modulus);
    ntt(&mut even, root_sq, modulus);
    ntt(&mut odd, root_sq, modulus);
    let mut w = 1;
    for i in 0..n / 2 {
        let t = (w * odd[i]) % modulus;
        poly[i] = (even[i] + t) % modulus;
        poly[i + n / 2] = (even[i] - t + modulus) % modulus;
        w = (w * root) % modulus;
    }
}

/// Computes (base^exp) % modulus efficiently.
///
/// # Arguments
/// * `base` - The base integer.
/// * `exp` - The exponent (non-negative).
/// * `modulus` - The modulus (must be > 1).
///
/// # Returns
/// The result of modular exponentiation.
///
/// # Panics
/// Panics if `modulus <= 1`.
pub fn modpow(mut base: i32, mut exp: u32, modulus: i32) -> i32 {
    assert!(modulus > 1, "Modulus must be > 1");
    let mut result = 1;
    base = base % modulus;
    while exp > 0 {
        if exp % 2 == 1 {
            result = (result * base) % modulus;
        }
        exp /= 2;
        base = (base * base) % modulus;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_modpow_basic() {
        assert_eq!(modpow(2, 10, 1000), 24);
        assert_eq!(modpow(3, 0, 7), 1);
        assert_eq!(modpow(5, 3, 13), 8);
    }
    #[test]
    fn test_ntt_power_of_two() {
        let mut poly = vec![1, 2, 3, 4];
        ntt(&mut poly, 17, 3329);
        assert_eq!(poly.len(), 4);
    }
    #[test]
    #[should_panic]
    fn test_ntt_non_power_of_two() {
        let mut poly = vec![1, 2, 3];
        ntt(&mut poly, 17, 3329);
    }
    #[test]
    #[should_panic]
    fn test_modpow_invalid_modulus() {
        modpow(2, 3, 1);
    }
}