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

/// キーを押下する指の割当。 1 = 人差し指、2 = 中指、３ = 薬指、４ = 小指
static FINGER_ASSIGNMENT: [[u8; 10]; 3] = [
    [4, 3, 2, 1, 1, 1, 1, 2, 3, 4],
    [4, 3, 2, 1, 1, 1, 1, 2, 3, 4],
    [4, 3, 2, 1, 1, 1, 1, 2, 3, 4],
];

#[derive(Debug, Clone)]
pub struct Conjunction {
    /// 連接のテキスト
    pub text: String,
    /// 連接の出現回数
    pub appearances: u32,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Clone)]
struct Pos(usize, usize);

impl Pos {
    fn is_skip_row_on_same_finger(&self, other: &Pos) -> bool {
        self.0 == other.0 && (self.1 as isize - other.1 as isize).abs() >= 1
    }

    /// 同一の手、かつ同一の指かどうか
    fn is_same_hand_and_finger(&self, other: &Pos) -> bool {
        let finger_self = FINGER_ASSIGNMENT[self.0][self.1];
        let finger_other = FINGER_ASSIGNMENT[other.0][other.1];
        let hand_self = HAND_ASSIGNMENT[self.0][self.1];
        let hand_other = HAND_ASSIGNMENT[other.0][other.1];

        finger_self == finger_other && hand_self == hand_other
    }
}

/// 2連接に対する評価を実施する
fn two_conjunction_scores(first: &Pos, second: &Pos) -> u64 {
    let rules = [
        |first: &Pos, second: &Pos| {
            // 同じ指で同じキーを連続して押下している場合はペナルティを与える
            if first == second {
                150 * FINGER_WEIGHTS[first.0][first.1]
            } else {
                0
            }
        },
        |first: &Pos, second: &Pos| {
            // 同じ指で同じ行をスキップしている場合はペナルティを与える
            if first.is_skip_row_on_same_finger(second) {
                50 * FINGER_WEIGHTS[first.0][first.1]
            } else {
                0
            }
        },
        |first: &Pos, second: &Pos| {
            // 同じ指を連続して打鍵しているばあいはペナルティを与える
            if first.is_same_hand_and_finger(second) {
                50 * FINGER_WEIGHTS[first.0][first.1]
            } else {
                0
            }
        },
    ];

    rules
        .iter()
        .fold(0, |score, rule| score + rule(first, second))
}

/// 3連接に対する評価を実施する
fn three_conjunction_scores(first: &Pos, second: &Pos, third: &Pos) -> u64 {
    let rules = [
        |first: &Pos, second: &Pos, third: &Pos| {
            // 同じ指で同じキーを連続して押下している場合はペナルティを与える
            if first == second && second == third {
                (300 * FINGER_WEIGHTS[first.0][first.1])
            } else {
                0
            }
        },
        |first: &Pos, second: &Pos, third: &Pos| {
            // 同じ指でスキップが連続している場合はペナルティ
            if first.is_skip_row_on_same_finger(second) && second.is_skip_row_on_same_finger(third)
            {
                (300 * FINGER_WEIGHTS[first.0][first.1])
            } else {
                0
            }
        },
        |first: &Pos, second: &Pos, third: &Pos| {
            // 同じ指を連続して打鍵しているばあいはペナルティを与える
            if first.is_same_hand_and_finger(second) && second.is_same_hand_and_finger(third) {
                100 * FINGER_WEIGHTS[first.0][first.1]
            } else {
                0
            }
        },
    ];

    rules
        .iter()
        .fold(0, |score, rule| score + rule(first, second, third))
}

/// 三重連接に対して特殊な評価を行う。
///
/// # Arguments
/// * `text` - 評価対象の文字列
/// * `keymap` - キーマップ
///
/// # Returns
/// 評価値
fn special_evaluations(
    conj: &Conjunction,
    pos_cache: &HashMap<char, (KeyKind, (usize, usize))>,
) -> u64 {
    let mut score = 0;
    let text: Vec<char> = conj.text.chars().collect();
    let keys = [
        pos_cache.get(&text[0]),
        pos_cache.get(&text[1]),
        pos_cache.get(&text[2]),
        pos_cache.get(&text[3]),
    ];

    let positions: Vec<Pos> = keys
        .iter()
        .filter_map(|v| v.map(|(_, (r, c))| Pos(*r, *c)))
        .collect();

    if positions.len() >= 2 {
        score += two_conjunction_scores(&positions[0], &positions[1]);
    }
    if positions.len() >= 3 {
        score += three_conjunction_scores(&positions[0], &positions[1], &positions[2]);
    }

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
pub fn evaluate(conjunctions: &[Conjunction], keymap: &Keymap) -> u64 {
    let mut score = 0;

    let mut pos_cache: HashMap<char, (KeyKind, (usize, usize))> = HashMap::new();

    for c in CHARS.iter() {
        if let Some(v) = keymap.get(*c) {
            pos_cache.insert(*c, v);
        }
    }

    for conjunction in conjunctions {
        for c in conjunction.text.chars() {
            if let Some((k, (r, c))) = pos_cache.get(&c) {
                score += FINGER_WEIGHTS[*r][*c];

                // 対象の文字がshift/濁音/半濁音の場合は、それに対応するキーも評価に加える
                // ただ、これらについて運指まで考慮するのはだいぶしんどいので、一旦気にしないで生成している
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

        score += special_evaluations(conjunction, &pos_cache);
    }
    score
}
