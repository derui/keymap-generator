use std::{collections::HashMap, fs::File, path::Path};

use crate::{
    char_def,
    layout::{linear::linear_layout, Point},
};

/// 各指が担当するキーに対する重み。
/// http://61degc.seesaa.net/article/284288569.html
/// 上記を左右同置に改変
#[rustfmt::skip]
static FINGER_WEIGHTS: [[u16; 10];3] = [
    [1000,   126, 105, 152,  1000,  1000, 152, 105, 126, 1000],
    [97,      96,  91,  90,   138,   138,  90,  91,  96,   97],
    [157,    169, 170, 135,   146,   146, 135, 170, 169,  157],
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
static TWO_CONNECTION_WEIGHT: [(Pos, Pos, u16); 20] = [
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
    // 中指中段→人差し上段
    (Pos(1, 7), Pos(0, 6), 90),
    // 薬指上段→小指下段
    (Pos(0, 8), Pos(2, 9), 140),
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
    // 中指中段→人差し指上段
    (Pos(1, 2), Pos(0, 3), 90),
    // 薬指上段→小指下段
    (Pos(0, 1), Pos(2, 0), 140),
];

pub struct ConnectionScore {
    /// 4連接までのscore。
    scores: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct CharFrequency {
    frequency: Vec<f64>,
}

impl CharFrequency {
    pub fn read(path: &Path) -> anyhow::Result<CharFrequency> {
        let mut frequency = vec![0; char_def::all_chars().len()];
        let file = File::open(path).unwrap();

        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_reader(&file);

        let char_position_map: HashMap<char, usize> = char_def::all_chars()
            .into_iter()
            .enumerate()
            .map(|(idx, v)| (v, idx))
            .collect();

        for result in rdr.records() {
            // The iterator yields Result<StringRecord, Error>, so we check the
            // error here.
            let record = result?;
            let Some(text) = record
                .get(4)
                .filter(|v| !v.is_empty())
                .map(|v| v.to_string())
            else {
                break;
            };
            let appearances: u32 = record.get(5).unwrap().parse()?;
            if let Some(v) = char_position_map.get(&text.chars().next().unwrap()) {
                frequency[*v] = appearances as u64;
            }
        }

        let ave = frequency.iter().sum::<u64>() / frequency.len() as u64;

        Ok(CharFrequency {
            frequency: frequency
                .iter()
                .map(|v| {
                    let prob = (*v as f64) / ave as f64;
                    ((1.0 - prob) * 100.0)
                })
                .collect(),
        })
    }

    pub fn get_weight(&self, index: usize) -> f64 {
        self.frequency[index]
    }
}

// struct for evaluation
#[derive(Debug)]
pub struct Evaluation {
    pub key_weight: f64,
    pub positions: (Pos, Option<Pos>),
}

impl ConnectionScore {
    pub fn new() -> Self {
        let indices = linear_layout();
        let mut this = ConnectionScore {
            scores: vec![0; 32_usize.pow(4)],
        };

        for i in indices.iter().cloned() {
            let score = this.evaluate_single_connection(&i.into());
            let index = this.get_index(&Some(i.into()), &None, &None, &None);
            this.scores[index] = score;

            for j in indices.iter().cloned() {
                let score = this.evaluate_two_connection(&i.into(), &j.into());
                let index = this.get_index(&Some(i.into()), &Some(j.into()), &None, &None);
                this.scores[index] = score;

                for k in indices.iter().cloned() {
                    let score = this.evaluate_three_connection(&i.into(), &j.into(), &k.into());
                    let index =
                        this.get_index(&Some(i.into()), &Some(j.into()), &Some(k.into()), &None);
                    this.scores[index] = score;

                    for l in indices.iter().cloned() {
                        let score =
                            this.evaluate_connection(&i.into(), &j.into(), &k.into(), &l.into());
                        let index = this.get_index(
                            &Some(i.into()),
                            &Some(j.into()),
                            &Some(k.into()),
                            &Some(l.into()),
                        );
                        this.scores[index] = score;
                    }
                }
            }
        }

        this
    }

    /// キーから、評価の結果を返す
    ///
    /// ここでの結果は、4連接自体と、シフトに対する評価の両方の合算値である。
    pub fn evaluate(&self, sequence: &[Evaluation]) -> u64 {
        let _weights = sequence.iter().map(|v| (v.key_weight)).collect::<Vec<_>>();
        let positions = sequence.iter().map(|v| (v.positions)).collect::<Vec<_>>();
        let mut score = 0;

        let first: Option<(usize, usize)> = positions.first().map(|(p, _)| p.into());
        let second: Option<(usize, usize)> = positions.get(1).map(|(p, _)| p.into());
        let third: Option<(usize, usize)> = positions.get(2).map(|(p, _)| p.into());
        let fourth: Option<(usize, usize)> = positions.get(3).map(|(p, _)| p.into());

        score += self.scores[self.get_index(&first, &second, &third, &fourth)] as u64;

        let first: Option<(usize, usize)> = positions.first().and_then(|(_, v)| *v).map(Into::into);
        let second: Option<(usize, usize)> = positions.get(1).and_then(|(_, v)| *v).map(Into::into);
        let third: Option<(usize, usize)> = positions.get(2).and_then(|(_, v)| *v).map(Into::into);
        let fourth: Option<(usize, usize)> = positions.get(3).and_then(|(_, v)| *v).map(Into::into);

        score +=
            (self.scores[self.get_index(&first, &second, &third, &fourth)] as f64 * 1.3) as u64;

        score + self.get_weight_score(sequence)
    }

    fn get_weight_score(&self, sequence: &[Evaluation]) -> u64 {
        let base = sequence
            .iter()
            .map(|v| {
                let w = v.key_weight;
                let p = v.positions.0;

                (FINGER_WEIGHTS[p.0][p.1] as f64 * w) as u64
            })
            .sum::<u64>();

        let shift = sequence
            .iter()
            .map(|v| {
                let w = v.key_weight;

                if let Some(p) = v.positions.1 {
                    (FINGER_WEIGHTS[p.0][p.1] as f64 * 1.3 * w) as u64
                } else {
                    0
                }
            })
            .sum::<u64>();

        base + shift
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

        score
            + FINGER_WEIGHTS[i.0][i.1] as u32
            + FINGER_WEIGHTS[j.0][j.1] as u32
            + FINGER_WEIGHTS[k.0][k.1] as u32
            + FINGER_WEIGHTS[l.0][l.1] as u32
    }

    /// 3連接の評価を行う
    ///
    /// 3連接の評価は、以下のscoreの合算とする。
    /// - 先頭2連接の評価
    /// - 今回打鍵するキーの評価
    ///
    /// 2連接の評価は、以下のルールに従う。
    /// - TWO_CONNECTIONにあるcost。但し設定されていない場合は0
    ///
    /// # Arguments
    /// * `i` - 1つ目のキー
    /// * `j` - 2つ目のキー
    /// * `k` - 3つ目のキー
    ///
    /// # Returns
    /// 評価値
    fn evaluate_three_connection(
        &self,
        i: &(usize, usize),
        j: &(usize, usize),
        k: &(usize, usize),
    ) -> u32 {
        let i: Pos = Pos::from(*i);
        let j: Pos = Pos::from(*j);
        let k: Pos = Pos::from(*k);

        // 2連接の評価
        let mut score = i.two_conjunction_scores(&j);
        score += j.two_conjunction_scores(&k);

        score
            + FINGER_WEIGHTS[i.0][i.1] as u32
            + FINGER_WEIGHTS[j.0][j.1] as u32
            + FINGER_WEIGHTS[k.0][k.1] as u32
    }

    /// 2連接の評価を行う
    ///
    /// 2連接の評価は、以下のscoreの合算とする。
    /// - 先頭2連接の評価
    /// - 今回打鍵するキーの評価
    ///
    /// 2連接の評価は、以下のルールに従う。
    /// - TWO_CONNECTIONにあるcost。但し設定されていない場合は0
    ///
    /// # Arguments
    /// * `i` - 1つ目のキー
    /// * `j` - 2つ目のキー
    ///
    /// # Returns
    /// 評価値
    fn evaluate_two_connection(&self, i: &(usize, usize), j: &(usize, usize)) -> u32 {
        let i: Pos = Pos::from(*i);
        let j: Pos = Pos::from(*j);

        // 2連接の評価
        let score = i.two_conjunction_scores(&j);

        score + FINGER_WEIGHTS[i.0][i.1] as u32 + FINGER_WEIGHTS[j.0][j.1] as u32
    }
    /// 単一キーの評価を行う
    ///
    /// 単一キーの評価は、以下のscoreの合算とする。
    /// - 今回打鍵するキーの評価
    ///
    /// # Arguments
    /// * `i` - 1つ目のキー
    ///
    /// # Returns
    /// 評価値
    fn evaluate_single_connection(&self, i: &(usize, usize)) -> u32 {
        let i: Pos = Pos::from(*i);

        FINGER_WEIGHTS[i.0][i.1] as u32
    }

    /// 3連接に対する評価を行う
    fn three_conjunction_scores(&self, first: &Pos, second: &Pos, third: &Pos) -> u32 {
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
                // 同じ指を連続して打鍵しているばあいはペナルティを与える
                if first.is_same_hand_and_finger(second) && second.is_same_hand_and_finger(third) {
                    250
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

        rules
            .iter()
            .fold(0, |score, rule| score + rule(first, second, third))
    }

    /// 4連接に対応する全体のindexを返す。
    ///
    /// 26キーあるので、これが5bitに収まる。shiftのstateはいずれかにつき一回にはなる。
    /// 全体としてこれらがこれが追加されると3bit必要になってしまい、32bitが必要になってしまい、メモリに乗らなくなってしまう。
    /// なので、シフトの評価自体は別途行うことにする。
    fn get_index(
        &self,
        i: &Option<(usize, usize)>,
        j: &Option<(usize, usize)>,
        k: &Option<(usize, usize)>,
        l: &Option<(usize, usize)>,
    ) -> usize {
        let bit_mask = 0b00011111;
        // rowは0-2、colは0-9なので、全部合わせても31に収まるのでこうする。bit maskは本来なくてもいいのだが、一応追加している
        let mut index: usize = 0;
        if let Some((r, c)) = i {
            index = (index << 5) | ((r * 10 + c) & bit_mask)
        }
        if let Some((r, c)) = j {
            index = (index << 5) | ((r * 10 + c) & bit_mask)
        }
        if let Some((r, c)) = k {
            index = (index << 5) | ((r * 10 + c) & bit_mask)
        }
        if let Some((r, c)) = l {
            index = (index << 5) | ((r * 10 + c) & bit_mask)
        }

        index
    }
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Pos(usize, usize);

impl From<(usize, usize)> for Pos {
    fn from((r, c): (usize, usize)) -> Self {
        Pos(r, c)
    }
}

impl From<&Point> for Pos {
    fn from(value: &Point) -> Self {
        let (r, c): (usize, usize) = value.into();
        Pos(r, c)
    }
}

impl From<Pos> for (usize, usize) {
    fn from(pos: Pos) -> Self {
        (pos.0, pos.1)
    }
}

impl From<&Pos> for (usize, usize) {
    fn from(pos: &Pos) -> Self {
        (pos.0, pos.1)
    }
}

impl Pos {
    fn is_skip_row_on_same_finger(&self, other: &Pos) -> bool {
        self.1 == other.1 && (self.0 as isize - other.0 as isize).abs() == 2
    }

    /// 異指で段をスキップしているか
    fn is_skip_row(&self, other: &Pos) -> bool {
        let hand_self = HAND_ASSIGNMENT[self.0][self.1];
        let hand_other = HAND_ASSIGNMENT[other.0][other.1];

        hand_self == hand_other && (self.0 as isize - other.0 as isize).abs() == 2
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
                if *first == *second {
                    150
                } else {
                    0
                }
            },
            |first: &Pos, second: &Pos| {
                // 同じ指で同じ行をスキップしている場合はペナルティを与える
                if first.is_skip_row_on_same_finger(second) {
                    100
                } else {
                    0
                }
            },
            |first: &Pos, second: &Pos| {
                // 同じ指を連続して打鍵しているばあいはペナルティを与える
                if first.is_same_hand_and_finger(second) {
                    50
                } else {
                    0
                }
            },
            |first: &Pos, second: &Pos| {
                // 段飛ばしをしている場合はペナルティを与える
                if first.is_skip_row(second) {
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
