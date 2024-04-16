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
    [10000, 30, 20, 30, 10000, 10000, 30, 20, 30, 10000],
    [40,    20, 10, 10,    40,    40, 10, 10, 20,    40],
    [80,    50, 40, 20,    60,    60, 20, 40, 50,    80],
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

/// 2連接に対する重み。順序を持つ。
static TWO_CONNECTION_WEIGHT: [(Pos, Pos, u64); 16] = [
    // 人差し指伸ばし→小指下段
    (Pos(2, 5), Pos(2, 9), 150),
    // 人差し指伸ばし→小指下段
    (Pos(1, 5), Pos(2, 9), 150),
    // 小指下段→人差し指伸ばし
    (Pos(2, 9), Pos(2, 5), 150),
    // 小指下段→人差し指伸ばし
    (Pos(2, 9), Pos(1, 5), 150),
    // 人差し指伸→小指中段
    (Pos(2, 5), Pos(1, 9), 150),
    // 人差し指伸→小指中段
    (Pos(1, 5), Pos(1, 9), 90),
    // 小指中段→人差し指伸ばし
    (Pos(1, 9), Pos(2, 5), 150),
    // 小指中段→人差し指伸ばし
    (Pos(1, 9), Pos(1, 5), 90),
    // 左手
    // 人差し指伸ばし→小指下段
    (Pos(2, 4), Pos(2, 0), 150),
    // 人差し指伸ばし→小指下段
    (Pos(1, 4), Pos(2, 0), 150),
    // 小指下段→人差し指伸ばし
    (Pos(2, 0), Pos(2, 4), 150),
    // 小指下段→人差し指伸ばし
    (Pos(2, 0), Pos(1, 4), 150),
    // 人差し指伸→小指中段
    (Pos(2, 4), Pos(1, 0), 150),
    // 人差し指伸→小指中段
    (Pos(1, 4), Pos(1, 0), 90),
    // 小指中段→人差し指伸ばし
    (Pos(1, 0), Pos(2, 4), 150),
    // 小指中段→人差し指伸ばし
    (Pos(1, 0), Pos(1, 4), 90),
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
        self.1 == other.1 && (self.0 as isize - other.0 as isize).abs() == 2
    }

    /// 異指で段をスキップしているか
    fn is_skip_row(&self, other: &Pos) -> bool {
        (self.0 as isize - other.0 as isize).abs() == 2
    }

    /// 同じ手で押下しているかどうか
    fn is_same_hand(&self, other: &Pos) -> bool {
        HAND_ASSIGNMENT[self.0][self.1] == HAND_ASSIGNMENT[other.0][other.1]
    }

    /// 同一の手、かつ同一の指かどうか
    fn is_same_hand_and_finger(&self, other: &Pos) -> bool {
        let finger_self = FINGER_ASSIGNMENT[self.0][self.1];
        let finger_other = FINGER_ASSIGNMENT[other.0][other.1];
        let hand_self = HAND_ASSIGNMENT[self.0][self.1];
        let hand_other = HAND_ASSIGNMENT[other.0][other.1];

        finger_self == finger_other && hand_self == hand_other
    }

    /// [other]との連接scoreを返す
    fn connection_score(&self, other: &Pos) -> u64 {
        let position = TWO_CONNECTION_WEIGHT
            .iter()
            .position(|(p1, p2, _)| *p1 == *self && *p2 == *other);

        if let Some(position) = position {
            TWO_CONNECTION_WEIGHT[position].2
        } else {
            0
        }
    }
}

type ShiftedPos = (Pos, Option<Pos>);

