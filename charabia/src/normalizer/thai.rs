use std::borrow::Cow;

use super::{Normalizer, NormalizerOption};
use crate::{Script, Token};

const NIKHAHIT: char = '\u{e4d}';
const SARA_AA: char = '\u{e32}';
const SARA_AM: char = '\u{e33}';

/// Returns `true` if the character is an invisible zero-width character:
/// - `\u{200B}` ZERO WIDTH SPACE (ZWSP)
/// - `\u{200C}` ZERO WIDTH NON-JOINER (ZWNJ)
/// - `\u{200D}` ZERO WIDTH JOINER (ZWJ)
/// - `\u{FEFF}` ZERO WIDTH NO-BREAK SPACE / BYTE ORDER MARK (BOM)
#[inline]
pub(crate) fn is_zero_width(c: char) -> bool {
    matches!(c, '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{feff}')
}

/// Returns `true` if the character is a Thai combining mark:
/// - Vowels: U+0E31 (ั), U+0E34–U+0E3A (ิ, ี, ึ, ื, ุ, ู, ฺ)
/// - Tone marks: U+0E48–U+0E4B (่, ้, ๊, ๋)
/// - Other marks: U+0E47 (็), U+0E4C–U+0E4E (์, ํ, ๎)
#[inline]
pub(crate) fn is_thai_combining_mark(c: char) -> bool {
    matches!(c, '\u{e31}' | '\u{e34}'..='\u{e3a}' | '\u{e47}'..='\u{e4e}')
}

/// A [`Normalizer`] for the Thai script.
///
/// Thai combining marks are semantically significant characters that must be
/// preserved during normalization:
///
/// - **Vowels** (U+0E31, U+0E34–U+0E3A, U+0E47): Change the meaning of a word.
///   e.g. วิทยุ (radio) vs วิทย (not a word)
/// - **Tone marks** (U+0E48–U+0E4B): Distinguish different words.
///   e.g. ง่าย (easy) vs งาย (different word)
/// - **Thanthakhat / Silence mark** (U+0E4C): Silences a consonant.
///   e.g. มนุษย์ (human) vs มนุษย (incorrect form)
///
/// Unlike Arabic or Hebrew where diacritics are optional annotations,
/// Thai combining marks are integral to correct orthography and must be
/// preserved to ensure accurate search results.
///
/// Additionally, this normalizer:
/// - **Filters Zero-Width Characters** (U+200B, U+200C, U+200D, U+FEFF): Removes
///   invisible zero-width space, non-joiner, joiner, and BOM characters from
///   tokens while keeping character offsets in sync.
/// - **Deduplicates Combining Marks**: Removes consecutive duplicate Thai combining
///   marks (e.g. repeated tone marks like ้้ or repeated vowels like ิิ) that often
///   occur from typing errors or PDF copy-paste, while keeping character offsets in sync.
/// - **Recomposes Sara Am** (ำ, U+0E33): Restores Sara Am which may have
///   been split into Nikhahit (U+0E4D) + Sara Aa (U+0E32) by the
///   [`CompatibilityDecompositionNormalizer`](super::CompatibilityDecompositionNormalizer).
///   Sara Am is a single, meaningful vowel in Thai and should be treated as one unit
///   for indexing purposes. Sara Am typed as Nikhahit + tone mark + Sara Aa (e.g. นํ้า)
///   is also recomposed, into tone mark + Sara Am (น้ำ).
///
/// The normalization is not lossy, so this normalizer is part of the default
/// normalizers and runs whether or not lossy normalization is enabled.
pub struct ThaiNormalizer;

impl Normalizer for ThaiNormalizer {
    fn normalize<'o>(&self, mut token: Token<'o>, options: &NormalizerOption) -> Token<'o> {
        // `Normalize for &str` calls every normalizer without checking `should_normalize`,
        // so return early to avoid allocating for text that does not need normalization.
        if !needs_normalization(token.lemma()) {
            return token;
        }

        match token.char_map.take() {
            Some(mut char_map) => {
                // a char_map already exists, keep it in sync with the recomposed lemma.
                token.lemma = Cow::Owned(recompose_with_char_map(&token.lemma, &mut char_map));
                token.char_map = Some(char_map);
            }
            None if options.create_char_map => {
                // no char_map exists, the lemma is considered to be the original string.
                let mut char_map: Vec<_> =
                    token.lemma.chars().map(|c| (c.len_utf8() as u8, c.len_utf8() as u8)).collect();
                token.lemma = Cow::Owned(recompose_with_char_map(&token.lemma, &mut char_map));
                token.char_map = Some(char_map);
            }
            None => {
                token.lemma = Cow::Owned(recompose(&token.lemma).flat_map(|(_, c)| c).collect());
            }
        }

        token
    }

    fn should_normalize(&self, token: &Token) -> bool {
        token.script == Script::Thai && needs_normalization(token.lemma())
    }
}

