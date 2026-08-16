//! Encode a message

use std::collections::HashMap;

use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha20Rng;

use crate::homoglyph::HOMOGLYPHS;
use crate::utils::{birthday_collision_probability, derive_seed};

/// Validate there are enough positions so the number of recipients
/// don't easily collide
pub fn check_capacity(num_recipients: usize, num_positions: usize) -> String {
    let p = birthday_collision_probability(num_recipients, num_positions);
    let k = 2f64.powi(num_positions as i32);

    let recommended_min_positions = {
        // Solve roughly for positions such that p stays under 1%
        let target_k = (num_recipients as f64).powi(2) / (2.0 * (1.0 / 0.99f64).ln());
        target_k.log2().ceil().max(1.0) as usize
    };

    if p > 0.01 {
        format!(
            "WARNING: with {num_recipients} recipients and only {num_positions} \
             insertion slots (2^{num_positions} = {k:.0} possible watermarks), \
             estimated collision probability is {:.2}%. \
             Two recipients may end up with the same (or a very close) watermark, \
             which breaks your ability to tell them apart if it leaks. \
             Recommend at least {recommended_min_positions} slots \
             (i.e. cover text needs at least that many homoglyph-eligible letters) \
             to keep collision probability under 1%.",
            p * 100.0
        )
    } else {
        format!(
            "OK: with {num_recipients} recipients and {num_positions} insertion slots \
             (2^{num_positions} = {k:.0} possible watermarks), \
             estimated collision probability is {:.4}%.",
            p * 100.0
        )
    }
}

/// Embed a watermark for `recipient_id` into `text` using the homoglyph table
pub fn watermark(text: &str, recipient_id: &str, secret_key: &[u8]) -> String {
    let seed = derive_seed(recipient_id, secret_key);
    let mut rng = ChaCha20Rng::from_seed(seed);

    let hg_positions = HOMOGLYPHS.candidate_positions(text);

    let mut hg_swaps: HashMap<usize, bool> = HashMap::new();
    for pos in &hg_positions {
        hg_swaps.insert(*pos, rng.random());
    }

    let mut out = String::with_capacity(text.len());
    for (i, c) in text.char_indices() {
        if let Some(&swap) = hg_swaps.get(&i) && swap{
            out.push(
                HOMOGLYPHS
                    .homoglyph_for(c)
                    .expect("Couldn't get homoglyph char for c = {c}"),
            );
            continue;
        }
        out.push(c);
    }
    out
}
