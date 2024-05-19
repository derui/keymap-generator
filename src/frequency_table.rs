use std::collections::{HashMap, HashSet};

use rand::{rngs::StdRng, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    char_def::{self},
    frequency_layer::{LayeredCharCombination, LayeredFrequency, UsedKeyPool},
    keymap::Keymap,
    layout::linear::{linear_layout, LINEAR_L_SHIFT_INDEX, LINEAR_R_SHIFT_INDEX},
};

pub const NORMAL_LAYER: &str = "normal";
pub const SHIFT_LAYER: &str = "shift";
const LAYERS: [&str; 2] = [NORMAL_LAYER, SHIFT_LAYER];

/// キーの配置についての基本制約を頻度で表現し、それに追従するキーを返す構造体
/// この構造体は、FrequencyTable自体から作成される。
#[derive(Debug)]
pub struct KeyAssigner {
    /// 組み合わせの頻度。内容はCombinationFrequencyと同一である
    layered_combinations: Vec<LayeredFrequency>,
    // 文字からindexに変換するmap
    character_map: HashMap<char, usize>,

    key_pool: UsedKeyPool,

    key_predicates: HashMap<usize, Vec<fn(&LayeredCharCombination) -> bool>>,
}

impl KeyAssigner {
    /// `freq_table` から[KeyAssigner]を生成する
    pub fn from_freq(
        freq_table: &FrequencyTable,
        predicates: &HashMap<usize, Vec<fn(&LayeredCharCombination) -> bool>>,
    ) -> Self {
        let mut key_pool = HashSet::new();

        key_pool.insert(
            char_def::definitions()
                .iter()
                .position(|v| v.is_punctuation_mark())
                .expect("should be found punctuation mark"),
        );
        key_pool.insert(
            char_def::definitions()
                .iter()
                .position(|v| v.is_reading_point())
                .expect("should be found reading point"),
        );

        Self {
            layered_combinations: freq_table.frequency.to_vec(),
            character_map: freq_table.character_map.clone(),
            key_pool,
            key_predicates: predicates.clone(),
        }
    }

    /// 有効なキーが少ない順に並べられたkeyのindexを返す
    pub fn ordered_key_indices(&self, rng: &mut StdRng) -> Vec<usize> {
        let mut vec = self
            .layered_combinations
            .iter()
            .enumerate()
            .map(|v| v.0)
            .collect::<Vec<_>>();
        let random = vec![0; vec.len()]
            .iter()
            .map(|_| rng.gen::<i32>())
            .collect::<Vec<_>>();

        vec.sort_by(|v1, v2| random[*v1].cmp(&random[*v2]));

        vec
    }

    /// 指定された `key_idx` において、選択確率に応じた [LayeredCharCombination] を返す
    pub fn pick_key(&mut self, rng: &mut StdRng, key_idx: usize) -> LayeredCharCombination {
        let freq = &self.layered_combinations[key_idx];
        let preds = self.key_predicates.get(&key_idx).cloned().unwrap_or(vec![
            |v: &LayeredCharCombination| {
                let normal = v.char_of_layer(NORMAL_LAYER);
                let shift = v.char_of_layer(SHIFT_LAYER);

                match (
                    normal.and_then(|v| v.turbid()),
                    shift.and_then(|v| v.turbid()),
                ) {
                    (Some(_), Some(_)) => false,
                    _ => true,
                }
            },
            |v: &LayeredCharCombination| {
                let normal = v.char_of_layer(NORMAL_LAYER);
                let shift = v.char_of_layer(SHIFT_LAYER);

                match (
                    normal.and_then(|v| v.semiturbid()),
                    shift.and_then(|v| v.semiturbid()),
                ) {
                    (Some(_), Some(_)) => false,
                    _ => true,
                }
            },
        ]);

        let char = freq.get_assignment(rng, &self.key_pool, &Vec::new(), &preds);

        LAYERS.iter().for_each(|name| {
            if let Some(c) = char.char_of_layer(name) {
                self.key_pool.insert(self.character_map[&c.normal()]);
            }
        });

        char
    }

