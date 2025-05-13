/// Naive recursive NTT implementation for polynomial multiplication.
/// This is for educational and prototyping purposes only.
pub fn ntt(poly: &mut [i32], root: i32, modulus: i32) {
    let n = poly.len();
    if n <= 1 { return; }

    // Split into even/odd coefficients
    let mut even = vec![0; n / 2];
    let mut odd = vec![0; n / 2];

    for i in 0..n / 2 {
        even[i] = poly[2 * i];
        odd[i] = poly[2 * i + 1];
    }

    // Modular exponentiation for root^2 % modulus
    let root_sq = modpow(root, 2, modulus);

    // Recursive NTT
    ntt(&mut even, root_sq, modulus);
    ntt(&mut odd, root_sq, modulus);

    // Combine results
    let mut w = 1;
    for i in 0..n / 2 {
        let t = (w * odd[i]) % modulus;
        poly[i] = (even[i] + t) % modulus;
        poly[i + n / 2] = (even[i] - t + modulus) % modulus; // Ensure non-negative
        w = (w * root) % modulus;
    }
}

/// Modular exponentiation: computes (base^exp) % modulus
fn modpow(mut base: i32, mut exp: u32, modulus: i32) -> i32 {
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