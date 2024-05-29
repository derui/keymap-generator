use std::{collections::HashMap, fs::File, io::Read, path::Path};

use scraper::{Html, Selector};

use crate::layout::{
    linear::{self, linear_layout, linear_mapping},
    Point,
};

/// 各指が担当するキーに対する重み。
/// http://61degc.seesaa.net/article/284288569.html
/// 上記を左右同置に改変し、多少の調整を付加
#[rustfmt::skip]
static FINGER_WEIGHTS: [[u16; 10];3] = [
    [300, 126, 105, 152,   300,   300, 152, 105, 126, 300],
    [120,  96,  91,  90,   138,   138,  90,  91,  96,  120],
    [157, 150, 135, 135,   170,   170, 135, 135, 150,  157],
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
    pub timings: HashMap<(Point, Point), u32>,
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

        let keys = [
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
                        timings.insert((*first, *second), timing);
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
#[derive(Debug, Clone)]
pub struct Evaluation {
    pub positions: Point,
    pub shift: bool,
}

impl Default for Evaluation {
    fn default() -> Self {
        Self {
            positions: Point::new(0, 0),
            shift: false,
        }
    }
}

impl ConnectionScore {
    pub fn new(timings: &TwoKeyTiming) -> Self {
        let mut indices = linear_layout();
        indices.push(linear::get_left_small_shifter());
        indices.push(linear::get_right_small_shifter());

        let mut this = ConnectionScore {
            scores: vec![0; 32_usize.pow(4)],
        };

        for i in indices.iter().cloned() {
            let score = this.evaluate_single_connection(&i.into());
            let index = this.get_index(&Some(i.into()), &None, &None, &None);
            this.scores[index] = score;

            for j in indices.iter().cloned() {
                let score = this.evaluate_two_connection(&i.into(), &j.into(), timings);
                let index = this.get_index(&Some(i.into()), &Some(j.into()), &None, &None);
                this.scores[index] = score;

                for k in indices.iter().cloned() {
                    let score =
                        this.evaluate_three_connection(&i.into(), &j.into(), &k.into(), timings);
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
    pub fn evaluate(&self, sequence: &[&Evaluation]) -> u64 {
        let mut score = 0;

        let first: (&Point, bool) = (&sequence[0].positions, sequence[0].shift);
        let second: (&Point, bool) = (&sequence[1].positions, sequence[1].shift);
        let third: (&Point, bool) = (&sequence[2].positions, sequence[2].shift);
        let fourth: (&Point, bool) = (&sequence[3].positions, sequence[3].shift);

        score += unsafe {
            let score = *self
                .scores
                .get_unchecked(self.get_index_of_evaluation(&first, &second, &third, &fourth));

            let shifts: i32 = first.1 as i32 + second.1 as i32 + third.1 as i32 + fourth.1 as i32;

            (score as f32 * (3_f32).sqrt().powi(shifts)) as u64
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
        let score = self.evaluate_three_connection(i, j, k, timings);
        let l: Point = Point::from(*l);

        score + FINGER_WEIGHTS[l.row()][l.col()] as u32
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
        timings: &TwoKeyTiming,
    ) -> u32 {
        let two_score = self.evaluate_two_connection(i, j, timings);
        let i: Point = Point::from(*i);
        let j: Point = Point::from(*j);
        let k: Point = Point::from(*k);

        two_score
            + self.three_conjunction_scores(&i, &j, &k)
            + FINGER_WEIGHTS[k.row()][k.col()] as u32
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
        i: &(usize, usize),
        j: &(usize, usize),
        timings: &TwoKeyTiming,
    ) -> u32 {
        let i: Point = Point::from(*i);
        let j: Point = Point::from(*j);

        // 2連接の評価
        FINGER_WEIGHTS[i.row()][i.col()] as u32
            + FINGER_WEIGHTS[j.row()][j.col()] as u32
            + point_score::two_conjunction_scores(&i, &j, timings)
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
        let i: Point = Point::from(*i);

        FINGER_WEIGHTS[i.row()][i.col()] as u32
    }

    /// 3連接に対する評価を行う
    fn three_conjunction_scores(&self, first: &Point, second: &Point, third: &Point) -> u32 {
        let rules = [
            |first: &Point, second: &Point, third: &Point| {
                // スキップが連続している場合はペナルティ
                if point_score::is_skip_row(first, second)
                    && point_score::is_skip_row(second, third)
                {
                    300
                } else {
                    0
                }
            },
            |first: &Point, second: &Point, third: &Point| {
                // 同じ指を連続して打鍵しているばあいはペナルティを与える
                if point_score::is_same_hand_and_finger(first, second)
                    && point_score::is_same_hand_and_finger(second, third)
                {
                    300
                } else {
                    0
                }
            },
            |first: &Point, second: &Point, third: &Point| {
                // 小指が連続する場合はペナルティを与える
                if point_score::is_pinky(first)
                    && point_score::is_pinky(second)
                    && point_score::is_pinky(third)
                {
                    200
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
            index = (index << 5) | ((r * 10 + c + 1) & bit_mask)
        } else {
            index <<= 5
        }
        if let Some((r, c)) = j {
            index = (index << 5) | ((r * 10 + c + 1) & bit_mask)
        } else {
            index <<= 5
        }
        if let Some((r, c)) = k {
            index = (index << 5) | ((r * 10 + c + 1) & bit_mask)
        } else {
            index <<= 5
        }
        if let Some((r, c)) = l {
            index = (index << 5) | ((r * 10 + c + 1) & bit_mask)
        } else {
            index <<= 5
        }

        index
    }

    /// 4連接に対応する全体のindexを返す。
    ///
    /// 26キーあるので、これが5bitに収まる。shiftのstateはいずれかにつき一回にはなる。
    /// 全体としてこれらがこれが追加されると3bit必要になってしまい、32bitが必要になってしまい、メモリに乗らなくなってしまう。
    /// なので、シフトの評価自体は別途行うことにする。
    fn get_index_of_evaluation(
        &self,
        i: &(&Point, bool),
        j: &(&Point, bool),
        k: &(&Point, bool),
        l: &(&Point, bool),
    ) -> usize {
        let bit_mask = 0b00011111;
        // rowは0-2、colは0-9なので、全部合わせても31に収まるのでこうする。bit maskは本来なくてもいいのだが、一応追加している
        let mut index: usize = 0;
        index = (index << 5) | ((i.0.row() * 10 + i.0.col() + 1) & bit_mask);
        index = (index << 5) | ((j.0.row() * 10 + j.0.col() + 1) & bit_mask);
        index = (index << 5) | ((k.0.row() * 10 + k.0.col() + 1) & bit_mask);
        index = (index << 5) | ((l.0.row() * 10 + l.0.col() + 1) & bit_mask);

        index
    }
}

mod point_score {
    use crate::layout::Point;

    use super::{TwoKeyTiming, FINGER_ASSIGNMENT, HAND_ASSIGNMENT};

    #[inline]
    fn is_skip_row_on_same_finger(me: &Point, other: &Point) -> bool {
        me.row() == other.row()
            && me.col() == other.col()
            && (me.row() as isize - other.row() as isize).abs() == 2
    }

    /// 異指で段をスキップしているか
    #[inline]
    pub fn is_skip_row(me: &Point, other: &Point) -> bool {
        let hand_self = HAND_ASSIGNMENT[me.row()][me.col()];
        let hand_other = HAND_ASSIGNMENT[other.row()][other.col()];
        let finger_self = FINGER_ASSIGNMENT[me.row()][me.col()];
        let finger_other = FINGER_ASSIGNMENT[other.row()][other.col()];

        hand_self == hand_other
            && finger_self != finger_other
            && (me.row() as isize - other.row() as isize).abs() == 2
    }

    /// 同じ指で異なる行、異なる列を入力しているか
    #[inline]
    pub fn is_same_finger_dance(me: &Point, other: &Point) -> bool {
        let hand_self = HAND_ASSIGNMENT[me.row()][me.col()];
        let hand_other = HAND_ASSIGNMENT[other.row()][other.col()];
        let finger_self = FINGER_ASSIGNMENT[me.row()][me.col()];
        let finger_other = FINGER_ASSIGNMENT[other.row()][other.col()];

        hand_self == hand_other
            && finger_self == finger_other
            && me.row() != other.row()
            && me.col() != other.col()
    }

    ///
    #[inline]
    pub fn is_arpeggio(me: &Point, other: &Point) -> bool {
        let hand_self = HAND_ASSIGNMENT[me.row()][me.col()];
        let hand_other = HAND_ASSIGNMENT[other.row()][other.col()];

        let alpeggios = [
            // 左手の人差し指と中指
            (Point::new(0, 2), Point::new(1, 3)),
            (Point::new(1, 2), Point::new(2, 3)),
            // 左手の人差し指と小指
            (Point::new(1, 0), Point::new(1, 3)),
            // 左手の人差し指と薬指
            (Point::new(0, 1), Point::new(1, 3)),
            // 左手の中指と薬指
            (Point::new(0, 2), Point::new(0, 1)),
            (Point::new(1, 2), Point::new(1, 1)),
            // 左手の中指と小指
            (Point::new(0, 2), Point::new(1, 0)),
            // 左手の小指と薬指
            (Point::new(1, 0), Point::new(0, 1)),
            // 右手の人差し指と中指
            (Point::new(1, 6), Point::new(0, 7)),
            (Point::new(2, 6), Point::new(1, 7)),
            // 右手の人差し指と薬指
            (Point::new(0, 8), Point::new(1, 6)),
            // 右手の人差し指と小指
            (Point::new(1, 9), Point::new(1, 6)),
            // 右手の中指と薬指
            (Point::new(0, 7), Point::new(0, 8)),
            (Point::new(1, 7), Point::new(1, 8)),
            // 右手の小指と薬指
            (Point::new(1, 9), Point::new(0, 8)),
        ];

        hand_self == hand_other
            && alpeggios
                .iter()
                .find(|v| **v == (*me, *other) || **v == (*other, *me))
                .is_some()
    }

    /// 同じ手で押下しているかどうか
    #[inline]
    pub fn is_same_hand(me: &Point, other: &Point) -> bool {
        HAND_ASSIGNMENT[me.row()][me.col()] == HAND_ASSIGNMENT[other.row()][other.col()]
    }

    #[inline]
    pub fn is_pinky(me: &Point) -> bool {
        FINGER_ASSIGNMENT[me.row()][me.col()] == 4
    }

    /// 同一の手、かつ同一の指かどうか
    #[inline]
    pub fn is_same_hand_and_finger(me: &Point, other: &Point) -> bool {
        let finger_self = FINGER_ASSIGNMENT[me.row()][me.col()];
        let finger_other = FINGER_ASSIGNMENT[other.row()][other.col()];
        let hand_self = HAND_ASSIGNMENT[me.row()][me.col()];
        let hand_other = HAND_ASSIGNMENT[other.row()][other.col()];

        finger_self == finger_other && hand_self == hand_other
    }

    /// 2連接に対する評価を実施する
    pub fn two_conjunction_scores(me: &Point, other: &Point, timings: &TwoKeyTiming) -> u32 {
        let rules = [
            |first: &Point, second: &Point| {
                // 同じ指で行をスキップしている場合はペナルティを与える
                if is_skip_row_on_same_finger(first, second) {
                    150
                } else {
                    0
                }
            },
            |first: &Point, second: &Point| {
                // 同じ指で異段異列の場合はペナルティを与える
                if is_same_finger_dance(first, second) {
                    200
                } else {
                    0
                }
            },
            |first: &Point, second: &Point| {
                // 同じ指で連続して押下している場合はペナルティを与える
                if is_same_hand_and_finger(first, second) {
                    150
                } else {
                    0
                }
            },
            |first: &Point, second: &Point| {
                // 段飛ばしをしている場合はペナルティを与える
                if is_skip_row(first, second) {
                    100
                } else {
                    0
                }
            },
            |first: &Point, second: &Point| {
                // first->secondが押下しやすいアルペジオではない場合はペナルティを与える
                if !is_arpeggio(first, second) {
                    50
                } else {
                    0
                }
            },
        ];

        let special_case_score = rules
            .iter()
            .fold(0_u32, |score, rule| score + rule(me, other));

        special_case_score + timings.timings.get(&(*me, *other)).unwrap_or(&0)
    }
}
