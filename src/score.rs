use crate::{
    char_def,
    connection_score::{ConnectionScore, Evaluation},
    keymap::Keymap,
};

#[derive(Debug, Clone)]
pub struct Conjunction {
    /// 連接のテキスト
    ///
    /// 内部の値は、そのままall_charsの順序である
    pub text: Vec<usize>,
    /// 連接の出現回数
    pub appearances: u32,

    /// このconjunctionのhash
    pub hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluated {
    // conjunctionを評価した結果
    score: u64,
    // 評価したconjunctionのhash
    conjunction_hash: u64,
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

impl Into<u64> for Score {
    fn into(self) -> u64 {
        self.total_score
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

    let mut score_obj = Score {
        evaluated: Vec::with_capacity(conjunctions.len()),
        total_score: 0,
    };

    let mut key_sequence: Vec<Evaluation> = Vec::with_capacity(10);
    for conjunction in conjunctions {
        for ch in conjunction.text.iter() {
            let seq = &pos_cache[*ch];

            key_sequence.extend(seq.iter().cloned());
        }

        let current_score = pre_scores.evaluate(&key_sequence) * conjunction.appearances as u64;
        score += current_score;
        score_obj.evaluated.push(Evaluated {
            score: current_score,
            conjunction_hash: conjunction.hash,
        });
        key_sequence.clear()
    }

    score_obj.total_score = score;
    score_obj
}
