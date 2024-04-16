use std::collections::HashMap;

use crate::{
    char_def::CHARS,
    keymap::{
        key_indices, KeyKind, Keymap, LEFT_SEMITURBID_INDEX, LEFT_SHIFT_INDEX, LEFT_TURBID_INDEX,
        RIGHT_SEMITURBID_INDEX, RIGHT_SHIFT_INDEX, RIGHT_TURBID_INDEX,
    },
};

/// 各指が担当するキーに対する重み。
#[rustfmt::skip]
static FINGER_WEIGHTS: [[u16; 10];3] = [
    [1000, 30, 20, 50, 1000, 1000, 50, 20, 30, 1000],
    [40,   20, 10, 10,   30,   30, 10, 10, 20,   40],
    [80,   40, 30, 20,   60,   60, 20, 30, 40,   80],
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
static TWO_CONNECTION_WEIGHT: [(Pos, Pos, u16); 18] = [
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
    // 中指中段→人差し冗談
    (Pos(1, 7), Pos(0, 6), 90),
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
    // 中指中段→人差し冗談
    (Pos(1, 2), Pos(0, 3), 90),
];

pub struct ConnectionScore {
    /// 4連接までのscore。
    scores: Vec<u32>,
    /// positionと対応するキー位置のmapping
    position_id_map: HashMap<Pos, usize>,
}

impl ConnectionScore {
    pub fn new() -> Self {
        let indices = key_indices().into_iter().collect::<Vec<_>>();
        let position_id_map = indices
            .iter()
            .enumerate()
            .map(|(i, pos)| (pos.clone().into(), i))
            .collect();

        let mut this = ConnectionScore {
            scores: vec![0; 32 ^ 4],
            position_id_map,
        };

        for i in indices.iter() {
            for j in indices.iter() {
                for k in indices.iter() {
                    for l in indices.iter() {
                        let score = this.evaluate_connection(i, j, k, l);
                        let index = this.get_index(i, j, k, l);
                        this.scores[index] = score;
                    }
                }
            }
        }

        this
    }

    /// 4連接の評価を行う
    ///
    /// 4連接の評価は、以下のscoreの合算とする。
    /// - 先頭2連接の評価
    /// - 先頭3連接の評価
    /// - 今回打鍵するキーの評価
    ///
    /// 2連接の評価は、以下のルールに従う。
    /// - TWO_CONNECTIONにあるcost。但し設定されていない場合は0
    ///
    /// 3連接の評価は、以下のルールに従う。
    /// - THREE_CONNECTIONSにあるcost。但し設定されていない場合は、打鍵するキーの評価値の合算
    ///
    /// # Arguments
    /// * `i` - 1つ目のキー
    /// * `j` - 2つ目のキー
    /// * `k` - 3つ目のキー
    /// * `l` - 4つ目のキー
    ///
    /// # Returns
    /// 評価値
    fn evaluate_connection(
        &self,
        i: &(usize, usize),
        j: &(usize, usize),
        k: &(usize, usize),
        l: &(usize, usize),
    ) -> u32 {
        let mut score = 0;
        let i: Pos = Pos::from(*i);
        let j: Pos = Pos::from(*j);
        let k: Pos = Pos::from(*k);
        let l: Pos = Pos::from(*l);

        // 2連接の評価
        score += i.two_conjunction_scores(&j);
        score += j.two_conjunction_scores(&k);

        // 3連接の評価
        score += self.three_conjunction_scores(&i, &j, &k);

        score + FINGER_WEIGHTS[l.0][l.1] as u32
    }

    /// 3連接に対する評価を行う
    ///
    /// 現状ではキーの値に対する評価のみである。
    fn three_conjunction_scores(&self, first: &Pos, second: &Pos, third: &Pos) -> u32 {
        let mut score = FINGER_WEIGHTS[first.0][first.1]
            + FINGER_WEIGHTS[second.0][second.1]
            + FINGER_WEIGHTS[third.0][third.1];

        let rules = [
            |first: &Pos, second: &Pos, third: &Pos| {
                // 同じ指でスキップが連続している場合はペナルティ
                if first.is_skip_row_on_same_finger(second)
                    && second.is_skip_row_on_same_finger(third)
                {
                    300
                } else {
                    0
                }
            },
            |first: &Pos, second: &Pos, third: &Pos| {
                // 同じ手を連続して打鍵しているばあいはペナルティを与える
                if first.is_same_hand(second) && second.is_same_hand(third) {
                    100
                } else {
                    0
                }
            },
        ];

        rules.iter().fold(score.into(), |score, rule| {
            score + rule(first, second, third)
        })
    }

    /// 4連接に対応する全体のindexを返す。
    ///
    /// 26キーあるので、これが5bitに収まる。shiftのstateはいずれかにつき一回にはなる。
    /// 全体としてこれらがこれが追加されると3bit必要になってしまい、32bitが必要になってしまい、メモリに乗らなくなってしまう。
    /// なので、シフトの評価自体は別途行うことにする。
    fn get_index(
        &self,
        i: &(usize, usize),
        j: &(usize, usize),
        k: &(usize, usize),
        l: &(usize, usize),
    ) -> usize {
        let bit_mask = 0b00011111;
        let i: Pos = Pos::from(*i);
        let j: Pos = Pos::from(*j);
        let k: Pos = Pos::from(*k);
        let l: Pos = Pos::from(*l);

        let i = self.position_id_map[&i] & bit_mask;
        let j = self.position_id_map[&j] & bit_mask;
        let k = self.position_id_map[&k] & bit_mask;
        let l = self.position_id_map[&l] & bit_mask;

        i << 15 | j << 10 | k << 5 | l
    }
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Clone)]
struct Pos(usize, usize);

impl From<(usize, usize)> for Pos {
    fn from((r, c): (usize, usize)) -> Self {
        Pos(r, c)
    }
}

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

    /// 2連接に対する評価を実施する
    fn two_conjunction_scores(&self, other: &Pos) -> u32 {
        let rules = [
            |first: &Pos, second: &Pos| {
                // 同じ指で同じキーを連続して押下している場合はペナルティを与える
                if first == second {
                    150
                } else {
                    0
                }
            },
            |first: &Pos, second: &Pos| {
                // 同じ指で同じ行をスキップしている場合はペナルティを与える
                if first.is_skip_row_on_same_finger(&second) {
                    100
                } else {
                    0
                }
            },
            |first: &Pos, second: &Pos| {
                // 同じ指を連続して打鍵しているばあいはペナルティを与える
                if first.is_same_hand_and_finger(&second) {
                    50
                } else {
                    0
                }
            },
            |first: &Pos, second: &Pos| {
                // 段飛ばしをしている場合はペナルティを与える
                if first.is_skip_row(&second) {
                    100
                } else {
                    0
                }
            },
            |first: &Pos, second: &Pos| {
                // 2連接のスコアを返す
                let position = TWO_CONNECTION_WEIGHT
                    .iter()
                    .position(|(p1, p2, _)| *p1 == *first && *p2 == *second);

                if let Some(position) = position {
                    TWO_CONNECTION_WEIGHT[position].2 as u32
                } else {
                    0
                }
            },
        ];

        rules
            .iter()
            .fold(0, |score, rule| score + rule(self, other))
    }
}