/// Returns `true` if the text contains characters that need normalization
/// (decomposed Sara Am, Zero-Width characters, or duplicate combining marks inside a word).
fn needs_normalization(text: &str) -> bool {
    contains_decomposed_sara_am(text)
        || contains_zero_width_inside_word(text)
        || contains_duplicate_combining_marks(text)
}

fn contains_duplicate_combining_marks(text: &str) -> bool {
    let mut prev = None;
    for c in text.chars() {
        if is_zero_width(c) {
            continue;
        }
        if is_thai_combining_mark(c) && prev == Some(c) {
            return true;
        }
        if is_thai_combining_mark(c) {
            prev = Some(c);
        } else {
            prev = None;
        }
    }
    false
}

fn contains_zero_width_inside_word(text: &str) -> bool {
    text.chars().any(is_zero_width) && text.chars().any(|c| !is_zero_width(c))
}

/// Returns `true` if the text contains Sara Am typed as Nikhahit (U+0E4D) + Sara Aa (U+0E32),
/// with or without a tone mark between them.
fn contains_decomposed_sara_am(text: &str) -> bool {
    text.match_indices(NIKHAHIT).any(|(i, _)| completes_sara_am(&text[i + NIKHAHIT.len_utf8()..]))
}

/// Returns `true` if the text following a Nikhahit starts with Sara Aa,
/// optionally preceded by a tone mark (e.g. นํ้า).
fn completes_sara_am(rest: &str) -> bool {
    let mut chars = rest.chars();
    match chars.next() {
        Some(SARA_AA) => true,
        Some('\u{e48}'..='\u{e4b}') => chars.next() == Some(SARA_AA),
        _ => false,
    }
}

/// Recompose Nikhahit (U+0E4D) + Sara Aa (U+0E32) into Sara Am (U+0E33), remove zero-width characters,
/// and deduplicate consecutive combining marks, yielding the length of each original char and the char it is replaced by.
fn recompose(lemma: &str) -> impl Iterator<Item = (usize, Option<char>)> + '_ {
    let drop_zero_width = lemma.chars().any(|c| !is_zero_width(c));
    let mut pending_sara_am = false;
    let mut prev_combining = None;
    lemma.char_indices().map(move |(i, c)| {
        let recomposed = match c {
            // zero-width characters inside a word are dropped.
            c if is_zero_width(c) && drop_zero_width => None,
            // duplicate combining mark is dropped.
            c if is_thai_combining_mark(c) && prev_combining == Some(c) => None,
            // the following Sara Aa is replaced by Sara Am.
            NIKHAHIT if completes_sara_am(&lemma[i + c.len_utf8()..]) => {
                pending_sara_am = true;
                None
            }
            SARA_AA if pending_sara_am => {
                pending_sara_am = false;
                Some(SARA_AM)
            }
            c => Some(c),
        };
        if !is_zero_width(c) {
            prev_combining = if is_thai_combining_mark(c) { Some(c) } else { None };
        }
        (c.len_utf8(), recomposed)
    })
}

/// Recompose Sara Am, see [`recompose`],
/// updating the normalized lengths of the `char_map` accordingly.
///
/// When Nikhahit and Sara Aa span several `char_map` entries (e.g. the original text
/// was typed as Nikhahit + Sara Aa), Nikhahit is dropped and Sara Aa becomes Sara Am,
/// so that the recomposed lemma still covers the whole original text.
fn recompose_with_char_map(lemma: &str, char_map: &mut [(u8, u8)]) -> String {
    let mut recomposed = String::with_capacity(lemma.len());
    let mut chars = recompose(lemma);
    for (_, normalized_len) in char_map.iter_mut() {
        let start = recomposed.len();
        let mut remaining = *normalized_len as usize;
        while remaining > 0 {
            let Some((len, c)) = chars.next() else { break };
            remaining = remaining.saturating_sub(len);
            recomposed.extend(c);
        }
        *normalized_len = (recomposed.len() - start) as u8;
    }

    recomposed
}

#[cfg(test)]
mod test {
    use std::borrow::Cow::{Borrowed, Owned};

