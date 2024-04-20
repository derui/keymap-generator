use std::sync::{mpsc::channel, Arc};

use rand::{rngs::StdRng, Rng};

use crate::{
    connection_score::ConnectionScore,
    keymap::Keymap,
    score::{self, Conjunction},
};

/// 遺伝的アルゴリズムを実行するための基盤を生成する
#[derive(Debug)]
pub struct Playground {
    generation: u64,
    // 全体で生成するkeymapの最大数
    gen_count: u8,
    // 全体で実行するworkerの最大数
    workers: usize,
    keymaps: Vec<Keymap>,
}

const MUTATION_PROPABILITY: f64 = 0.01;
const CROSS_PROPABILITY: f64 = 0.85;
const WORKERS: u8 = 20;

impl Playground {
    pub fn new(gen_count: u8, rng: &mut StdRng) -> Self {
        assert!(gen_count > 0, "gen_count must be greater than 0");

        // まずは必要な数だけ生成しておく
        let mut keymaps = Vec::new();
        while keymaps.len() < gen_count as usize {
            let keymap = Keymap::generate(rng);

            if keymap.meet_requirements() {
                keymaps.push(keymap);
            }
        }

        Playground {
            generation: 1,
            gen_count,
            keymaps,
            workers: WORKERS as usize,
        }
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// 世代を一つ進める。結果として、現世代でベストだったkeymapを返す
    ///
    /// 結果として、bestなscoreとkeymapを返す
    pub fn advance(
        &mut self,
        rng: &mut StdRng,
        conjunctions: &[Conjunction],
        pre_scores: Arc<Box<ConnectionScore>>,
    ) -> (u64, Keymap) {
        self.generation += 1;

        let mut new_keymaps = Vec::new();
        let rank = self.rank(conjunctions, pre_scores.clone()).to_vec();
        let probabilities = self.make_probabilities(&rank);

        // new_keymapsがgen_countになるまで繰り返す
        while new_keymaps.len() < self.gen_count as usize {
            let prob = rng.gen::<f64>();

            if prob < CROSS_PROPABILITY {
                // 交叉
                let keymap = self.select(rng, &rank, &probabilities);
                loop {
                    let new_keymap = keymap.imitate_cross(rng);

                    if new_keymap.meet_requirements() {
                        new_keymaps.push(new_keymap);
                        break;
                    }
                }
            } else {
                // 複製
                let keymap = self.select(rng, &rank, &probabilities);
                new_keymaps.push(keymap);
            }
        }

        let new_keymaps = new_keymaps
            .into_iter()
            .map(|keymap| {
                let prob = rng.gen::<f64>();
                if prob < MUTATION_PROPABILITY {
                    loop {
                        // 突然変異
                        let keymap = keymap.mutate(rng);

                        if keymap.meet_requirements() {
                            break keymap;
                        }
                    }
                } else {
                    keymap
                }
            })
            .collect();

        let best_keymap = self.keymaps[rank[0].1].clone();
        self.keymaps = new_keymaps;

        log::debug!(
            "generation: {}, best score is: {}",
            self.generation,
            rank[0].0,
        );

        (rank[0].0, best_keymap.clone())
    }

    /// rankから、それぞれが選択される確率を返す。サイズは常にrankと同じサイズである。
    ///
    /// rankは、scoreが低い順であるため、全体を逆に扱っている。
    fn make_probabilities(&self, rank: &[(u64, usize)]) -> Vec<f64> {
        let mut probs = vec![0.0; rank.len()];
        let total_score = rank.iter().map(|(score, _)| *score).sum::<u64>();

        for (idx, (score, _)) in rank.iter().enumerate() {
            probs[idx] = 1.0 - *score as f64 / total_score as f64;
        }

        probs.reverse();
        probs
    }

    /// 確率から選択されたkeymapを返す
    ///
    /// rankは、scoreが低い順であるため、全体を逆に扱っている。
    ///
    /// # Arguments
    /// * `rank` - rank
    /// * `probs` - 選択確率
    /// * `rng` - 乱数生成器
    ///
    /// # Returns
    /// 選択されたkeymap
    fn select(&self, rng: &mut StdRng, rank: &[(u64, usize)], probs: &[f64]) -> Keymap {
        let prob = rng.gen::<f64>();

        let mut idx = None;
        let mut prob_accum = 0.0;

        for (prob_idx, v) in probs.iter().enumerate() {
            prob_accum += *v;
            if prob_accum >= prob {
                idx = Some(prob_idx);
                break;
            }
        }

        self.keymaps[rank[idx.expect("should be found")].1].clone()
    }

    /// scoreに基づいてkeymapをランク付けする。
    ///
    /// この中から、全体の特定の%までに対して確率を按分する
    fn rank(
        &self,
        conjunctions: &[Conjunction],
        pre_scores: Arc<Box<ConnectionScore>>,
    ) -> Vec<(u64, usize)> {
        let pool = threadpool::ThreadPool::new(self.workers);

        let conjunctions = Arc::new(conjunctions.to_vec());

        let mut scores = Vec::new();
        let keymaps = self.keymaps.clone();
        let (tx, tr) = channel();

        keymaps.iter().enumerate().for_each(|(idx, k)| {
            let tx = tx.clone();
            let keymap = k.clone();
            let conjunctions = conjunctions.clone();
            let pre_scores = pre_scores.clone();

            pool.execute(move || {
                let score = score::evaluate(&conjunctions, &pre_scores, &keymap);
                tx.send((score, idx)).expect("should be success")
            })
        });

        for (score, idx) in tr.iter().take(self.keymaps.len()) {
            scores.push((score, idx));
        }

        scores.sort_by(|a, b| a.0.cmp(&b.0));
        scores
    }
}
