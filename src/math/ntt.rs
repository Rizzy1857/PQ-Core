/// Naive NTT implementation for polynomial multiplication
pub fn ntt(poly: &mut [i32], root: i32, modulus: i32) {
    let n = poly.len();
    if n <= 1 { return; }

    // Split into even/odd coefficients
    let mut even = vec![0; n/2];
    let mut odd = vec![0; n/2];
    
    for i in 0..n/2 {
        even[i] = poly[2*i];
        odd[i] = poly[2*i+1];
    }

    // Recursive NTT
    ntt(&mut even, root.pow(2) % modulus, modulus);
    ntt(&mut odd, root.pow(2) % modulus, modulus);

    // Combine results
    let mut w = 1;
    for i in 0..n/2 {
        let t = (w * odd[i]) % modulus;
        poly[i] = (even[i] + t) % modulus;
        poly[i+n/2] = (even[i] - t) % modulus;
        w = (w * root) % modulus;
    }
}