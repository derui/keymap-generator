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
}

impl Conjunction {
    /// 指定された文字を含まず、再評価が必要ないかを判定する
    ///
    /// # Arguments
    /// * `diff_chars` - 差分となる文字。[all_chars]の順序である
    fn should_skip_reevaluation(&self, diff_chars: &[usize]) -> bool {
        for ch in self.text.iter() {
            if diff_chars.contains(ch) {
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
    // 評価したconjunctionのhash
    conjunction: Conjunction,
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

fn make_pos_cache(keymap: &Keymap) -> Vec<Vec<Evaluation>> {
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
        pre_scores: &ConnectionScore,
        keymap: &Keymap,
        diff_chars: &[char],
    ) -> Score {
        let mut score_obj = self.clone();
        let all_chars = char_def::all_chars();
        let diff_chars = diff_chars
            .iter()
            .map(|v| {
                all_chars
                    .iter()
                    .position(|x| *x == *v)
                    .expect("should be found char in all_chars")
            })
            .collect::<Vec<_>>();

        let pos_cache = make_pos_cache(keymap);

        let mut key_sequence: Vec<Evaluation> = Vec::with_capacity(10);
        for evaluated in score_obj.evaluated.iter_mut() {
            let conj = &evaluated.conjunction;

            if conj.should_skip_reevaluation(&diff_chars) {
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
    for conjunction in conjunctions {
        for ch in conjunction.text.iter() {
            let seq = &pos_cache[*ch];

            key_sequence.extend(seq.iter().cloned());
        }

        let current_score = pre_scores.evaluate(&key_sequence) * conjunction.appearances as u64;
        score += current_score;
        score_obj.evaluated.push(Evaluated {
            score: current_score,
            conjunction: conjunction.clone(),
        });
        key_sequence.clear()
    }

    score_obj.total_score = score;
    score_obj
}
