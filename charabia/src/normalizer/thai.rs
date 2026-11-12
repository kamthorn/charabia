use std::borrow::Cow;

use super::{CharNormalizer, CharOrStr, Normalizer, NormalizerOption};
use crate::{Script, Token};

const NIKHAHIT: char = '\u{e4d}';
const SARA_AA: char = '\u{e32}';
const SARA_AM: char = '\u{e33}';
const SARA_E: char = '\u{e40}';
const SARA_AE: char = '\u{e41}';

/// Thai specialized [`Normalizer`].
///
/// This Normalizer gives the same lemma to the forms of a word that are typed or copied
/// differently but look the same:
/// - Sara Am (ำ, U+0E33), decomposed into Nikhahit (U+0E4D) + Sara Aa (U+0E32) by the
///   [`CompatibilityDecompositionNormalizer`](super::CompatibilityDecompositionNormalizer),
///   or typed as these two characters, with or without a tone mark between them,
///   is recomposed: น้ำ can be typed as น + ้ + ํ + า or as น + ํ + ้ + า.
/// - The invisible characters often found in text copied from PDF or web pages
///   (zero width space, zero width joiner, zero width no-break space, soft hyphen,
///   word joiner) are removed.
/// - A vowel or a tone mark typed twice is kept once (เพื่่อ → เพื่อ).
/// - Two Sara E typed for Sara Ae are replaced by Sara Ae (เเรง → แรง).
/// - A tone mark typed before the vowel above or below the consonant is put after it
///   (ท่ี → ที่).
///
/// The normalization is not lossy, so this Normalizer is not part of the lossy normalizers.
pub struct ThaiNormalizer;

impl Normalizer for ThaiNormalizer {
    fn normalize<'o>(&self, mut token: Token<'o>, options: &NormalizerOption) -> Token<'o> {
        // `Normalize for &str` calls every normalizer without calling `should_normalize`,
        // so return early to avoid allocating when there is nothing to normalize.
        if !needs_normalization(&token.lemma) {
            return token;
        }

        // if no char_map exists, the lemma is the original text.
        let mut char_map = token.char_map.take().or_else(|| {
            options.create_char_map.then(|| {
                token.lemma.chars().map(|c| (c.len_utf8() as u8, c.len_utf8() as u8)).collect()
            })
        });

        token.lemma = Cow::Owned(match char_map.as_deref_mut() {
            Some(char_map) => normalize_with_char_map(&token.lemma, char_map),
            None => normalize_chars(&token.lemma).filter_map(|(_, c)| c).collect(),
        });
        token.char_map = char_map;

        token
    }

    fn should_normalize(&self, token: &Token) -> bool {
        token.script == Script::Thai && needs_normalization(&token.lemma)
    }
}

/// A lossy [`Normalizer`] replacing the Thai digits (๐ to ๙) by the ASCII digits,
/// so that a year written ๒๕๖๗ matches 2567.
pub struct ThaiDigitNormalizer;

impl CharNormalizer for ThaiDigitNormalizer {
    fn normalize_char(&self, c: char) -> Option<CharOrStr> {
        match c {
            '\u{e50}'..='\u{e59}' => Some(char::from(b'0' + (c as u32 - 0xe50) as u8).into()),
            c => Some(c.into()),
        }
    }

    fn should_normalize(&self, token: &Token) -> bool {
        token.lemma().chars().any(|c| matches!(c, '\u{e50}'..='\u{e59}'))
    }
}

/// Returns `true` if the normalization changes the text, see [`ThaiNormalizer`].
pub(crate) fn needs_normalization(text: &str) -> bool {
    text.chars().zip(normalize_chars(text)).any(|(c, (_, normalized))| normalized != Some(c))
}

/// Returns `true` if the char is invisible and is removed by the normalization.
fn is_invisible(c: char) -> bool {
    matches!(c, '\u{200b}' | '\u{200d}' | '\u{feff}' | '\u{ad}' | '\u{2060}')
}

