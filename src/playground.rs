use std::sync::{mpsc::channel, Arc};

use rand::rngs::StdRng;

use crate::{
    connection_score::ConnectionScore,
    frequency_table::{FrequencyTable, KeyAssigner},
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

    frequency_table: FrequencyTable,
    pool: threadpool::ThreadPool,
}

const WORKERS: u8 = 20;
const MUTATE_RATE: f64 = 0.02;
const MUTATE_SHIFT: f64 = 0.05;
const LEARNING_RATE: f64 = 0.1;

impl Playground {
    pub fn new(gen_count: u8, rng: &mut StdRng, frequency_table: FrequencyTable) -> Self {
        assert!(gen_count > 0, "gen_count must be greater than 0");

        // まずは必要な数だけ生成しておく
        let mut keymaps = Vec::new();
        while keymaps.len() < 2 {
            let mut assigner = KeyAssigner::from_freq(&frequency_table);
            if let Some(keymap) = Keymap::generate(rng, &mut assigner) {
                keymaps.push(keymap);
            }
        }

        Playground {
            pool: threadpool::ThreadPool::new(WORKERS as usize),
            generation: 1,
            gen_count,
            keymaps,
            workers: WORKERS as usize,
            frequency_table,
        }
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn frequency_table(&self) -> FrequencyTable {
        self.frequency_table.clone()
    }

    /// 世代を一つ進める。結果として、現世代でベストだったkeymapを返す
    ///
    /// 内部実装としては、分布表の更新と生成が主になるので、PBILと同類の動きである
    /// 結果として、今回の中でbestなscoreとkeymapを返す
    pub fn advance(
        &mut self,
        rng: &mut StdRng,
        conjunctions: &[Conjunction],
        pre_scores: Arc<ConnectionScore>,
    ) -> (u64, Keymap) {
        self.generation += 1;

        let mut new_keymaps = Vec::new();
        let rank = self.rank(conjunctions, pre_scores.clone()).to_vec();

        // new_keymapsがgen_countになるまで繰り返す
        while new_keymaps.len() < 2 {
            let mut assigner = KeyAssigner::from_freq(&self.frequency_table);
            if let Some(new_keymap) = Keymap::generate(rng, &mut assigner) {
                new_keymaps.push(new_keymap);
            }
        }

        // 最良のkeymapを遺伝子として見立て、頻度表を更新する
        let (_, best_idx) = rank.first().expect("should be success");
        let best_keymap =
            self.re_rank_neighbor(conjunctions, pre_scores.clone(), &self.keymaps[*best_idx]);
        let (_, worst_idx) = rank.iter().last().expect("should be success");
        self.frequency_table.update(
            &best_keymap,
            &self.keymaps[*worst_idx],
            1.0 / self.gen_count as f64,
        );

        let best_keymap = self.keymaps[rank[0].1].clone();
        self.keymaps = new_keymaps;

        (rank[0].0, best_keymap.clone())
    }

    /// 最近傍探索をして、類似keymapのなかでbestなものを探す
    fn re_rank_neighbor(
        &self,
        conjunctions: &[Conjunction],
        pre_scores: Arc<ConnectionScore>,
        keymap: &Keymap,
    ) -> Keymap {
        let conjunctions = Arc::new(conjunctions.to_vec());

        let (tx, tr) = channel();
        let mut keymaps: Vec<Keymap> = Vec::new();

        for (i, _) in keymap.iter().enumerate() {
            for (j, _) in keymap.iter().enumerate() {
                if i >= j {
                    continue;
                }

                let swaps = keymap.swap_keys(i, j);
                keymaps.extend_from_slice(&swaps);
            }
        }

        keymaps.iter().enumerate().for_each(|(idx, k)| {
            let k = k.clone();
            let tx = tx.clone();
            let conjunctions = conjunctions.clone();
            let pre_scores = pre_scores.clone();

            self.pool.execute(move || {
                let score = score::evaluate(&conjunctions, &pre_scores, &k);
                tx.send((score, idx)).expect("should be success")
            })
        });

        let mut scores: Vec<(u64, usize)> = tr.iter().take(keymaps.len()).collect();
        scores.sort_by(|a, b| a.0.cmp(&b.0));
        keymaps[scores[0].1].clone()
    }

    /// scoreに基づいてkeymapをランク付けする。
    ///
    /// この中から、全体の特定の%までに対して確率を按分する
    fn rank(
        &self,
        conjunctions: &[Conjunction],
        pre_scores: Arc<ConnectionScore>,
    ) -> Vec<(u64, usize)> {
        let conjunctions = Arc::new(conjunctions.to_vec());

        let keymaps = self.keymaps.clone();
        let (tx, tr) = channel();

        keymaps.into_iter().enumerate().for_each(|(idx, k)| {
            let tx = tx.clone();
            let conjunctions = conjunctions.clone();
            let pre_scores = pre_scores.clone();

            self.pool.execute(move || {
                let score = score::evaluate(&conjunctions, &pre_scores, &k);
                tx.send((score, idx)).expect("should be success")
            })
        });

        let mut scores: Vec<(u64, usize)> = tr.iter().take(self.keymaps.len()).collect();
        scores.sort_by(|a, b| a.0.cmp(&b.0));
        scores
    }
}
