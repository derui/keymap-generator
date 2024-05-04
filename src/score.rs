use crate::{
    char_def,
    connection_score::{ConnectionScore, Evaluation, Pos},
    keymap::{KeyKind, Keymap},
    layout::{
        self,
        linear::{
            LINEAR_L_SEMITURBID_INDEX, LINEAR_L_SHIFT_INDEX, LINEAR_L_TURBID_INDEX,
            LINEAR_R_SEMITURBID_INDEX, LINEAR_R_SHIFT_INDEX, LINEAR_R_TURBID_INDEX,
        },
        Point,
    },
};

/// キーを押下する手の割当。1 = 左手、2 = 右手
static HAND_ASSIGNMENT: [[u8; 10]; 3] = [
    [1, 1, 1, 1, 1, 2, 2, 2, 2, 2],
    [1, 1, 1, 1, 1, 2, 2, 2, 2, 2],
    [1, 1, 1, 1, 1, 2, 2, 2, 2, 2],
];

#[derive(Debug, Clone)]
pub struct Conjunction {
    /// 連接のテキスト
    ///
    /// 内部の値は、そのままall_charsの順序である
    pub text: Vec<usize>,
    /// 連接の出現回数
    pub appearances: u32,
}

/// [keymap]の評価を行う。scoreは低いほど良好であるとする。
///
/// # Arguments
/// * `texts` - 評価対象の文字列
/// * `keymap` - キーマップ
///
/// # Returns
/// 評価値
pub fn evaluate(
    conjunctions: &[Conjunction],
    pre_scores: &ConnectionScore,
    keymap: &Keymap,
) -> u64 {
    let mut score = 0;
    let linear_layout = layout::linear::linear_layout();

    let mut pos_cache: Vec<(KeyKind, Point)> = Vec::with_capacity(char_def::all_chars().len());

    for c in char_def::all_chars().iter() {
        let Some(v) = keymap.get(*c) else {
            unreachable!("should not have any missing key")
        };
        pos_cache.push(v);
    }

    let mut key_sequence: Vec<Evaluation> = Vec::with_capacity(4);
    for conjunction in conjunctions {
        for ch in conjunction.text.iter() {
            let (k, point) = pos_cache[*ch];
            let (r, c): (usize, usize) = point.into();

            // 対象の文字がshift/濁音/半濁音の場合は、それに対応するキーも評価に加える
            // この場合、一応1動作として扱うのだが、次の打鍵に対してそれぞれからの評価が追加されるものとする
            let additional_finger: Option<Pos> = match k {
                crate::keymap::KeyKind::Normal => None,
                crate::keymap::KeyKind::Shift => {
                    if HAND_ASSIGNMENT[r][c] == 2 {
                        Some((&linear_layout[LINEAR_L_SHIFT_INDEX]).into())
                    } else {
                        Some((&linear_layout[LINEAR_R_SHIFT_INDEX]).into())
                    }
                }
                crate::keymap::KeyKind::Turbid => {
                    if HAND_ASSIGNMENT[r][c] == 2 {
                        Some((&linear_layout[LINEAR_L_TURBID_INDEX]).into())
                    } else {
                        Some((&linear_layout[LINEAR_R_TURBID_INDEX]).into())
                    }
                }
                crate::keymap::KeyKind::Semiturbid => {
                    if HAND_ASSIGNMENT[r][c] == 2 {
                        Some((&linear_layout[LINEAR_L_SEMITURBID_INDEX]).into())
                    } else {
                        Some((&linear_layout[LINEAR_R_SEMITURBID_INDEX]).into())
                    }
                }
            };

            let v = Evaluation {
                positions: ((r, c).into(), additional_finger),
            };
            key_sequence.push(v);
        }

        score += pre_scores.evaluate(&key_sequence);
        key_sequence.clear()
    }
    score
}
