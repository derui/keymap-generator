use std::sync::mpsc::channel;

use rand::{rngs::StdRng, Rng};

use crate::{
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

const CROSS_PROPABILITY: f64 = 0.05;
const MUTATION_PROPABILITY: f64 = 0.01;
const SAVE_PERCENT: f64 = 0.3;
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
    pub fn advance(&mut self, rng: &mut StdRng, conjunctions: &[Conjunction]) -> (u64, Keymap) {
        self.generation += 1;

        let mut new_keymaps = Vec::new();
        let rank = self
            .rank(conjunctions)
            .iter()
            .take((self.gen_count as f64 * SAVE_PERCENT) as usize)
            .cloned()
            .collect::<Vec<_>>();
        let select_prob = self.make_probabilities(&rank);

        // new_keymapsがgen_countになるまで繰り返す
        while new_keymaps.len() < self.gen_count as usize {
            let prob = rng.gen::<f64>();

            if prob < CROSS_PROPABILITY {
                // 交叉
                let mut map1 = self.select(rng, &rank, &select_prob);
                let mut map2 = self.select(rng, &rank, &select_prob);

                map1.cross(&mut map2, rng);

                if map1.meet_requirements() {
                    new_keymaps.push(map1);
                }

                if map2.meet_requirements() {
                    new_keymaps.push(map2);
                }
            } else if prob < CROSS_PROPABILITY + MUTATION_PROPABILITY {
                // 突然変異
                let keymap = self.select(rng, &rank, &select_prob).mutate(rng);

                if keymap.meet_requirements() {
                    new_keymaps.push(keymap);
                }
            } else {
                // 複製
                let keymap = self.select(rng, &rank, &select_prob);
                new_keymaps.push(keymap);
            }
        }

        let best_keymap = self.keymaps[rank[0].1].clone();
        self.keymaps = new_keymaps;

        log::info!(
            "generation: {}, best score is: {}",
            self.generation,
            rank[0].0
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
    fn rank(&self, conjunctions: &[Conjunction]) -> Vec<(u64, usize)> {
        let pool = threadpool::ThreadPool::new(self.workers);

        let mut scores = Vec::new();
        let keymaps = self.keymaps.clone();
        let (tx, tr) = channel();

        keymaps.iter().enumerate().for_each(|(idx, k)| {
            let tx = tx.clone();
            let keymap = k.clone();
            let conjunctions: Vec<Conjunction> = conjunctions.to_vec();

            pool.execute(move || {
                let score = score::evaluate(&conjunctions, &keymap);
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