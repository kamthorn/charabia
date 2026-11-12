use std::sync::LazyLock;

use thaibreak::{MatchOptions, ThaiTrie, Tokenizer};

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
/// The match options of thai-break let a dictionary word match through the typing variants that
/// the `ThaiNormalizer` removes (an invisible char, a
/// mark typed twice, a tone mark typed before the vowel), so that they do not cut a word; the
/// segments are still slices of the original text.
pub struct ThaiSegmenter;

static TOKENIZER: LazyLock<Tokenizer> = LazyLock::new(|| {
    let mut trie = ThaiTrie::embedded().expect("the dictionary embedded in thai-break is valid");
    trie.set_compact_lookup(true);
    let mut tokenizer = Tokenizer::new(trie, None);
    tokenizer.set_match_options(MatchOptions {
        ignore_invisible: true,
        collapse_repeated_marks: true,
        reorder_tone_marks: true,
    });
    tokenizer
});

impl Segmenter for ThaiSegmenter {
    fn segment_str<'o>(&self, to_segment: &'o str) -> Box<dyn Iterator<Item = &'o str> + 'o> {
        let mut boundaries = TOKENIZER.boundaries(to_segment);
        // the abbreviation mark ฯ ends the word before it, which the
        // ThaiAbbreviationNormalizer then removes from the lemma (เกล้าฯ, not เกล้า and ฯ).
        let mut i = 1;
        while i + 1 < boundaries.len() {
            let (before, mark) = (
                &to_segment[boundaries[i - 1]..boundaries[i]],
                &to_segment[boundaries[i]..boundaries[i + 1]],
            );
            if mark == "ฯ" && !before.trim().is_empty() {
                boundaries.remove(i);
            } else {
                i += 1;
            }
        }
        Box::new((1..boundaries.len()).map(move |i| &to_segment[boundaries[i - 1]..boundaries[i]]))
    }
}

/// [`Segmenter`] for the Thai names of people, used for the Thai text with the Khmer locale:
/// the text is only cut by the separators, so that a name is a single word, and the prefix
/// search and the typo tolerance apply to the whole name.
pub struct ThaiNameSegmenter;

impl Segmenter for ThaiNameSegmenter {
    fn segment_str<'o>(&self, to_segment: &'o str) -> Box<dyn Iterator<Item = &'o str> + 'o> {
        Box::new(std::iter::once(to_segment))
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

    fn words(text: &str) -> Vec<String> {
        crate::Tokenize::tokenize(&text)
            .filter(|t| t.is_word())
            .map(|t| t.lemma().to_string())
            .collect()
    }

    #[test]
    fn the_repetition_mark_is_a_separator() {
        assert_eq!(words("สิ่งต่างๆ"), ["สิ่ง", "ต่าง"]);
        assert_eq!(words("เด็ก ๆ ชอบ"), ["เด็ก", "ชอบ"]);
    }

    #[test]
    fn the_abbreviation_mark_is_removed_from_the_word() {
        assert_eq!(words("กรุงเทพฯ"), ["กรุงเทพ"]);
        assert_eq!(words("ทูลเกล้าฯ ถวาย"), ["ทูล", "เกล้า", "ถวาย"]);
        assert_eq!(words("ผลไม้ ฯลฯ"), ["ผลไม้", "ฯลฯ"]);
    }

    #[test]
    fn the_full_stop_of_a_thai_abbreviation_is_a_soft_separator() {
        use crate::{SeparatorKind, TokenKind};
        let hard = |text: &str| {
            crate::Tokenize::tokenize(&text)
                .filter(|t| t.kind == TokenKind::Separator(SeparatorKind::Hard))
                .count()
        };
        assert_eq!(hard("พ.ศ. ๒๕๖๗"), 0);
        assert_eq!(hard("ผศ.ดร. สมชาย"), 0);
        assert_eq!(hard("กทม. และ"), 0);
        // the end of a sentence stays a hard separator.
        assert_eq!(hard("คนไทย. ต่อไป"), 1);
        assert_eq!(hard("ข้อ 1. การวิจัย"), 1);
        assert_eq!(hard("end. Next"), 1);
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

    #[test]
    fn thai_names_are_not_segmented_with_the_khmer_locale() {
        use crate::{Language, TokenizerBuilder};

        let mut builder = TokenizerBuilder::default();
        let allow_list = [Language::Khm];
        builder.allow_list(&allow_list);
        let tokenizer = builder.build();
        let words: Vec<_> = tokenizer
            .tokenize("วัลลภา มูลเมือง")
            .filter(|t| t.is_word())
            .map(|t| t.lemma().to_string())
            .collect();
        assert_eq!(words, ["วัลลภา", "มูลเมือง"]);

        // the other locales and the detected language still segment the text.
        for allow_list in [&[Language::Eng][..], &[Language::Tha][..], &[][..]] {
            let mut builder = TokenizerBuilder::default();
            if !allow_list.is_empty() {
                builder.allow_list(allow_list);
            }
            let tokenizer = builder.build();
            let words: Vec<_> = tokenizer
                .tokenize("ข้าวมันไก่")
                .filter(|t| t.is_word())
                .map(|t| t.lemma().to_string())
                .collect();
            assert_eq!(words, ["ข้าว", "มัน", "ไก่"], "{allow_list:?}");
        }
    }
}
