use std::sync::LazyLock;

use thaibreak::break_iterator::BreakIterator;
use thaibreak::tokenizer::Tokenizer;
use thaibreak::trie::ThaiTrie;

use crate::normalizer::thai::{needs_normalization, normalize_chars};
// Import `Segmenter` trait.
use crate::segmenter::Segmenter;

/// Thai specialized [`Segmenter`].
///
/// This Segmenter uses [thai-break](https://github.com/kamthorn/thai-break): the text is cut
/// on Thai Character Cluster boundaries, the words are chosen by a weighted Viterbi search on
/// its base dictionary, and the words outside of the dictionary are kept together.
///
/// The base dictionary of thai-break has few long compounds, so the parts of a compound word
/// are segments of their own and can be searched.
///
/// The text is segmented as normalized by the [`ThaiNormalizer`](crate::normalizer::thai::ThaiNormalizer),
/// so that an invisible char or a mark typed twice does not cut a word.
pub struct ThaiSegmenter;

static TOKENIZER: LazyLock<Tokenizer> = LazyLock::new(|| {
    let trie = ThaiTrie::embedded().expect("the dictionary embedded in thai-break is valid");
    Tokenizer::new(trie, None)
});

impl Segmenter for ThaiSegmenter {
    fn segment_str<'o>(&self, to_segment: &'o str) -> Box<dyn Iterator<Item = &'o str> + 'o> {
        let boundaries = if needs_normalization(to_segment) {
            normalized_boundaries(to_segment)
        } else {
            BreakIterator::new(&TOKENIZER, to_segment).boundaries().to_vec()
        };
        Box::new((1..boundaries.len()).map(move |i| &to_segment[boundaries[i - 1]..boundaries[i]]))
    }
}

/// Returns the boundaries of the normalized text, as byte offsets in the original text.
///
/// The chars removed by the normalization belong to the segment that precedes them.
fn normalized_boundaries(text: &str) -> Vec<usize> {
    let mut normalized = String::with_capacity(text.len());
    // for each char of the normalized text, its offset and the offset of its original char.
    let mut offsets = Vec::new();
    let mut original_offset = 0;
    for (len, c) in normalize_chars(text) {
        if let Some(c) = c {
            offsets.push((normalized.len(), original_offset));
            normalized.push(c);
        }
        original_offset += len;
    }

    let boundaries = BreakIterator::new(&TOKENIZER, &normalized).boundaries().to_vec();
    // the boundaries are char boundaries of the normalized text, so they are all found.
    let inner = boundaries.iter().filter(|&&b| b > 0 && b < normalized.len()).map(|&b| {
        let i = offsets.binary_search_by_key(&b, |&(n, _)| n).unwrap_or_else(|i| i);
        offsets.get(i).map_or(text.len(), |&(_, original)| original)
    });
    let mut mapped: Vec<usize> = std::iter::once(0).chain(inner).collect();
    if !text.is_empty() {
        mapped.push(text.len());
    }

    mapped
}

// Test the segmenter:
#[cfg(test)]
mod test {
    use crate::segmenter::test::test_segmenter;

    const TEXT: &str = "ภาษาไทยง่ายนิดเดียว ไก่ขันตอนเช้าบนขันน้ำ ฉันสระผมที่สระน้ำด้วยน้ำยาสระผม 123 456";

    // the compounds are split into words, so that their parts can be searched.
    const SEGMENTED: &[&str] = &[
        "ภาษา",
        "ไทย",
        "ง่าย",
        "นิดเดียว",
        " ",
        "ไก่",
        "ขัน",
        "ตอน",
        "เช้า",
        "บน",
        "ขัน",
        "น้ำ",
        " ",
        "ฉัน",
        "สระ",
        "ผม",
        "ที่",
        "สระ",
        "น้ำ",
        "ด้วย",
        "น้ำยา",
        "สระ",
        "ผม",
        " ",
        "123",
        " ",
        "456",
    ];

    // the normalization keeps the Thai vowels and tone marks.
    const TOKENIZED: &[&str] = SEGMENTED;

    // Macro that run several tests on the Segmenter.
    test_segmenter!(ThaiSegmenter, TEXT, SEGMENTED, TOKENIZED, Script::Thai, Language::Tha);

    fn segment_thai(text: &str) -> Vec<&str> {
        ThaiSegmenter.segment_str(text).collect()
    }

    #[test]
    fn parts_of_compounds_are_segments() {
        assert_eq!(segment_thai("ข้าวมันไก่"), ["ข้าว", "มัน", "ไก่"]);
        assert_eq!(segment_thai("ปัญญาประดิษฐ์"), ["ปัญญา", "ประดิษฐ์"]);
        assert_eq!(segment_thai("คุณภาพชีวิตผู้สูงอายุ"), ["คุณภาพ", "ชีวิต", "ผู้", "สูง", "อายุ"]);
    }

    #[test]
    fn words_of_the_dictionary_are_not_split() {
        for word in ["กรมธรรม์", "มหาวิทยาลัย", "กระทรวง", "นายก"]
        {
            assert_eq!(segment_thai(word), [word]);
        }
    }

    #[test]
    fn vowels_and_tone_marks_are_not_split_from_their_consonant() {
        assert_eq!(segment_thai("ดีดู"), ["ดี", "ดู"]);
        assert_eq!(segment_thai("ไม่มีใครรู้"), ["ไม่", "มี", "ใคร", "รู้"]);
    }

    #[test]
    fn numbers_are_not_split() {
        assert_eq!(segment_thai("ปี2024ราคา๑๒๓บาท"), ["ปี", "2024", "ราคา", "๑๒๓", "บาท"]);
    }

    #[test]
    fn typing_variants_are_segmented_like_the_normalized_text() {
        for (variant, expected) in [
            ("ภูมิ\u{200d}ปัญญา", &["ภูมิ\u{200d}ปัญญา"][..]),
            ("และ\u{200b}แนวทาง", &["และ\u{200b}", "แนวทาง"][..]),
            ("\u{feff}การตีตรา", &["\u{feff}การ", "ตี", "ตรา"][..]),
            ("เพื่\u{ad}อศึกษา", &["เพื่\u{ad}อ", "ศึกษา"][..]),
            ("เพื่่อการเกษตร", &["เพื่่อ", "การ", "เกษตร"][..]),
            ("เเรงจูงใจ", &["เเรง", "จูง", "ใจ"][..]),
            ("ท\u{e48}\u{e35}ดิน", &["ท\u{e48}\u{e35}ดิน"][..]),
        ] {
            assert_eq!(segment_thai(variant), expected, "{variant:?}");
        }

        let tokens: Vec<_> =
            crate::Tokenize::tokenize(&"ภูมิ\u{200d}ปัญญาและ\u{200b}ท\u{e48}\u{e35}ดิน")
                .map(|t| t.lemma().to_string())
                .collect();
        assert_eq!(tokens, ["ภูมิปัญญา", "และ", "ที่ดิน"]);
    }

    #[test]
    fn segments_cover_the_whole_text() {
        for text in [
            "",
            "a",
            "ๆๆ",
            "เ",
            "ั",
            "\u{e4d}",
            "\u{e4d}\u{e49}",
            "ก\u{e4d}\u{e49}",
            "กกกกกกกกกกกกกกกก",
            "ภาษาไทย123abc",
            "\u{200b}",
            "\u{200b}\u{200b}ก",
            "ก\u{200b}",
            "เเ",
            "่ี",
        ] {
            assert_eq!(segment_thai(text).concat(), text);
        }
    }
}
