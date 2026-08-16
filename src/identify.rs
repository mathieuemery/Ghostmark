//! Identify who leaked the text

use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha20Rng;

use crate::homoglyph::HOMOGLYPHS;
use crate::utils::derive_seed;

/// Recompute the expected homoglyph swap sequence for a candidate recipient
fn expected_bits(cover_text: &str, recipient_id: &str, secret_key: &[u8]) -> Vec<bool> {
    let seed = derive_seed(recipient_id, secret_key);
    let mut rng = ChaCha20Rng::from_seed(seed);
    let n_hg = HOMOGLYPHS.candidate_positions(cover_text).len();
    (0..n_hg).map(|_| rng.random()).collect()
}

/// Extract the actual homoglyph swap sequence observed in a text
fn extract_bits(cover_text: &str, stego_text: &str) -> Vec<bool> {
    let mut hg = Vec::new();
    for (cc, sc) in cover_text.chars().zip(stego_text.chars()) {
        if HOMOGLYPHS.homoglyph_for(cc).is_some() {
            hg.push(HOMOGLYPHS.is_homoglyph_swap(sc));
        }
    }
    hg
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub recipient_id: String,
    pub matches: u32,
    pub total_bits: u32,
}

impl Candidate {
    pub fn match_rate(&self) -> f64 {
        if self.total_bits == 0 {
            0.0
        } else {
            self.matches as f64 / self.total_bits as f64
        }
    }
}

/// Score every candidate against a leaked text
pub fn identify_ranked(
    cover_text: &str,
    leaked_text: &str,
    candidates: &[String],
    secret_key: &[u8],
) -> Vec<Candidate> {
    let observed = extract_bits(cover_text, leaked_text);
    if observed.is_empty() {
        return Vec::new();
    }

    let mut scored: Vec<Candidate> = candidates
        .iter()
        .map(|id| {
            let expected = expected_bits(cover_text, id, secret_key);
            let matches = expected
                .iter()
                .zip(observed.iter())
                .filter(|(a, b)| a == b)
                .count() as u32;
            Candidate {
                recipient_id: id.clone(),
                matches,
                total_bits: observed.len() as u32,
            }
        })
        .collect();

    scored.sort_by_key(|c| std::cmp::Reverse(c.matches));
    scored
}

/// Identify the single best match, if any candidates were provided.
pub fn identify<'a>(
    cover_text: &str,
    leaked_text: &str,
    candidates: &'a [String],
    secret_key: &[u8],
) -> Option<(&'a str, u32)> {
    let ranked = identify_ranked(cover_text, leaked_text, candidates, secret_key);
    let best = ranked.first()?;
    candidates
        .iter()
        .find(|id| id.as_str() == best.recipient_id)
        .map(|id| (id.as_str(), best.matches))
}