    /// 左シフトキーに対する組み合わせを返す。
    ///
    /// この関数は、right_shift_keyとセットで利用することを前提としている。
    pub fn left_shift_key(&mut self, rng: &mut StdRng) -> LayeredCharCombination {
        let freq = &self.layered_combinations[LINEAR_L_SHIFT_INDEX];

        let char = freq.get_assignment(
            rng,
            &self.key_pool,
            &Vec::new(),
            &[|comb: &LayeredCharCombination| {
                comb.char_of_layer(NORMAL_LAYER)
                    .map_or(true, |v| v.is_cleartone() && !v.is_sulphuric())
                    && comb
                        .char_of_layer(SHIFT_LAYER)
                        .map_or(false, |v| v.is_cleartone() && !v.is_sulphuric())
            }],
        );

        LAYERS.iter().for_each(|name| {
            if let Some(c) = char.char_of_layer(name) {
                self.key_pool.insert(self.character_map[&c.normal()]);
            }
        });

        char
    }

    /// 右シフトキーに対する組み合わせを返す。
    ///
    /// この関数は、left_shift_keyとセットで利用することを前提としている。
    pub fn right_shift_key(
        &mut self,
        rng: &mut StdRng,
        left_combination: &LayeredCharCombination,
    ) -> LayeredCharCombination {
        let freq = &self.layered_combinations[LINEAR_R_SHIFT_INDEX];

        // シフトキーは、シフト面が同一であることが要件になる。
        let preds = vec![|comb: &LayeredCharCombination| {
            comb.char_of_layer(NORMAL_LAYER)
                .map_or(true, |v| v.is_cleartone() && !v.is_sulphuric())
        }];

        let char = freq.get_assignment(
            rng,
            &self.key_pool,
            &[(SHIFT_LAYER, left_combination.char_of_layer(SHIFT_LAYER))],
            &preds,
        );

        LAYERS.iter().for_each(|name| {
            if let Some(c) = char.char_of_layer(name) {
                self.key_pool.insert(self.character_map[&c.normal()]);
            }
        });

        char
    }
}

/// 条件に一致する文字の組み合わせを返す

/// キーの出現回数を記録するテーブル
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrequencyTable {
    // 各キーごとに、どの文字がどれだけ出現したかを記録する
    //
    // キーの数として、シフトへの割当も１キーとしてカウントしている。
    frequency: Vec<LayeredFrequency>,

    // 文字と頻度表におけるindexのマッピング
    character_map: HashMap<char, usize>,
}

impl FrequencyTable {
    /// 頻度表を新規に作成する。
    pub fn new() -> Self {
        // 可能なキーの位置は26個なので、その分の分布を設定する
        // 句読点は特殊なキーに割り当てられるため、それらは除外する
        let combinations = vec![LayeredFrequency::new(&LAYERS); linear_layout().len()];

        FrequencyTable {
            frequency: combinations,
            character_map: char_def::definitions()
                .into_iter()
                .enumerate()
                .map(|(i, c)| (c.normal(), i))
                .collect(),
        }
    }

    /// `keymap` にある文字から、頻度表を更新する
    pub fn update(&mut self, best_keymap: &Keymap, learning_rate: f64) {
        for (key_idx, def) in best_keymap.iter().enumerate() {
            let normal_def = def.unshift_def();
            let shifted_def = def.shifted_def();
            let keys = vec![(NORMAL_LAYER, normal_def), (SHIFT_LAYER, shifted_def)];

            self.frequency[key_idx].update(&keys, learning_rate)
        }
    }

    /// `mutation_prob` に該当する確率で、各キーにおける分布に突然変異を与える
    pub fn mutate(&mut self, rng: &mut StdRng, mutation_prob: f64) {
        if rng.gen::<f64>() > mutation_prob {
            return;
        }

        self.frequency[rng.gen_range(0..linear_layout().len())].mutate(rng);
    }
}
