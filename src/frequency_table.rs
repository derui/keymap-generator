use std::collections::HashMap;

use rand::{rngs::StdRng, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    char_def::{self},
    keymap::Keymap,
};

/// 頻度表テーブルにおけるレイヤーを示す
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Layer {
    Unshift,
    Shift,
}

impl From<Layer> for usize {
    fn from(value: Layer) -> Self {
        match value {
            Layer::Unshift => UNSHIFT_LAYER,
            Layer::Shift => SHIFT_LAYER,
        }
    }
}

/// キーの出現回数を記録するテーブル
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrequencyTable {
    // 各キーごとに、どの文字がどれだけ出現したかを記録する
    //
    // キーの数として、シフトへの割当も１キーとしてカウントしている。シフト自体は +26 のオフセットとしている
    frequency: Vec<Vec<Vec<f64>>>,

    // 文字と頻度表におけるindexのマッピング
    character_map: HashMap<char, usize>,

    // 文字と頻度表におけるindexのマッピング
    character_index_map: Vec<char>,
}

const UNSHIFT_LAYER: usize = 0;
const SHIFT_LAYER: usize = 1;

impl FrequencyTable {
    /// 頻度表を新規に作成する。
    ///
    /// 一様な変更として認識するため、0.5で初期化している
    pub fn new() -> Self {
        FrequencyTable {
            // シフト面と無シフト面でそれぞれ別にする
            frequency: vec![vec![vec![0.5; 50]; 26]; 2],
            character_map: char_def::definitions()
                .into_iter()
                .enumerate()
                .map(|(i, c)| (c.normal(), i))
                .collect(),
            character_index_map: char_def::definitions()
                .into_iter()
                .map(|c| c.normal())
                .collect(),
        }
    }

    /// 対象のキーにおいて、対象の確率に対応する文字を返す
    ///
    /// # Arguments
    /// * `chars` - 選択対象の文字の一覧。使用できない場所はNone。 [char_def::definitions]から返却されるものと同じ順序でなければならない
    /// * `key_idx` - キーのindex
    /// * `rng` - 乱数生成機
    ///
    /// # Returns
    /// キーのindex、対象のlayer、対応するキー
    pub fn get_char<T>(
        &self,
        chars: &[Option<T>],
        key_idx: usize,
        rng: &mut StdRng,
    ) -> (usize, Layer, char) {
        let layer = if rng.gen() {
            Layer::Unshift
        } else {
            Layer::Shift
        };
        let target_row = &self.frequency[usize::from(layer)][key_idx];

        let total_availability = chars
            .iter()
            .enumerate()
            .map(|(idx, v)| match v {
                Some(_) => target_row[idx],
                None => 0.0,
            })
            .sum::<f64>();

        let prob: f64 = rng.gen();
        let mut freq_accum = 0.0;
        for (idx, target) in chars.iter().enumerate() {
            if target.is_some() {
                let freq = &self.frequency[usize::from(layer)][key_idx][idx] / total_availability;

                if (freq_accum + freq) >= prob {
                    return (idx, layer, self.character_index_map[idx]);
                }
                freq_accum += freq;
            }
        }

        // 一応最後のキーだけ取得する
        let idx = chars.iter().rposition(|v| v.is_some()).unwrap();
        (idx, layer, self.character_index_map[idx])
    }

    /// `keymap` にある文字から、頻度表を更新する
    pub fn update(&mut self, best_keymap: &Keymap, worst_keymap: &Keymap, learning_rate: f64) {
        let mut best_keymap_map = vec![vec![vec![false; 50]; 26]; 2];
        let mut worst_keymap_map = vec![vec![vec![false; 50]; 26]; 2];

        for (key_idx, def) in best_keymap.iter().enumerate() {
            if let Some(c_idx) = def.unshift().and_then(|v| self.character_map.get(&v)) {
                best_keymap_map[UNSHIFT_LAYER][key_idx][*c_idx] = true;
            }

            if let Some(c_idx) = def.shifted().and_then(|v| self.character_map.get(&v)) {
                best_keymap_map[SHIFT_LAYER][key_idx][*c_idx] = true;
            }
        }

        for (key_idx, def) in worst_keymap.iter().enumerate() {
            if let Some(c_idx) = def.unshift().and_then(|v| self.character_map.get(&v)) {
                worst_keymap_map[UNSHIFT_LAYER][key_idx][*c_idx] = true;
            }

            if let Some(c_idx) = def.shifted().and_then(|v| self.character_map.get(&v)) {
                worst_keymap_map[SHIFT_LAYER][key_idx][*c_idx] = true;
            }
        }

        // worstなキーについては、
        for (idx, layer) in best_keymap_map.iter().enumerate() {
            for (idx2, key) in layer.iter().enumerate() {
                for (idx3, ch) in key.iter().enumerate() {
                    if *ch == worst_keymap_map[idx][idx2][idx3] {
                        continue;
                    }

                    let freq = self.frequency[idx][idx2][idx3];
                    if *ch {
                        self.frequency[idx][idx2][idx3] = freq + 1.0;
                    } else {
                        self.frequency[idx][idx2][idx3] = freq - 0.5;
                    }
                }
            }
        }
    }

    /// 指定した確率で、頻度に対して突然変異を実施する
    ///
    /// ここでの突然変異は、それぞれの回数に対して特定の割合をランダムに加減するものである
    pub fn mutate(&mut self, rng: &mut StdRng, mutate_shift: &f64, mutate_prob: &f64) {
        for layer in self.frequency.iter_mut() {
            for row in layer.iter_mut() {
                for freq in row.iter_mut() {
                    if rng.gen::<f64>() < *mutate_prob {
                        let current = *freq;
                        let shift = if rng.gen() { 1.0 } else { 0.0 };

                        *freq = current * (1.0 - *mutate_shift) + (shift * *mutate_shift);
                    }
                }
            }
        }
    }
}
