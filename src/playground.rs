use std::sync::{mpsc::channel, Arc};

use rand::{rngs::StdRng, Rng, SeedableRng};

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
    keymaps: Vec<Keymap>,

    frequency_table: FrequencyTable,
    pool: threadpool::ThreadPool,
}

const TOURNAMENT_SIZE: usize = 10;
const KEYMAP_SIZE: usize = 30;
const WORKERS: u8 = 12;
const MUTATION_PROB: f64 = 0.30;
const MUTATION_RATE: f64 = 0.05;

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
            keymaps,
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
        connection_score: Arc<ConnectionScore>,
        _conjunctions_2gram: &[Conjunction],
    ) -> (u64, Keymap) {
        self.generation += 1;

        let rank = self.rank(conjunctions, connection_score.clone()).to_vec();
        let mut picked_keymaps = Vec::new();
        // self.keymapsを個体と見立てて、確率分布を更新する
        for (rank, idx) in self.take_ranks(rng, &rank, TOURNAMENT_SIZE).iter() {
            self.frequency_table
                .update(&self.keymaps[*idx], 1.0 / (*rank + 1) as f64);
            picked_keymaps.push(self.keymaps[*idx].clone());
        }
        self.frequency_table
            .mutate(rng, MUTATION_PROB, MUTATION_RATE);

        // 確率分布から生成する
        let (tx, tr) = channel();

        (0..(KEYMAP_SIZE - TOURNAMENT_SIZE)).for_each(|_| {
            let tx = tx.clone();
            let frequency_table = self.frequency_table.clone();
            let mut rng = StdRng::seed_from_u64(rng.gen());

            self.pool.execute(move || loop {
                let mut assigner = KeyAssigner::from_freq(&frequency_table);
                if let Some(new_keymap) = Keymap::generate(&mut rng, &mut assigner) {
                    tx.send(new_keymap).unwrap();
                    break;
                }
            })
        });

        let new_keymaps: Vec<Keymap> = tr.iter().take(KEYMAP_SIZE - TOURNAMENT_SIZE).collect();
        let best_keymap = self.keymaps[rank[0].1].clone();
        self.keymaps = new_keymaps;
        self.keymaps.extend_from_slice(&picked_keymaps);

        (rank[0].0, best_keymap)
    }

    /// 最近傍探索をして、類似keymapのなかでbestなものを探す
    fn re_rank_neighbor(
        &self,
        conjunctions: &[Conjunction],
        connection_score: Arc<ConnectionScore>,
        keymap: &Keymap,
    ) -> Keymap {
        let conjunctions = Arc::new(conjunctions.to_vec());

        let (tx, tr) = channel();
        let mut keymaps: Vec<Keymap> = Vec::with_capacity(5000);

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
            let pre_scores = connection_score.clone();

            self.pool.execute(move || {
                let score = score::evaluate(&conjunctions, &pre_scores, &k);
                tx.send((score, idx)).expect("should be success")
            })
        });

        let mut scores: Vec<(u64, usize)> = tr.iter().take(keymaps.len()).collect();
        scores.sort_by(|a, b| a.0.cmp(&b.0));
        keymaps[scores[0].1].to_owned()
    }

    /// 指定した `count` の個数分 `rank` から取得する
    ///
    /// 返却されるtupleの0がrank、1が実際のindexである
    fn take_ranks(
        &self,
        rng: &mut StdRng,
        rank: &[(u64, usize)],
        count: usize,
    ) -> Vec<(usize, usize)> {
        let mut rank = Vec::from_iter(rank.iter().cloned());
        let mut ret = vec![];
        let mut rest = count;

        while rest > 0 && !rank.is_empty() {
            let mut accum = 0.0;
            let prob = rng.gen::<f64>();

            for idx in 0..rank.len() {
                if accum + (0.5 / (idx + 1) as f64) >= prob {
                    let (_, rank) = rank.remove(idx);
                    ret.push((idx, rank));
                    rest -= 1;
                    break;
                }
                accum += 0.5 / (idx + 1) as f64
            }
        }

        ret
    }

    /// scoreに基づいてkeymapをランク付けする。
    ///
    /// この中から、全体の特定の%までに対して確率を按分する
    fn rank(
        &self,
        conjunctions: &[Conjunction],
        connection_score: Arc<ConnectionScore>,
    ) -> Vec<(u64, usize)> {
        let conjunctions = Arc::new(conjunctions.to_vec());

        let keymaps = self.keymaps.clone();
        let (tx, tr) = channel();

        keymaps.into_iter().enumerate().for_each(|(idx, k)| {
            let tx = tx.clone();
            let conjunctions = conjunctions.clone();
            let pre_scores = connection_score.clone();

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