/// Returns `true` if the char is a Thai vowel above or below the consonant, a tone mark,
/// or another Thai combining mark.
fn is_combining_mark(c: char) -> bool {
    matches!(c, '\u{e31}' | '\u{e34}'..='\u{e3a}' | '\u{e47}'..='\u{e4e}')
}

fn is_tone_mark(c: char) -> bool {
    matches!(c, '\u{e48}'..='\u{e4b}')
}

/// Returns `true` if the char is a vowel that is written before the tone mark.
fn is_vowel_before_tone_mark(c: char) -> bool {
    matches!(c, '\u{e31}' | '\u{e34}'..='\u{e39}')
}

/// Returns `true` if the text following a Nikhahit starts with Sara Aa,
/// or with a tone mark and Sara Aa.
fn completes_sara_am(rest: &str) -> bool {
    let mut chars = rest.chars();
    match chars.next() {
        Some(SARA_AA) => true,
        Some(c) if is_tone_mark(c) => chars.next() == Some(SARA_AA),
        _ => false,
    }
}

/// Normalizes the text, see [`ThaiNormalizer`].
///
/// Yields, for each char of the text, its length and the char replacing it, if any.
pub(crate) fn normalize_chars(text: &str) -> impl Iterator<Item = (usize, Option<char>)> + '_ {
    let mut chars = text.chars();
    let mut after_nikhahit = false;
    // what replaces the next char, when it is known from the current one.
    let mut next_replacement: Option<Option<char>> = None;
    let mut previous = None;
    std::iter::from_fn(move || {
        let c = chars.next()?;
        let following = chars.clone().next();
        let normalized = match next_replacement.take() {
            Some(replacement) => replacement,
            None => match c {
                c if is_invisible(c) => None,
                NIKHAHIT if completes_sara_am(chars.as_str()) => {
                    after_nikhahit = true;
                    None
                }
                SARA_AA if after_nikhahit => {
                    after_nikhahit = false;
                    Some(SARA_AM)
                }
                SARA_E if following == Some(SARA_E) => {
                    next_replacement = Some(None);
                    Some(SARA_AE)
                }
                c if is_tone_mark(c) && following.is_some_and(is_vowel_before_tone_mark) => {
                    next_replacement = Some(Some(c));
                    following
                }
                c if is_combining_mark(c) && previous == Some(c) => None,
                c => Some(c),
            },
        };
        if normalized.is_some() {
            previous = normalized;
        }

        Some((c.len_utf8(), normalized))
    })
}

/// Normalizes the lemma, see [`normalize_chars`], recomputing the normalized lengths
/// of the `char_map`.
///
/// Nikhahit and Sara Aa are in the same `char_map` entry when an original Sara Am has been
/// decomposed, and in two entries when the original text contains the two characters.
fn normalize_with_char_map(lemma: &str, char_map: &mut [(u8, u8)]) -> String {
    let mut normalized = String::with_capacity(lemma.len());
    let mut chars = normalize_chars(lemma);
    for (_, normalized_len) in char_map.iter_mut() {
        let start = normalized.len();
        let mut remaining = *normalized_len as usize;
        while remaining > 0 {
            let Some((len, c)) = chars.next() else { break };
            remaining = remaining.saturating_sub(len);
            normalized.extend(c);
        }
        *normalized_len = (normalized.len() - start) as u8;
    }

    normalized
}

#[cfg(test)]
mod test {
    use std::borrow::Cow::{Borrowed, Owned};

    use super::*;
    use crate::normalizer::Normalize;

