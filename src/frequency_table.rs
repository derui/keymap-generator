use std::collections::{HashMap, HashSet};

use rand::{rngs::StdRng, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    char_def::{self, definitions, CharDef},
    frequency_layer::{LayeredCharCombination, LayeredFrequency, UsedKeyPool},
    keymap::Keymap,
    layout::linear::{linear_layout, LINEAR_L_SHIFT_INDEX, LINEAR_R_SHIFT_INDEX},
};

const LAYERS: [&str; 2] = ["normal", "shift"];

/// キーの配置についての基本制約を頻度で表現し、それに追従するキーを返す構造体
/// この構造体は、FrequencyTable自体から作成される。
#[derive(Debug)]
pub struct KeyAssigner {
    /// 組み合わせの頻度。内容はCombinationFrequencyと同一である
    layered_combinations: Vec<LayeredFrequency>,
    // 文字と頻度表におけるindexのマッピング
    character_index_map: Vec<CharDef>,
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
        Self {
            layered_combinations: freq_table.frequency.to_vec(),
            character_index_map: char_def::definitions().into_iter().collect(),
            character_map: freq_table.character_map.clone(),
            key_pool: HashSet::new(),
            key_predicates: predicates.clone(),
        }
    }

    /// 有効なキーが少ない順に並べられたkeyのindexを返す
    pub fn ordered_key_indices(&self) -> Vec<usize> {
        let vec = self
            .layered_combinations
            .iter()
            .enumerate()
            .map(|v| v.0)
            .collect::<Vec<_>>();

        vec
    }

    /// 指定された `key_idx` において、選択確率に応じた [CharCombination] を返す
    ///
    /// 選択されたキーに含まれている文字は、無シフト面の両方から使用できなくなる
    pub fn pick_key(&mut self, rng: &mut StdRng, key_idx: usize) -> LayeredCharCombination {
        let freq = &self.layered_combinations[key_idx];
        let preds = self
            .key_predicates
            .get(&key_idx)
            .cloned()
            .unwrap_or(Vec::new());

        let char = freq.get_assignment(rng, &self.key_pool, &Vec::new(), &preds);

        LAYERS.iter().for_each(|name| {
            if let Some(c) = char.char_of_layer(*name) {
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
            &vec![|comb: &LayeredCharCombination| {
                comb.char_of_layer("normal")
                    .map_or(true, |v| v.is_cleartone())
                    && comb
                        .char_of_layer("shift")
                        .map_or(true, |v| v.is_cleartone())
            }],
        );

        LAYERS.iter().for_each(|name| {
            if let Some(c) = char.char_of_layer(*name) {
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
            comb.char_of_layer("normal")
                .map_or(true, |v| v.is_cleartone())
                && comb
                    .char_of_layer("shift")
                    .map_or(true, |v| v.is_cleartone())
        }];
        let char = freq.get_assignment(
            rng,
            &self.key_pool,
            &vec![("shift", left_combination.char_of_layer("shift"))],
            &preds,
        );

        LAYERS.iter().for_each(|name| {
            if let Some(c) = char.char_of_layer(*name) {
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
        let combinations = vec![LayeredFrequency::new(&LAYERS); 26];

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
            let keys = vec![("normal", normal_def), ("shift", shifted_def)];

            self.frequency[key_idx].update(&keys, learning_rate)
        }
    }

    /// `mutation_prob` に該当する確率で、各キーにおける分布に突然変異を与える
    pub fn mutate(&mut self, rng: &mut StdRng, mutation_prob: f64) {
        if rng.gen::<f64>() > mutation_prob {
            return;
        }

        // self.frequency[rng.gen_range(0..linear_layout().len())].mutate(rng);
    }
}
