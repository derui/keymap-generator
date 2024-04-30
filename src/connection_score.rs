use std::{collections::HashMap, fs::File, io::Read, path::Path};

use scraper::{Html, Selector};

use crate::layout::{
    linear::{linear_layout, linear_mapping},
    Point,
};

/// 各指が担当するキーに対する重み。
/// http://61degc.seesaa.net/article/284288569.html
/// 上記を左右同置に改変し、多少の調整を付加
#[rustfmt::skip]
static FINGER_WEIGHTS: [[u16; 10];3] = [
    [1000,   126, 105, 152,  1000,  1000, 152, 105, 126, 1000],
    [120,     96,  91,  90,   138,   138,  90,  91,  96,  120],
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

/// ２キーの連接における所要時間。
#[derive(Debug, Clone)]
pub struct TwoKeyTiming {
    timings: HashMap<(Pos, Pos), u32>,
}

impl TwoKeyTiming {
    pub fn load(path: &Path) -> anyhow::Result<TwoKeyTiming> {
        let mut timings = HashMap::new();
        let mut file = File::open(path).unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        let html = Html::parse_document(&buf);
        let matrix_selector = Selector::parse("#matrix > tbody").unwrap();
        let tr_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();

        let keys = vec![
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
            'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', ';', ',', '.', '/',
        ];
        let mappings = linear_mapping();

        // tr/tdを一個ずつ対応させていく。0または1000の場合は無視する
        // tr/tdのそれぞれ１行目は、header行なので無視する
        for (ridx, row) in html
            .select(&matrix_selector)
            .next()
            .unwrap()
            .select(&tr_selector)
            .skip(1)
            .enumerate()
        {
            for (cidx, col) in row.select(&td_selector).skip(1).enumerate() {
                let txt = col.text().collect::<String>();

                if txt == "0" || txt == "1000" {
                    continue;
                }
                let timing = txt.parse::<u32>().expect("should be numeric");

                let first = mappings.get(&keys[ridx]);
                let second = mappings.get(&keys[cidx]);
                match (first, second) {
                    (Some(first), Some(second)) => {
                        let first = first.into();
                        let second = second.into();

                        timings.insert((first, second), timing);
                    }
                    _ => continue,
                }
            }
        }

        Ok(TwoKeyTiming { timings })
    }
}

pub struct ConnectionScore {
    /// 4連接までのscore。
    scores: Vec<u32>,
}

// struct for evaluation
#[derive(Debug)]
pub struct Evaluation {
    pub positions: (Pos, Option<Pos>),
}

impl ConnectionScore {
    pub fn new(timings: &TwoKeyTiming) -> Self {
        let indices = linear_layout();
        let mut this = ConnectionScore {
            scores: vec![0; 32_usize.pow(4)],
        };

        for i in indices.iter().cloned() {
            let score = this.evaluate_single_connection(&i.into());
            let index = this.get_index(&Some(i.into()), &None, &None, &None);
            this.scores[index] = score;

            for j in indices.iter().cloned() {
                let score = this.evaluate_two_connection(timings, &i.into(), &j.into());
                let index = this.get_index(&Some(i.into()), &Some(j.into()), &None, &None);
                this.scores[index] = score;

                for k in indices.iter().cloned() {
                    let score =
                        this.evaluate_three_connection(timings, &i.into(), &j.into(), &k.into());
                    let index =
                        this.get_index(&Some(i.into()), &Some(j.into()), &Some(k.into()), &None);
                    this.scores[index] = score;

                    for l in indices.iter().cloned() {
                        let score = this.evaluate_connection(
                            timings,
                            &i.into(),
                            &j.into(),
                            &k.into(),
                            &l.into(),
                        );
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
        let mut positions = sequence.iter().map(|v| v.positions);
        let mut score = 0;

        let first: Option<(usize, usize)> = positions.nth(0).map(|(p, _)| p.into());
        let second: Option<(usize, usize)> = positions.nth(1).map(|(p, _)| p.into());
        let third: Option<(usize, usize)> = positions.nth(2).map(|(p, _)| p.into());
        let fourth: Option<(usize, usize)> = positions.nth(3).map(|(p, _)| p.into());

        score += unsafe {
            *self
                .scores
                .get_unchecked(self.get_index(&first, &second, &third, &fourth)) as u64
        };

        let first: Option<(usize, usize)> = positions.nth(0).and_then(|(_, p)| p.map(|v| v.into()));
        let second: Option<(usize, usize)> =
            positions.nth(1).and_then(|(_, p)| p.map(|v| v.into()));
        let third: Option<(usize, usize)> = positions.nth(2).and_then(|(_, p)| p.map(|v| v.into()));
        let fourth: Option<(usize, usize)> =
            positions.nth(3).and_then(|(_, p)| p.map(|v| v.into()));

        score += unsafe {
            let v = *self
                .scores
                .get_unchecked(self.get_index(&first, &second, &third, &fourth))
                as f64;
            (v * 1.3) as u64
        };

        score
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
        timings: &TwoKeyTiming,
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
        score += i.two_conjunction_scores(&j, timings.clone());
        score += j.two_conjunction_scores(&k, timings.clone());

        // 3連接の評価
        score += self.three_conjunction_scores(&i, &j, &k);

        score + FINGER_WEIGHTS[l.0][l.1] as u32
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
        timings: &TwoKeyTiming,
        i: &(usize, usize),
        j: &(usize, usize),
        k: &(usize, usize),
    ) -> u32 {
        let i: Pos = Pos::from(*i);
        let j: Pos = Pos::from(*j);
        let k: Pos = Pos::from(*k);

        // 2連接の評価
        let mut score = i.two_conjunction_scores(&j, timings.clone());
        score += j.two_conjunction_scores(&k, timings.clone());

        score + FINGER_WEIGHTS[k.0][k.1] as u32
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
    fn evaluate_two_connection(
        &self,
        timings: &TwoKeyTiming,
        i: &(usize, usize),
        j: &(usize, usize),
    ) -> u32 {
        let i: Pos = Pos::from(*i);
        let j: Pos = Pos::from(*j);

        // 2連接の評価
        let score = i.two_conjunction_scores(&j, timings.clone());

        score + FINGER_WEIGHTS[i.0][i.1] as u32
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
    fn two_conjunction_scores(&self, other: &Pos, timings: TwoKeyTiming) -> u32 {
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
        ];

        let special_case_score = rules
            .iter()
            .fold(0 as u32, |score, rule| score + rule(self, other));

        special_case_score + timings.timings.get(&(*self, *other)).clone().unwrap_or(&0)
    }
}
