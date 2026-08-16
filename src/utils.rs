//! Utils methods used by other modules

use hkdf::Hkdf;
use sha2::Sha256;

/// Derive a deterministic 32-byte seed for a given recipient identifier
pub fn derive_seed(recipient_id: &str, secret_key: &[u8]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(None, secret_key);
    let mut seed = [0u8; 32];

    hk.expand(recipient_id.as_bytes(), &mut seed)
        .expect("Couldn't expand hkdf to 32 bytess");
    seed
}

pub fn birthday_collision_probability(num_recipients: usize, num_positions: usize) -> f64 {
    let n = num_recipients as f64;
    let k = 2.0_f64.powi(num_positions as i32);

    let x = -(n * n) / (2.0 * k);
    -x.exp_m1()
}
