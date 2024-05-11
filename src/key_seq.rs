use crate::layout::{self, Point};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyPressPattern {
    Sequential(Point),
    Shift(Point, Point),
}

/// ある文字を入力する際にキーを押下する順序を表す
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeySeq {
    // 入力する文字
    char: char,
    // キーを押下する順序。
    sequence: Vec<KeyPressPattern>,
}

impl KeySeq {
    /// シフトを表す新しいKeySeqを生成する
    ///
    /// # Arguments
    /// * `char` - 入力する文字
    /// * `key_pos` - キーを押下する順序
    ///
    /// # Returns
    /// 新しいKeySeq
    pub fn from_shift(char: char, key_pos: &Point) -> Self {
        let layout = layout::linear::linear_layout();
        let shift_key = match layout::linear::get_hand_of_point(key_pos) {
            layout::Hand::Right => layout[layout::linear::LINEAR_L_SHIFT_INDEX],
            layout::Hand::Left => layout[layout::linear::LINEAR_R_SHIFT_INDEX],
        };

        KeySeq {
            char,
            sequence: vec![KeyPressPattern::Shift(shift_key, *key_pos)],
        }
    }

    /// シフトを表す新しいKeySeqを生成する
    ///
    /// # Arguments
    /// * `char` - 入力する文字
    /// * `key_pos` - キーを押下する順序
    ///
    /// # Returns
    /// 新しいKeySeq
    pub fn from_shift_like(char: char, key_pos: &Point, shift_pos: &Point) -> Self {
        KeySeq {
            char,
            sequence: vec![KeyPressPattern::Shift(*shift_pos, *key_pos)],
        }
    }

    /// 単打を表す新しいKeySeqを生成する
    ///
    /// # Arguments
    /// * `char` - 入力する文字
    /// * `key_pos` - キーを押下する順序
    ///
    /// # Returns
    /// 新しいKeySeq
    pub fn from_unshift(char: char, key_pos: &Point) -> Self {
        KeySeq {
            char,
            sequence: vec![KeyPressPattern::Sequential(*key_pos)],
        }
    }

    /// 濁音または半濁音を表す新しいKeySeqを生成する。
    ///
    /// # Arguments
    /// * `char` - 入力する文字
    /// * `key_pos` - キーを押下する順序
    /// * `shift_seq` - 濁音自体を表すKeySeq
    ///
    /// # Returns
    /// 新しいKeySeq
    pub fn from_turbid_like(char: char, key_pos: &Point, shift_seq: &KeySeq) -> KeySeq {
        let mut seq: Vec<KeyPressPattern> = vec![];
        seq.extend(shift_seq.sequence.iter().cloned());
        seq.push(KeyPressPattern::Sequential(*key_pos));

        KeySeq {
            char,
            sequence: seq,
        }
    }

    /// key sequenceを文字列に変換する
    pub fn to_char_sequence(&self) -> String {
        let mut s = String::new();

        for seq in &self.sequence {
            match seq {
                KeyPressPattern::Sequential(p) => {
                    s.push_str(&format!("{}", layout::linear::get_char_of_point(p)));
                }
                KeyPressPattern::Shift(shift_p, p) => {
                    s.push_str(&format!(
                        "{}{}",
                        layout::linear::get_char_of_point(shift_p),
                        layout::linear::get_char_of_point(p)
                    ));
                }
            }
        }
        s
    }

    /// `char` を返す
    pub fn char(&self) -> char {
        self.char
    }

    /// `sequence` を返す
    pub fn as_raw_sequence(&self) -> Vec<(Point, Option<Point>)> {
        self.sequence
            .iter()
            .map(|seq| match seq {
                KeyPressPattern::Shift(f, s) => (*s, Some(*f)),
                KeyPressPattern::Sequential(p) => (*p, None),
            })
            .collect()
    }
}
