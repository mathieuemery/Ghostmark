//! Manages char translation

pub struct Homoglyphs([(char, char); 9]);

impl Homoglyphs {
    pub const fn new(entries: [(char, char); 9]) -> Self {
        Self(entries)
    }

    pub fn homoglyph_for(&self, c: char) -> Option<char> {
        self.0
            .iter()
            .find(|&&(original, _)| original == c)
            .map(|&(_, homoglyph)| homoglyph)
    }

    pub fn is_homoglyph_swap(&self, c: char) -> bool {
        self.0.iter().any(|&(_, homoglyph)| homoglyph == c)
    }

    pub fn candidate_positions(&self, text: &str) -> Vec<usize> {
        text.char_indices()
            .filter(|&(_, c)| self.homoglyph_for(c).is_some())
            .map(|(i, _)| i)
            .collect()
    }
}

pub const HOMOGLYPHS: Homoglyphs = Homoglyphs::new([
    ('a', '\u{0430}'),
    ('c', '\u{0441}'),
    ('e', '\u{0435}'),
    ('i', '\u{0456}'),
    ('o', '\u{043E}'),
    ('p', '\u{0440}'),
    ('s', '\u{0455}'),
    ('x', '\u{0445}'),
    ('y', '\u{0443}'),
]);