    use crate::normalizer::{Normalizer, NormalizerOption};
    use crate::{Language, Script, Token};

    use super::ThaiNormalizer;

    const NORMALIZER_OPTIONS: NormalizerOption = NormalizerOption {
        create_char_map: true,
        lossy: true,
        classifier: crate::normalizer::ClassifierOption { stop_words: None, separators: None },
    };

    /// Helper to normalize a single token with ThaiNormalizer
    fn normalize(token: Token<'static>) -> Token<'static> {
        if ThaiNormalizer.should_normalize(&token) {
            ThaiNormalizer.normalize(token, &NORMALIZER_OPTIONS)
        } else {
            token
        }
    }

    // --- Tests for Sara Am recomposition ---

    #[test]
    fn recompose_sara_am_trailing() {
        // วิทยุ does not contain Sara Am — should be unchanged
        let token =
            Token {
                lemma: Owned("วิทยุ".to_string()), script: Script::Thai, ..Default::default()
            };
        let result = normalize(token);
        assert_eq!(result.lemma(), "วิทยุ");
    }

    #[test]
    fn normalize_without_decomposed_sara_am_does_not_allocate() {
        // `normalize` can be called without `should_normalize`, e.g. when normalizing a `&str`.
        for lemma in ["วิทยุ", "น้ำ", "hello"] {
            let token = Token { lemma: Borrowed(lemma), ..Default::default() };
            let result = ThaiNormalizer.normalize(token, &NORMALIZER_OPTIONS);
            assert!(matches!(result.lemma, Borrowed(_)), "{lemma} must not be reallocated");
            assert!(result.char_map.is_none());
        }
    }

    #[test]
    fn recompose_sara_am_decomposed() {
        // น้ำ after NFKD decomposition: น + ้ + U+0E4D + า → should become น + ้ + ำ
        let decomposed = "\u{e19}\u{e49}\u{e4d}\u{e32}"; // น + mai tho + Nikhahit + Sara Aa
        let token = Token {
            lemma: Owned(decomposed.to_string()),
            script: Script::Thai,
            ..Default::default()
        };
        let result = normalize(token);
        // Expected: น + ้ + ำ (Sara Am U+0E33)
        assert_eq!(result.lemma(), "\u{e19}\u{e49}\u{e33}");
    }

    #[test]
    fn recompose_sara_am_in_word() {
        // น้ำยา decomposed → น + ้ + U+0E4D + า + ย + า
        let decomposed = "\u{e19}\u{e49}\u{e4d}\u{e32}\u{e22}\u{e32}";
        let token = Token {
            lemma: Owned(decomposed.to_string()),
            script: Script::Thai,
            ..Default::default()
        };
        let result = normalize(token);
        assert_eq!(result.lemma(), "\u{e19}\u{e49}\u{e33}\u{e22}\u{e32}");
    }

    /// Verify that Sara Am recomposition keeps an existing `char_map` in sync
    /// with the recomposed lemma when the original text was typed as
    /// Nikhahit + Sara Aa (two original characters).
    #[test]
    fn test_sara_am_recomposition_with_existing_char_map() {
        // Decomposed น้ำ: น(3 bytes) + ้(3 bytes) + U+0E4D(3 bytes) + า(3 bytes)
        let decomposed = "\u{e19}\u{e49}\u{e4d}\u{e32}";
        let token = Token {
            lemma: Owned(decomposed.to_string()),
            char_end: decomposed.chars().count(),
            byte_end: decomposed.len(),
            script: Script::Thai,
            language: Some(Language::Tha),
            // Simulate a char_map that a previous normalizer produced.
            // Each source char maps 3 input bytes → 3 output bytes.
            char_map: Some(vec![(3, 3), (3, 3), (3, 3), (3, 3)]),
            ..Default::default()
        };

        let result = normalize(token);

        // Lemma must be recomposed (3 chars: น + ้ + ำ).
        assert_eq!(result.lemma(), "\u{e19}\u{e49}\u{e33}");
        // Nikhahit is dropped and Sara Aa becomes Sara Am.
        assert_eq!(result.char_map, Some(vec![(3, 3), (3, 3), (3, 0), (3, 3)]));
        // the whole lemma covers the whole original string.
        assert_eq!(result.original_lengths(result.lemma().len()), (4, 12));
    }

    /// Sara Am decomposed by the CompatibilityDecompositionNormalizer is a single
    /// original character mapped to Nikhahit + Sara Aa in the `char_map`.
    #[test]
    fn test_sara_am_recomposition_of_compatibility_decomposition() {
        // น้ำ after NFKD: ำ (3 original bytes) is mapped to U+0E4D + U+0E32 (6 bytes).
        let token = Token {
            lemma: Owned("\u{e19}\u{e49}\u{e4d}\u{e32}".to_string()),
            char_end: 3,
            byte_end: 9,
            script: Script::Thai,
            char_map: Some(vec![(3, 3), (3, 3), (3, 6)]),
            ..Default::default()
        };

        let result = normalize(token);

        assert_eq!(result.lemma(), "น้ำ");
        assert_eq!(result.char_map, Some(vec![(3, 3), (3, 3), (3, 3)]));
    }

    /// Without an existing `char_map`, the created `char_map` must account for
    /// the dropped Nikhahit.
    #[test]
    fn test_sara_am_recomposition_creates_char_map() {
        let decomposed = "\u{e17}\u{e4d}\u{e32}\u{e07}\u{e32}\u{e19}"; // ทํางาน
        let token = Token {
            lemma: Owned(decomposed.to_string()),
            char_end: decomposed.chars().count(),
            byte_end: decomposed.len(),
            script: Script::Thai,
            ..Default::default()
        };

        let result = normalize(token);

        assert_eq!(result.lemma(), "ทำงาน");
        assert_eq!(result.char_map, Some(vec![(3, 3), (3, 0), (3, 3), (3, 3), (3, 3), (3, 3)]));
        assert_eq!(result.original_lengths(result.lemma().len()), (6, 18));
    }

    /// Sara Am may be typed as Nikhahit + tone mark + Sara Aa, e.g. in legacy text.
    #[test]
    fn recompose_sara_am_with_tone_mark_after_nikhahit() {
        for (decomposed, expected) in [
            ("\u{e19}\u{e4d}\u{e49}\u{e32}", "น้ำ"), // น + ํ + ้ + า
            ("\u{e15}\u{e4d}\u{e48}\u{e32}", "ต่ำ"), // ต + ํ + ่ + า
            ("\u{e19}\u{e4d}\u{e49}\u{e32}\u{e15}\u{e32}\u{e25}", "น้ำตาล"),
        ] {
            for create_char_map in [false, true] {
                let options = NormalizerOption { create_char_map, ..NORMALIZER_OPTIONS };
                let token = Token {
                    lemma: Owned(decomposed.to_string()),
                    script: Script::Thai,
                    ..Default::default()
                };
                assert!(ThaiNormalizer.should_normalize(&token));
                let result = ThaiNormalizer.normalize(token, &options);
                assert_eq!(result.lemma(), expected);
            }
        }
    }

    /// Nikhahit is not recomposed when it is not followed by Sara Aa,
    /// even after a tone mark.
    #[test]
    fn nikhahit_without_sara_aa_is_kept() {
        for lemma in ["\u{e19}\u{e4d}\u{e49}", "\u{e19}\u{e4d}\u{e49}\u{e22}\u{e32}", "\u{e4d}"] {
            let token =
                Token { lemma: Borrowed(lemma), script: Script::Thai, ..Default::default() };
            assert!(!ThaiNormalizer.should_normalize(&token));
            let result = ThaiNormalizer.normalize(token, &NORMALIZER_OPTIONS);
            assert!(matches!(result.lemma, Borrowed(_)), "{lemma} must not be reallocated");
        }
    }

    /// The tone mark keeps its own `char_map` entry, so that each original char
    /// is still mapped to its normalized char.
    #[test]
    fn test_sara_am_with_tone_mark_after_nikhahit_char_map() {
        let decomposed = "\u{e19}\u{e4d}\u{e49}\u{e32}"; // นํ้า
        let token = Token {
            lemma: Owned(decomposed.to_string()),
            char_end: decomposed.chars().count(),
            byte_end: decomposed.len(),
            script: Script::Thai,
            ..Default::default()
        };

        let result = normalize(token);

        assert_eq!(result.lemma(), "น้ำ");
        assert_eq!(result.char_map, Some(vec![(3, 3), (3, 0), (3, 3), (3, 3)]));
        assert_eq!(result.original_lengths(result.lemma().len()), (4, 12));
    }

    // --- Integration test: full normalization pipeline ---

    #[test]
    fn full_pipeline_preserves_thai_marks() {
        use crate::normalizer::Normalize;

        let options =
            NormalizerOption { create_char_map: false, lossy: true, ..Default::default() };

        // วิทยุ — trailing sara u should be preserved (the bug from issue #371)
        let token = Token {
            lemma: Owned("วิทยุ".to_string()),
            char_end: "วิทยุ".chars().count(),
            byte_end: "วิทยุ".len(),
            script: Script::Thai,
            language: Some(Language::Tha),
            ..Default::default()
        };
        let normalized = token.normalize(&options);
        assert_eq!(normalized.lemma(), "วิทยุ", "trailing sara u (ุ) must be preserved");

        // ธาตุ — trailing sara u
        let token = Token {
            lemma: Owned("ธาตุ".to_string()),
            char_end: "ธาตุ".chars().count(),
            byte_end: "ธาตุ".len(),
            script: Script::Thai,
            language: Some(Language::Tha),
            ..Default::default()
        };
        let normalized = token.normalize(&options);
        assert_eq!(normalized.lemma(), "ธาตุ", "trailing sara u (ุ) must be preserved");

        // มนุษย์ — trailing thanthakhat (silence mark)
        let token = Token {
            lemma: Owned("มนุษย์".to_string()),
            char_end: "มนุษย์".chars().count(),
            byte_end: "มนุษย์".len(),
            script: Script::Thai,
            language: Some(Language::Tha),
            ..Default::default()
        };
        let normalized = token.normalize(&options);
        assert_eq!(normalized.lemma(), "มนุษย์", "thanthakhat (์) must be preserved");

        // ง่าย — tone mark
        let token = Token {
            lemma: Owned("ง่าย".to_string()),
            char_end: "ง่าย".chars().count(),
            byte_end: "ง่าย".len(),
            script: Script::Thai,
            language: Some(Language::Tha),
            ..Default::default()
        };
        let normalized = token.normalize(&options);
        assert_eq!(normalized.lemma(), "ง่าย", "tone mark (่) must be preserved");

        // น้ำ — Sara Am must survive (not decomposed)
        let token = Token {
            lemma: Owned("น้ำ".to_string()),
            char_end: "น้ำ".chars().count(),
            byte_end: "น้ำ".len(),
            script: Script::Thai,
            language: Some(Language::Tha),
            ..Default::default()
        };
        let normalized = token.normalize(&options);
        assert_eq!(normalized.lemma(), "น้ำ", "Sara Am (ำ) must not be decomposed");
    }

    /// Complement to `full_pipeline_preserves_thai_marks` that runs the full
    /// normalizer pipeline with `create_char_map: true`.  This ensures that
    /// Sara Am recomposition and any char_map-related paths do not regress or panic
    /// when the pipeline is asked to produce position mappings.
    #[test]
    fn full_pipeline_preserves_thai_marks_with_char_map() {
        use crate::normalizer::Normalize;

        let options = NormalizerOption { create_char_map: true, lossy: true, ..Default::default() };

        // น้ำ — Sara Am must survive full pipeline even with char_map enabled.
        let token = Token {
            lemma: Owned("น้ำ".to_string()),
            char_end: "น้ำ".chars().count(),
            byte_end: "น้ำ".len(),
            script: Script::Thai,
            language: Some(Language::Tha),
            ..Default::default()
        };
        let normalized = token.normalize(&options);
        assert_eq!(normalized.lemma(), "น้ำ", "Sara Am (ำ) must not be decomposed (char_map path)");

        // วิทยุ — vowels preserved with char_map enabled.
        let token = Token {
            lemma: Owned("วิทยุ".to_string()),
            char_end: "วิทยุ".chars().count(),
            byte_end: "วิทยุ".len(),
            script: Script::Thai,
            language: Some(Language::Tha),
            ..Default::default()
        };
        let normalized = token.normalize(&options);
        assert_eq!(
            normalized.lemma(),
            "วิทยุ",
            "trailing sara u (ุ) must be preserved (char_map path)"
        );

        // มนุษย์ — silence mark preserved with char_map enabled.
        let token = Token {
            lemma: Owned("มนุษย์".to_string()),
            char_end: "มนุษย์".chars().count(),
            byte_end: "มนุษย์".len(),
            script: Script::Thai,
            language: Some(Language::Tha),
            ..Default::default()
        };
        let normalized = token.normalize(&options);
        assert_eq!(normalized.lemma(), "มนุษย์", "thanthakhat (์) must be preserved (char_map path)");
    }

    /// Sara Am recomposition is not lossy: it must also be applied when lossy
    /// normalization is disabled.
    #[test]
    fn full_pipeline_recomposes_sara_am_without_lossy() {
        use crate::normalizer::Normalize;

        for create_char_map in [false, true] {
            let options = NormalizerOption { create_char_map, lossy: false, ..Default::default() };

            for word in ["น้ำ", "ทำ", "น้ำตาล"] {
                let token = Token {
                    lemma: Owned(word.to_string()),
                    char_end: word.chars().count(),
                    byte_end: word.len(),
                    script: Script::Thai,
                    language: Some(Language::Tha),
                    ..Default::default()
                };
                let normalized = token.normalize(&options);
                assert_eq!(normalized.lemma(), word, "Sara Am (ำ) must not be decomposed");
                if create_char_map {
                    assert_eq!(
                        normalized.original_lengths(normalized.lemma().len()),
                        (word.chars().count(), word.len())
                    );
                }
            }
        }
    }

    /// The forms of น้ำ found in real-world text are all normalized to the same lemma.
    #[test]
    fn full_pipeline_normalizes_all_sara_am_forms() {
        use crate::normalizer::Normalize;

        for lossy in [false, true] {
            for create_char_map in [false, true] {
                let options = NormalizerOption { create_char_map, lossy, ..Default::default() };

                for word in [
                    "น้ำ",                           // composed Sara Am
                    "\u{e19}\u{e49}\u{e4d}\u{e32}", // tone mark + Nikhahit + Sara Aa
                    "\u{e19}\u{e4d}\u{e49}\u{e32}", // Nikhahit + tone mark + Sara Aa
                ] {
                    let token = Token {
                        lemma: Owned(word.to_string()),
                        char_end: word.chars().count(),
                        byte_end: word.len(),
                        script: Script::Thai,
                        language: Some(Language::Tha),
                        ..Default::default()
                    };
                    let normalized = token.normalize(&options);
                    assert_eq!(normalized.lemma(), "น้ำ", "{word:?} must be normalized to น้ำ");
                    if create_char_map {
                        assert_eq!(
                            normalized.original_lengths(normalized.lemma().len()),
                            (word.chars().count(), word.len())
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn zero_width_characters_are_removed_during_normalization() {
        use crate::normalizer::Normalize;

        for word in [
            "ภาษา\u{200b}ไทย", // ZWSP
            "ภาษา\u{200c}ไทย", // ZWNJ
            "ภาษา\u{200d}ไทย", // ZWJ
            "ภาษา\u{feff}ไทย", // BOM / ZWNBSP
            "ภา\u{200b}ษา\u{feff}ไทย",
        ] {
            let token = Token {
                lemma: Owned(word.to_string()),
                char_end: word.chars().count(),
                byte_end: word.len(),
                script: Script::Thai,
                language: Some(Language::Tha),
                ..Default::default()
            };
            let options =
                NormalizerOption { create_char_map: true, lossy: false, ..Default::default() };
            let normalized = token.normalize(&options);
            assert_eq!(normalized.lemma(), "ภาษาไทย");
            // char_map should map normalized length back to original length including zero-width chars
            assert_eq!(
                normalized.original_lengths(normalized.lemma().len()),
                (word.chars().count(), word.len())
            );
        }
    }

    #[test]
    fn duplicate_combining_marks_are_deduplicated_during_normalization() {
        use crate::normalizer::Normalize;

        for (input, expected) in [
            ("น้้้ำ", "น้ำ"),
            ("วี้้", "วี้"),
            ("วิิชา", "วิชา"),
            ("ดุุ", "ดุ"),
            ("สิทธิ์์", "สิทธิ์"),
            ("วานััน", "วานัน"),
            ("ง่่าย", "ง่าย"),
            ("ข้้้อความ", "ข้อความ"),
        ] {
            let token = Token {
                lemma: Owned(input.to_string()),
                char_end: input.chars().count(),
                byte_end: input.len(),
                script: Script::Thai,
                language: Some(Language::Tha),
                ..Default::default()
            };
            let options =
                NormalizerOption { create_char_map: true, lossy: false, ..Default::default() };
            let normalized = token.normalize(&options);
            assert_eq!(normalized.lemma(), expected, "failed for input {input:?}");
            // char_map must account for all original input characters
            if let Some(char_map) = normalized.char_map {
                let total_orig_chars = char_map.len();
                let total_orig_bytes: usize = char_map.iter().map(|(orig, _)| *orig as usize).sum();
                assert_eq!(
                    (total_orig_chars, total_orig_bytes),
                    (input.chars().count(), input.len()),
                    "char_map sum mismatch for input {input:?}"
                );
            }
        }
    }
}
