// Generated from thai-tcc r2 specification. DO NOT EDIT.

pub struct TccDfa {
    pub classes: usize,
    pub start: u16,
    pub class: [u8; 128],
    pub trans: &'static [u16],
    pub is_match: &'static [bool],
}

impl TccDfa {
    #[inline(always)]
    pub fn find(&self, chars: &[char], start: usize) -> Option<usize> {
        let mut state = self.start;
        let mut found = None;
        for (pos, &ch) in chars.iter().enumerate().skip(start) {
            let off = (ch as u32).wrapping_sub(0x0E00);
            if off >= 128 {
                break;
            }
            state = self.trans[state as usize * self.classes + self.class[off as usize] as usize];
            if state == 0 {
                break;
            }
            if self.is_match[state as usize] {
                found = Some(pos + 1);
            }
        }
        found
    }
}

pub static GENERAL_DFA: TccDfa = TccDfa {
    classes: 30,
    start: 1,
    class: [
        0, 1, 2, 3, 2, 3, 3, 4, 3, 3, 3, 3, 3, 4, 3, 3, 3, 3, 3, 3, 5, 5, 3, 6, 3, 4, 5, 5, 5, 3,
        5, 3, 3, 4, 7, 8, 3, 9, 3, 10, 3, 3, 3, 11, 3, 12, 3, 0, 13, 14, 15, 16, 17, 18, 19, 20,
        21, 22, 0, 0, 0, 0, 0, 0, 23, 24, 25, 26, 26, 0, 0, 27, 28, 28, 28, 28, 29, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    trans: &[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 2, 3, 3, 3, 3, 3, 3, 4, 3, 3, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 6, 7, 8, 0, 0, 0,
        0, 9, 9, 9, 9, 9, 9, 9, 10, 9, 9, 9, 9, 11, 12, 11, 11, 11, 11, 13, 13, 11, 11, 0, 0, 0, 0,
        14, 15, 0, 0, 9, 9, 9, 9, 9, 9, 9, 10, 9, 9, 9, 9, 11, 12, 11, 11, 11, 11, 13, 13, 11, 11,
        0, 0, 0, 0, 16, 15, 0, 0, 9, 9, 9, 9, 9, 9, 9, 10, 9, 9, 9, 9, 11, 12, 11, 11, 11, 11, 17,
        13, 11, 11, 0, 0, 0, 0, 16, 15, 0, 0, 18, 18, 19, 19, 20, 21, 19, 19, 19, 19, 22, 19, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 24,
        23, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 25, 25, 25, 25, 25, 25, 25, 25,
        25, 25, 25, 25, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 11, 11, 11, 11,
        11, 11, 11, 11, 11, 11, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 26, 26,
        26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 0, 0, 0, 0, 27, 0, 0, 0, 27, 27, 0, 0, 0, 0, 0, 0,
        14, 0, 26, 26, 26, 26, 26, 26, 26, 28, 26, 26, 26, 26, 0, 0, 0, 0, 27, 0, 0, 0, 27, 27, 0,
        0, 0, 0, 0, 0, 14, 0, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 29, 0, 0, 30, 30, 30, 30, 30, 30, 30, 30, 30, 31, 30, 30, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 29, 0, 29,
        29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 34, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14, 0, 0, 35, 35, 35, 35, 35, 35, 35, 36, 36, 36, 35,
        35, 29, 0, 37, 0, 38, 39, 0, 40, 0, 0, 0, 0, 0, 0, 34, 41, 0, 0, 35, 35, 35, 35, 35, 35,
        35, 42, 42, 42, 35, 35, 29, 0, 37, 0, 38, 39, 0, 40, 0, 0, 0, 0, 0, 0, 34, 41, 0, 0, 35,
        35, 35, 35, 35, 35, 35, 36, 36, 42, 35, 35, 29, 0, 37, 0, 38, 39, 0, 40, 0, 0, 0, 0, 0, 0,
        34, 41, 0, 0, 35, 35, 35, 35, 35, 35, 35, 36, 42, 42, 35, 35, 29, 0, 37, 0, 38, 39, 0, 40,
        0, 0, 0, 0, 0, 0, 34, 41, 0, 0, 42, 42, 42, 36, 42, 42, 36, 36, 36, 36, 42, 42, 29, 0, 37,
        0, 38, 39, 0, 40, 0, 0, 0, 0, 0, 0, 34, 41, 0, 0, 43, 43, 43, 43, 43, 43, 43, 44, 44, 44,
        43, 43, 29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 34, 37, 0, 0, 44, 44, 44, 44, 44, 44,
        44, 44, 44, 44, 44, 44, 29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 34, 37, 0, 0, 9, 9, 9,
        9, 9, 9, 9, 9, 9, 9, 9, 9, 29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 37, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 27, 0, 0, 0, 27, 27, 0, 0, 0, 0, 0, 0, 14, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14, 0, 27,
        27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 27, 0, 0, 0, 0, 27, 0, 0, 0, 27, 27, 0, 0, 0, 0, 0,
        0, 14, 0, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 29, 0, 0, 0, 29, 0, 0, 0, 29, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 14, 0, 0, 0, 29, 0, 0, 0, 29, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 30, 30, 30, 30, 30, 30, 30, 30, 30, 31, 30, 30, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26,
        26, 26, 0, 0, 45, 0, 27, 46, 0, 0, 27, 27, 0, 0, 0, 0, 0, 47, 14, 0, 26, 26, 26, 26, 26,
        26, 26, 26, 26, 26, 26, 26, 0, 0, 37, 0, 48, 46, 0, 49, 27, 27, 0, 0, 0, 0, 34, 50, 14, 0,
        9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        51, 51, 51, 51, 51, 51, 51, 51, 51, 51, 51, 51, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 34, 0, 0, 0, 0, 0, 0, 0, 0, 37, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 52, 0, 0, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 37, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 53, 0, 0, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 29, 0, 37, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 26, 0, 0, 45, 0, 27, 46, 0, 0,
        27, 27, 0, 0, 0, 0, 34, 47, 14, 0, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 0, 0, 0,
        0, 27, 0, 0, 0, 27, 27, 0, 0, 0, 0, 0, 0, 29, 0, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54,
        54, 54, 0, 0, 0, 0, 27, 0, 0, 0, 27, 27, 0, 0, 0, 0, 34, 0, 29, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 45, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 45, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 29, 29, 29, 29, 29, 29,
        29, 29, 29, 29, 29, 29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 34, 14, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 37, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 37, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 9, 9, 9,
        9, 9, 9, 9, 9, 9, 9, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 34, 0, 0, 0, 0, 0,
        0, 0, 37, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 9, 9, 9,
        9, 9, 9, 9, 9, 9, 9, 37, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 27, 0, 0, 0, 27, 27, 0, 0, 0, 0, 0, 0, 29, 0, 0, 0, 0,
        0, 0, 0, 45, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 37, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    is_match: &[
        false, false, true, true, true, false, false, false, false, false, false, true, true, true,
        true, true, true, true, true, true, true, true, true, true, true, true, false, false,
        false, true, true, true, true, true, false, false, false, true, false, false, true, true,
        false, false, false, false, false, false, false, false, false, true, false, true, false,
        false, false,
    ],
};

pub static LOOKAHEAD_DFA: TccDfa = TccDfa {
    classes: 8,
    start: 1,
    class: [
        0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 3, 4, 0, 0, 3, 3, 0, 0,
        0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 6, 6, 6, 6, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ],
    trans: &[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 3, 3, 0, 0, 0, 0, 0, 0, 4, 4, 5, 5, 0,
        0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 8, 8, 0, 0, 0, 0, 0, 0, 0, 6, 0,
        0, 0, 0, 0, 0, 9, 9, 10, 0, 0, 0, 11, 0, 0, 0, 10, 0, 0, 0, 11, 0, 0, 0, 0, 0, 0, 0, 11, 0,
        0, 0, 0, 0, 0, 0, 0,
    ],
    is_match: &[false, false, false, false, false, false, true, false, false, false, false, true],
};

#[inline]
fn is_digit(ch: char) -> bool {
    ch.is_ascii_digit() || ('\u{0e50}'..='\u{0e59}').contains(&ch)
}

#[inline]
fn is_thai_consonant(ch: char) -> bool {
    ('\u{0e01}'..='\u{0e2e}').contains(&ch)
}

#[inline]
fn is_leading_vowel(ch: char) -> bool {
    ('\u{0e40}'..='\u{0e44}').contains(&ch)
}

/// End of a matched cluster `chars[start..end]`. A final consonant followed by a dependent vowel or mark
/// starts the next cluster instead: "รึยัง" is "รึ" + "ยัง", not "รึย" + "ัง". Except ว before ะ, which is
/// part of the vowel -ัวะ ("ผัวะ").
#[inline]
fn cluster_end(chars: &[(usize, char)], start: usize, end: usize) -> usize {
    if let Some(&(_, next)) = chars.get(end) {
        let last = chars[end - 1].1;
        if end - start >= 2
            && is_thai_consonant(last)
            && is_dependent_thai(next)
            && !(last == '\u{0e27}' && next == '\u{0e30}')
        {
            return end - 1;
        }
    }
    end
}

/// Whether a character can never start a cluster: ะ ั า ำ ิ–ฺ ๅ ็–๎.
#[inline]
pub fn is_dependent_thai(ch: char) -> bool {
    matches!(ch as u32, 0x0E30..=0x0E3A | 0x0E45 | 0x0E47..=0x0E4E)
}

/// The lookahead rules end a cluster only where the next character can start a syllable: a consonant, a
/// leading vowel, the end of the text, or a character outside the Thai block.
#[inline]
fn is_followed_by_lookahead_char(next: Option<char>) -> bool {
    match next {
        None => true,
        Some(ch) => {
            let cp = ch as u32;
            (0x0E01..=0x0E2E).contains(&cp)
                || (0x0E40..=0x0E44).contains(&cp)
                || !(0x0E00..=0x0E7F).contains(&cp)
        }
    }
}

/// Compute valid TCC break positions for a slice of `(byte_offset, char)`.
///
/// Returns a boolean vector of size `chars.len() + 1` where `true` indicates
/// a safe character cluster boundary.
pub fn tcc_pos_array(
    chars: &[(usize, char)],
    is_zero_width_fn: impl Fn(char) -> bool,
) -> Vec<bool> {
    let n = chars.len();
    let mut valid = vec![false; n + 1];
    valid[0] = true;
    valid[n] = true;

    let char_slice: Vec<char> = chars.iter().map(|&(_, c)| c).collect();
    let mut pos = 0;
    while pos < n {
        // Lookahead rules (starting with เ)
        if char_slice[pos] == '\u{0e40}' {
            if let Some(end) = LOOKAHEAD_DFA.find(&char_slice, pos) {
                if is_followed_by_lookahead_char(char_slice.get(end).copied()) {
                    pos = end;
                    valid[pos] = true;
                    continue;
                }
            }
        }

        // General DFA matching
        pos = match GENERAL_DFA.find(&char_slice, pos) {
            Some(end) => cluster_end(chars, pos, end),
            None => pos + 1,
        };
        valid[pos] = true;
    }

    for i in 0..n {
        let ch = chars[i].1;
        if is_zero_width_fn(ch) {
            valid[i] = true;
            valid[i + 1] = true;
        } else if !(0x0E00..=0x0E7F).contains(&(ch as u32)) {
            // Non-Thai characters are always boundaries on both sides
            valid[i] = true;
            valid[i + 1] = true;
        } else if i > 0 && is_dependent_thai(ch) && !(ch == '\u{0E4D}' && is_digit(chars[i - 1].1))
        {
            // Never a boundary before a dependent vowel or mark (except degree sign 90ํ)
            valid[i] = false;
        }
    }

    // Do not split consecutive digits
    for i in 1..n {
        if chars[i - 1].1.is_numeric() && chars[i].1.is_numeric() {
            valid[i] = false;
        }
    }

    // A consonant with a tone mark and without vowel is followed by its vowel or its
    // final consonant (e.g. ก่อน, ก้น, ช้อป), unlike after a leading vowel (e.g. ไม่|มี, ไหว้|วัน).
    let char_at = |idx: usize| chars.get(idx).map(|(_, c)| *c);
    for i in 1..n {
        let previous = chars[i - 1].1;
        let current = chars[i].1;
        if matches!(previous, '\u{e48}'..='\u{e4b}') && is_thai_consonant(current) {
            let before = |n: usize| i.checked_sub(n).and_then(char_at);
            let after_toned_consonant = before(2).is_some_and(is_thai_consonant)
                && !before(3).is_some_and(is_leading_vowel)
                && !(before(3).is_some_and(is_thai_consonant)
                    && before(4).is_some_and(is_leading_vowel));
            if after_toned_consonant {
                valid[i] = false;
            }
        }
    }

    valid
}
