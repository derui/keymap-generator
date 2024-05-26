use std::fmt::Display;

use crate::{
    char_def,
    connection_score::{ConnectionScore, Evaluation},
    keymap::Keymap,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conjunction {
    /// 連接のテキスト
    ///
    /// 内部の値は、そのままall_charsの順序である
    pub text: Vec<usize>,
    /// 連接の出現回数
    pub appearances: u32,

    /// 各文字に対応する素数を乗算したもの
    pub hash: u64,
}

impl Conjunction {
    /// 指定された文字を含まず、再評価が必要ないかを判定する
    ///
    /// # Arguments
    /// * `diff_chars` - 差分となる文字。[all_chars]から返されるprimeである
    #[inline]
    fn can_skip_evaluation(&self, diff_chars: &[u64]) -> bool {
        for d in diff_chars {
            if self.hash % *d == 0 {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluated {
    // conjunctionを評価した結果
    score: u64,
    // 評価したconjunctionのindex
    conjunction_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Score {
    // conjunctionの評価結果
    evaluated: Vec<Evaluated>,

    total_score: u64,
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.total_score.cmp(&other.total_score)
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.total_score.cmp(&other.total_score))
    }
}

impl From<Score> for u64 {
    fn from(val: Score) -> Self {
        val.total_score
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.total_score)
    }
}

fn make_pos_cache(keymap: &Keymap) -> Vec<Vec<Evaluation>> {
    let mut pos_cache: Vec<Vec<Evaluation>> = Vec::with_capacity(char_def::all_chars().len());

    for (_, c) in char_def::all_chars().iter() {
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

    pos_cache
}

impl Score {
    /// 変更があった文字に対する評価を行う。scoreは低いほど良好であるとする。
    ///
    /// # Arguments
    /// * `pre_scores` - 事前に評価した連接評価
    /// * `diff_chars` - 前回との差分となる文字
    ///
    /// # Returns
    /// 評価値
    pub fn evaluate_only_diff(
        &self,
        conjunctions: &[Conjunction],
        pre_scores: &ConnectionScore,
        keymap: &Keymap,
        diff_chars: &[char],
    ) -> Score {
        let mut score_obj = self.clone();
        let all_chars = char_def::all_chars();
        let diff_chars = diff_chars
            .iter()
            .filter_map(|v| all_chars.iter().find(|(_, x)| *x == *v))
            .map(|v| v.0)
            .collect::<Vec<_>>();

        let pos_cache = make_pos_cache(keymap);

        let mut key_sequence: Vec<Evaluation> = Vec::with_capacity(10);
        for evaluated in score_obj.evaluated.iter_mut() {
            let conj = unsafe { conjunctions.get_unchecked(evaluated.conjunction_index) };

            if conj.can_skip_evaluation(&diff_chars) {
                continue;
            }

            for ch in conj.text.iter() {
                let seq = &pos_cache[*ch];

                key_sequence.extend(seq.iter().cloned());
            }

            let current_score = pre_scores.evaluate(&key_sequence) * conj.appearances as u64;
            score_obj.total_score -= evaluated.score;
            evaluated.score = current_score;
            score_obj.total_score += current_score;
            key_sequence.clear()
        }

        score_obj
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
) -> Score {
    let mut score = 0;

    let pos_cache = make_pos_cache(keymap);

    let mut score_obj = Score {
        evaluated: Vec::with_capacity(conjunctions.len()),
        total_score: 0,
    };

    let mut key_sequence: Vec<Evaluation> = Vec::with_capacity(10);
    for (index, conjunction) in conjunctions.iter().enumerate() {
        for ch in conjunction.text.iter() {
            let seq = &pos_cache[*ch];

            key_sequence.extend(seq.iter().cloned());
        }

        let current_score = pre_scores.evaluate(&key_sequence) * conjunction.appearances as u64;
        score += current_score;
        score_obj.evaluated.push(Evaluated {
            score: current_score,
            conjunction_index: index,
        });
        key_sequence.clear()
    }

    score_obj.total_score = score;
    score_obj
}
