use std::collections::HashMap;

use crate::{
    char_def::CHARS,
    keymap::{
        KeyKind, Keymap, LEFT_SEMITURBID_INDEX, LEFT_SHIFT_INDEX, LEFT_TURBID_INDEX,
        RIGHT_SEMITURBID_INDEX, RIGHT_SHIFT_INDEX, RIGHT_TURBID_INDEX,
    },
};

/// 各指が担当するキーに対する重み。
#[rustfmt::skip]
static FINGER_WEIGHTS: [[u64; 10];3] = [
    [10000, 40,  20,  30, 10000, 10000, 30, 20, 40, 10000],
    [80,    60,  30,  10,    30,    30, 10, 30, 60, 80],
    [100,   60,  40,  20,    60,    60, 20, 40, 60, 100],
];

/// キーを押下する手の割当。1 = 左手、2 = 右手
static HAND_ASSIGNMENT: [[u8; 10]; 3] = [
    [1, 1, 1, 1, 1, 2, 2, 2, 2, 2],
    [1, 1, 1, 1, 1, 2, 2, 2, 2, 2],
    [1, 1, 1, 1, 1, 2, 2, 2, 2, 2],
];

#[derive(Debug)]
pub struct Conjunction {
    /// 連接のテキスト
    pub text: String,
    /// 連接の出現回数
    pub appearances: u32,
}

/// 連接に対して特殊な評価を行う。
///
/// # Arguments
/// * `text` - 評価対象の文字列
/// * `keymap` - キーマップ
///
/// # Returns
/// 評価値
fn special_evaluations(text: &Conjunction, keymap: &Keymap) -> u64 {
    let mut score = 0;

    score
}

/// [keymap]の評価を行う。scoreは低いほど良好であるとする。
///
/// # Arguments
/// * `texts` - 評価対象の文字列
/// * `keymap` - キーマップ
///
/// # Returns
/// 評価値
pub fn evaluate(conjunctions: &Vec<Conjunction>, keymap: &Keymap) -> u64 {
    let mut score = 0;

    let mut pos_cache: HashMap<char, (KeyKind, (usize, usize))> = HashMap::new();

    for c in CHARS.iter() {
        if let Some(v) = keymap.get(*c) {
            pos_cache.insert(*c, v.clone());
        }
    }

    for conjunction in conjunctions {
        for c in conjunction.text.chars() {
            if let Some((k, (r, c))) = pos_cache.get(&c) {
                score += FINGER_WEIGHTS[*r][*c];

                // 対象の文字がshift/濁音/半濁音の場合は、それに対応するキーも評価に加える
                let additional_finger = match k {
                    crate::keymap::KeyKind::Normal => None,
                    crate::keymap::KeyKind::Shift => {
                        if HAND_ASSIGNMENT[*r][*c] == 1 {
                            Some(RIGHT_SHIFT_INDEX)
                        } else {
                            Some(LEFT_SHIFT_INDEX)
                        }
                    }
                    crate::keymap::KeyKind::Turbid => {
                        if HAND_ASSIGNMENT[*r][*c] == 1 {
                            Some(RIGHT_TURBID_INDEX)
                        } else {
                            Some(LEFT_TURBID_INDEX)
                        }
                    }
                    crate::keymap::KeyKind::Semiturbid => {
                        if HAND_ASSIGNMENT[*r][*c] == 1 {
                            Some(RIGHT_SEMITURBID_INDEX)
                        } else {
                            Some(LEFT_SEMITURBID_INDEX)
                        }
                    }
                };

                if let Some((r, c)) = additional_finger {
                    score += FINGER_WEIGHTS[r][c];
                }
            }
        }

        score += special_evaluations(&conjunction, keymap);
    }
    score
}
