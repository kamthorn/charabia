use std::sync::LazyLock;

use thaibreak::break_iterator::BreakIterator;
use thaibreak::tokenizer::Tokenizer;
use thaibreak::trie::ThaiTrie;

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
pub struct ThaiSegmenter;

static TOKENIZER: LazyLock<Tokenizer> = LazyLock::new(|| {
    let trie = ThaiTrie::embedded().expect("the dictionary embedded in thai-break is valid");
    Tokenizer::new(trie, None)
});

impl Segmenter for ThaiSegmenter {
    fn segment_str<'o>(&self, to_segment: &'o str) -> Box<dyn Iterator<Item = &'o str> + 'o> {
        let boundaries = BreakIterator::new(&TOKENIZER, to_segment).boundaries().to_vec();
        Box::new((1..boundaries.len()).map(move |i| &to_segment[boundaries[i - 1]..boundaries[i]]))
    }
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
        ] {
            assert_eq!(segment_thai(text).concat(), text);
        }
    }
}
