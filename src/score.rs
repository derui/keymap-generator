use crate::{
    char_def,
    connection_score::{ConnectionScore, Evaluation},
    key_seq::KeySeq,
    keymap::Keymap,
    layout::{self},
};

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

    let mut pos_cache: Vec<Vec<Evaluation>> = Vec::with_capacity(char_def::all_chars().len());

    for c in char_def::all_chars().iter() {
        let Some(v) = keymap.get(*c) else {
            unreachable!("should not have any missing key")
        };
        pos_cache.push(
            v.as_raw_sequence()
                .iter()
                .map(|(p, shift)| Evaluation {
                    positions: p.into(),
                    shift: shift.is_some(),
                })
                .collect(),
        );
    }

    let mut key_sequence: Vec<Evaluation> = Vec::with_capacity(10);
    for conjunction in conjunctions {
        for ch in conjunction.text.iter() {
            let seq = &pos_cache[*ch];

            key_sequence.extend(seq.iter().cloned());
        }

        score += pre_scores.evaluate(&key_sequence) * conjunction.appearances as u64;
        key_sequence.clear()
    }
    score
}