/// 2連接に対する評価を実施する
fn two_conjunction_scores(first: &ShiftedPos, second: &ShiftedPos) -> u64 {
    let rules = [
        |first: &ShiftedPos, second: &ShiftedPos| {
            // 同じ指で同じキーを連続して押下している場合はペナルティを与える
            if first == second {
                150 * FINGER_WEIGHTS[first.0 .0][first.0 .1]
            } else {
                0
            }
        },
        |first: &ShiftedPos, second: &ShiftedPos| {
            // 同じ指で同じ行をスキップしている場合はペナルティを与える
            if first.0.is_skip_row_on_same_finger(&second.0) {
                50 * FINGER_WEIGHTS[first.0 .0][first.0 .1]
            } else {
                0
            }
        },
        |first: &ShiftedPos, second: &ShiftedPos| {
            // 同じ指を連続して打鍵しているばあいはペナルティを与える
            if first.0.is_same_hand_and_finger(&second.0) {
                50 * FINGER_WEIGHTS[first.0 .0][first.0 .1]
            } else {
                0
            }
        },
        |first: &ShiftedPos, second: &ShiftedPos| {
            // 段飛ばしをしている場合はペナルティを与える
            if first.0.is_skip_row(&second.0) {
                FINGER_WEIGHTS[first.0 .0][first.0 .1] + FINGER_WEIGHTS[second.0 .0][second.0 .1]
            } else {
                0
            }
        },
        |first: &ShiftedPos, second: &ShiftedPos| {
            // 2連接のスコアを返す
            first.0.connection_score(&second.0)
        },
        |first: &ShiftedPos, second: &ShiftedPos| {
            // 同指のシフトがつづいている場合はペナルティを与える
            let (_, first) = first.clone();
            let (_, second) = second.clone();
            let first_hand = first.map(|p| HAND_ASSIGNMENT[p.0][p.1]).unwrap_or(0);
            let second_hand = second.map(|p| HAND_ASSIGNMENT[p.0][p.1]).unwrap_or(0);

            if first_hand == second_hand {
                150
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
fn three_conjunction_scores(first: &ShiftedPos, second: &ShiftedPos, third: &ShiftedPos) -> u64 {
    let rules = [
        |first: &ShiftedPos, second: &ShiftedPos, third: &ShiftedPos| {
            // 同じ指でスキップが連続している場合はペナルティ
            let (first, _) = first;
            let (second, _) = second;
            let (third, _) = third;
            if first.is_skip_row_on_same_finger(second) && second.is_skip_row_on_same_finger(third)
            {
                300 * FINGER_WEIGHTS[first.0][first.1]
            } else {
                0
            }
        },
        |first: &ShiftedPos, second: &ShiftedPos, third: &ShiftedPos| {
            // 同じ手を連続して打鍵しているばあいはペナルティを与える
            let (first, _) = first;
            let (second, _) = second;
            let (third, _) = third;
            if first.is_same_hand(second) && second.is_same_hand(third) {
                100
            } else {
                0
            }
        },
        |first: &ShiftedPos, second: &ShiftedPos, third: &ShiftedPos| {
            // 同じ手でシフトを連続している場合はペナルティを与える
            let (_, first) = first.clone();
            let (_, second) = second.clone();
            let (_, third) = third.clone();
            let first_hand = first.map(|p| HAND_ASSIGNMENT[p.0][p.1]).unwrap_or(0);
            let second_hand = second.map(|p| HAND_ASSIGNMENT[p.0][p.1]).unwrap_or(0);
            let third_hand = third.map(|p| HAND_ASSIGNMENT[p.0][p.1]).unwrap_or(0);

            if first_hand == second_hand && second_hand == third_hand {
                300
            } else {
                0
            }
        },
    ];

    rules
        .iter()
        .fold(0, |score, rule| score + rule(first, second, third))
}

/// 連接に対して特殊な評価を行う。
///
/// # Arguments
/// * `sequence` - 評価する対象のキー連接
///
/// # Returns
/// 評価値
fn sequence_evaluation(sequence: &Vec<(Pos, Option<Pos>)>) -> u64 {
    let mut score = sequence
        .iter()
        .map(|(unshift, shift)| {
            let score_unshift = FINGER_WEIGHTS[unshift.0][unshift.1];
            // シフトは固有のコストがかかるものとしている
            let score_shift = shift
                .clone()
                .map(|p| FINGER_WEIGHTS[p.0][p.1] + 50)
                .unwrap_or(0);
            score_unshift + score_shift
        })
        .sum();

    // 先頭の連接を評価する
    if sequence.len() >= 2 {
        score += two_conjunction_scores(&sequence[0], &sequence[1]);
    }

    // 3連接を評価する
    if sequence.len() >= 3 {
        score += three_conjunction_scores(&sequence[0], &sequence[1], &sequence[2]);
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
        let mut key_sequence: Vec<(Pos, Option<Pos>)> = Vec::new();

        for c in conjunction.text.chars() {
            if let Some((k, (r, c))) = pos_cache.get(&c) {
                // 対象の文字がshift/濁音/半濁音の場合は、それに対応するキーも評価に加える
                // この場合、一応1動作として扱うのだが、次の打鍵に対してそれぞれからの評価が追加されるものとする
                let additional_finger = match k {
                    crate::keymap::KeyKind::Normal => None,
                    crate::keymap::KeyKind::Shift => {
                        if HAND_ASSIGNMENT[*r][*c] == 2 {
                            Some(LEFT_SHIFT_INDEX)
                        } else {
                            Some(RIGHT_SHIFT_INDEX)
                        }
                    }
                    crate::keymap::KeyKind::Turbid => {
                        if HAND_ASSIGNMENT[*r][*c] == 2 {
                            Some(LEFT_TURBID_INDEX)
                        } else {
                            Some(RIGHT_TURBID_INDEX)
                        }
                    }
                    crate::keymap::KeyKind::Semiturbid => {
                        if HAND_ASSIGNMENT[*r][*c] == 2 {
                            Some(LEFT_SEMITURBID_INDEX)
                        } else {
                            Some(RIGHT_SEMITURBID_INDEX)
                        }
                    }
                };

                key_sequence.push((Pos(*r, *c), additional_finger.map(|(r, c)| Pos(r, c))));
            }
        }

        score += sequence_evaluation(&key_sequence);
    }
    score
}