    fn thai_token(lemma: &str) -> Token<'_> {
        Token {
            lemma: Borrowed(lemma),
            char_end: lemma.chars().count(),
            byte_end: lemma.len(),
            script: Script::Thai,
            ..Default::default()
        }
    }

    fn options(create_char_map: bool) -> NormalizerOption<'static> {
        NormalizerOption { create_char_map, ..Default::default() }
    }

    #[test]
    fn decomposed_sara_am_is_recomposed() {
        for (decomposed, expected) in [
            ("น\u{e49}\u{e4d}\u{e32}", "น้ำ"),
            ("ท\u{e4d}\u{e32}งาน", "ทำงาน"),
            ("น\u{e49}\u{e4d}\u{e32}ท\u{e4d}\u{e32}", "น้ำทำ"),
            // the tone mark is typed after the Nikhahit.
            ("น\u{e4d}\u{e49}\u{e32}", "น้ำ"),
            ("ต\u{e4d}\u{e48}\u{e32}", "ต่ำ"),
            ("น\u{e4d}\u{e49}\u{e32}ตาล", "น้ำตาล"),
        ] {
            for create_char_map in [false, true] {
                let token = thai_token(decomposed);
                assert!(ThaiNormalizer.should_normalize(&token));
                let token = ThaiNormalizer.normalize(token, &options(create_char_map));
                assert_eq!(token.lemma(), expected);
                assert_eq!(token.char_map.is_some(), create_char_map);
            }
        }
    }

    #[test]
    fn text_without_decomposed_sara_am_is_not_reallocated() {
        // a Nikhahit that is not followed by Sara Aa is kept.
        for lemma in
            ["น้ำ", "วิทยุ", "\u{e4d}", "น\u{e4d}", "น\u{e4d}\u{e49}", "น\u{e4d}\u{e49}ย", "hello"]
        {
            let token = thai_token(lemma);
            assert!(!ThaiNormalizer.should_normalize(&token));
            let token = ThaiNormalizer.normalize(token, &options(true));
            assert!(matches!(token.lemma, Borrowed(_)), "{lemma} must not be reallocated");
            assert_eq!(token.char_map, None);
        }
    }

    #[test]
    fn char_map_of_sara_am_typed_as_two_chars() {
        // ทํางาน: Nikhahit and Sara Aa are two original chars.
        let typed = "ท\u{e4d}\u{e32}งาน";
        let expected = vec![(3, 3), (3, 0), (3, 3), (3, 3), (3, 3), (3, 3)];

        // the char_map is created.
        let token = ThaiNormalizer.normalize(thai_token(typed), &options(true));
        assert_eq!(token.lemma(), "ทำงาน");
        assert_eq!(token.char_map, Some(expected.clone()));
        assert_eq!(token.original_lengths(token.lemma().len()), (6, 18));

        // the char_map already exists.
        let token = Token { char_map: Some(vec![(3, 3); 6]), ..thai_token(typed) };
        let token = ThaiNormalizer.normalize(token, &options(true));
        assert_eq!(token.lemma(), "ทำงาน");
        assert_eq!(token.char_map, Some(expected));
    }

    #[test]
    fn char_map_of_sara_am_typed_with_a_tone_mark_after_nikhahit() {
        // นํ้า: the tone mark keeps its own entry.
        let token = ThaiNormalizer.normalize(thai_token("น\u{e4d}\u{e49}\u{e32}"), &options(true));
        assert_eq!(token.lemma(), "น้ำ");
        assert_eq!(token.char_map, Some(vec![(3, 3), (3, 0), (3, 3), (3, 3)]));
        assert_eq!(token.original_lengths(token.lemma().len()), (4, 12));
    }

    #[test]
    fn char_map_of_sara_am_decomposed_by_a_previous_normalizer() {
        // น้ำ: the original Sara Am (3 bytes) has been decomposed into 6 bytes.
        let token = Token {
            lemma: Owned("น\u{e49}\u{e4d}\u{e32}".to_string()),
            char_map: Some(vec![(3, 3), (3, 3), (3, 6)]),
            ..thai_token("น้ำ")
        };
        let token = ThaiNormalizer.normalize(token, &options(true));
        assert_eq!(token.lemma(), "น้ำ");
        assert_eq!(token.char_map, Some(vec![(3, 3), (3, 3), (3, 3)]));
    }

    #[test]
    fn vowels_and_tone_marks_are_kept_by_the_lossy_normalization() {
        let options = NormalizerOption { lossy: true, ..Default::default() };
        for word in ["วิทยุ", "ธาตุ", "มนุษย์", "ง่าย", "น้ำ"]
        {
            assert_eq!(thai_token(word).normalize(&options).lemma(), word);
        }
    }

    #[test]
    fn sara_am_is_kept_by_the_non_lossy_normalization() {
        for create_char_map in [false, true] {
            for (word, expected) in [
                ("น้ำ", "น้ำ"),
                ("ทำงาน", "ทำงาน"),
                ("น\u{e49}\u{e4d}\u{e32}", "น้ำ"),
                ("น\u{e4d}\u{e49}\u{e32}", "น้ำ"),
            ] {
                let token = thai_token(word).normalize(&options(create_char_map));
                assert_eq!(token.lemma(), expected);
                if create_char_map {
                    assert_eq!(
                        token.original_lengths(token.lemma().len()),
                        (word.chars().count(), word.len())
                    );
                }
            }
        }
    }

    #[test]
    fn typing_variants_are_normalized() {
        for (variant, expected) in [
            ("ภูมิ\u{200d}ปัญญา", "ภูมิปัญญา"),
            ("และ\u{200b}แนวทาง", "และแนวทาง"),
            ("\u{feff}การ", "การ"),
            ("เพื่\u{ad}อ", "เพื่อ"),
            ("0\u{2060}.\u{2060}9", "0.9"),
            ("เพื่่อ", "เพื่อ"),
            ("คุุณภาพ", "คุณภาพ"),
            ("เกณฑ์์", "เกณฑ์"),
            ("เเรง", "แรง"),
            ("ท\u{e48}\u{e35}ดิน", "ที่ดิน"),
            ("ก\u{e48}\u{e38}ม", "กุ่ม"),
            ("ก\u{e49}\u{e31}น", "กั้น"),
            ("น\u{e4d}\u{e49}\u{e32}ท\u{e48}\u{e35}", "น้ำที่"),
        ] {
            for create_char_map in [false, true] {
                let token =
                    ThaiNormalizer.normalize(thai_token(variant), &options(create_char_map));
                assert_eq!(token.lemma(), expected, "{variant:?}");
                if create_char_map {
                    // every original char has an entry, the removed ones are mapped to 0 bytes.
                    let char_map = token.char_map.unwrap();
                    assert_eq!(char_map.len(), variant.chars().count(), "{variant:?}");
                    let original: usize = char_map.iter().map(|&(o, _)| o as usize).sum();
                    let normalized: usize = char_map.iter().map(|&(_, n)| n as usize).sum();
                    assert_eq!(
                        (original, normalized),
                        (variant.len(), expected.len()),
                        "{variant:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn correct_text_is_not_reallocated() {
        for lemma in ["ที่ดิน", "เพื่อ", "แรง", "ภูมิปัญญา", "กุ่ม", "กั้น", "เลข ๑๒๓", "ปฏิเสธ"]
        {
            let token = ThaiNormalizer.normalize(thai_token(lemma), &options(true));
            assert!(matches!(token.lemma, Borrowed(_)), "{lemma} must not be reallocated");
        }
    }

    #[test]
    fn char_map_of_a_swapped_tone_mark() {
        // ท่ี: the tone mark and the vowel keep their own entries.
        let token = ThaiNormalizer.normalize(thai_token("ท\u{e48}\u{e35}"), &options(true));
        assert_eq!(token.lemma(), "ที่");
        assert_eq!(token.char_map, Some(vec![(3, 3), (3, 3), (3, 3)]));
    }

    #[test]
    fn thai_digits_are_replaced_by_ascii_digits_when_lossy() {
        let lossy = NormalizerOption { lossy: true, ..Default::default() };
        for (text, expected) in [("๒๕๖๗", "2567"), ("1,1๐๐", "1,100"), ("พ.ศ.๒๕๖๗", "พ.ศ.2567")]
        {
            assert_eq!(text.normalize(&lossy), expected);
        }
        let non_lossy = NormalizerOption { lossy: false, ..Default::default() };
        assert_eq!("๒๕๖๗".normalize(&non_lossy), "๒๕๖๗");

        let tokens: Vec<_> = crate::Tokenize::tokenize(&"ปี ๒๕๖๗ และ 2567")
            .filter(|t| t.is_word())
            .map(|t| t.lemma().to_string())
            .collect();
        assert_eq!(tokens, ["ปี", "2567", "และ", "2567"]);
    }
}
