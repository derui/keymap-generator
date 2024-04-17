use std::collections::HashMap;

use crate::{
    char_def::CHARS,
    connection_score::{ConnectionScore, Pos},
    keymap::{
        KeyKind, Keymap, LEFT_SEMITURBID_INDEX, LEFT_SHIFT_INDEX, LEFT_TURBID_INDEX,
        RIGHT_SEMITURBID_INDEX, RIGHT_SHIFT_INDEX, RIGHT_TURBID_INDEX,
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
    pub text: String,
    /// 連接の出現回数
    pub appearances: u32,
}

/// 連接に対して特殊な評価を行う。
///
/// # Arguments
/// * `sequence` - 評価する対象のキー連接
///
/// # Returns
/// 評価値
fn sequence_evaluation(sequence: &Vec<(Pos, Option<Pos>)>, pre_scores: &ConnectionScore) -> u64 {
    let score = sequence
        .iter()
        .map(|(_, shift)| {
            // シフトは固有のコストがかかるものとしている
            let score_shift = shift.clone().map(|_| 50).unwrap_or(0);
            score_shift
        })
        .sum();

    if sequence.len() == 4 {
        score + pre_scores.evaluate(&sequence[0], &sequence[1], &sequence[2], &sequence[3])
    } else {
        score
    }
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

                key_sequence.push((
                    (*r, *c).into(),
                    additional_finger.map(|(r, c)| (r, c).into()),
                ));
            }
        }

        score += sequence_evaluation(&key_sequence, &pre_scores);
    }
    score
}
