use std::collections::HashMap;

use rand::{rngs::StdRng, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    char_def::{self},
    keymap::Keymap,
};

/// キーの出現回数を記録するテーブル
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrequencyTable {
    // 各キーごとに、どの文字がどれだけ出現したかを記録する
    //
    // キーの数として、シフトへの割当も１キーとしてカウントしている。シフト自体は +26 のオフセットとしている
    frequency: Vec<Vec<f64>>,

    // 文字と頻度表におけるindexのマッピング
    character_map: HashMap<char, usize>,

    // 文字と頻度表におけるindexのマッピング
    character_index_map: Vec<char>,
}

impl FrequencyTable {
    /// 頻度表を新規に作成する。
    ///
    /// 確率が0にならないように、初期値は1としている
    pub fn new() -> Self {
        FrequencyTable {
            frequency: vec![vec![0.5; 50]; 26],
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
    /// * `probability` - 確率。0.0から1.0の間でなければならない
    ///
    /// # Returns
    /// 対象のキーにおいて、対象の確率に対応する文字
    pub fn get_char<T>(
        &self,
        chars: &[Option<T>],
        key_idx: usize,
        rng: &mut StdRng,
    ) -> (usize, char)
    where
        for<'a> char: From<&'a T>,
    {
        for (idx, target) in chars.iter().enumerate() {
            if target.is_some() {
                let freq = &self.frequency[key_idx][idx];

                if *freq >= rng.gen() {
                    return (idx, self.character_index_map[idx]);
                }
            }
        }

        // 一応最後のキーだけ取得する
        let idx = chars.iter().rposition(|v| v.is_some()).unwrap();
        (idx, self.character_index_map[idx])
    }

    /// `keymap` にある文字から、頻度表を更新する
    ///
    /// bestのみを考慮するのだが、ここでは確率のみを更新する。あるキーで選択された文字については、0.5として計算する
    pub fn update(&mut self, keymap: &Keymap, learning_rate: f64) {
        // シフトの場合でも同じキーへの割当として扱う。このため、若干歪な頻度になる。
        for (key_idx, def) in keymap.iter().enumerate() {
            let mut updated = Vec::new();
            if let Some(c_idx) = def.unshift().and_then(|v| self.character_map.get(&v)) {
                let freq = self.frequency[key_idx][*c_idx];
                self.frequency[key_idx][*c_idx] =
                    freq * (1.0 - learning_rate) + 0.5 * learning_rate;
                updated.push(*c_idx);
            }

            if let Some(c_idx) = def.shifted().and_then(|v| self.character_map.get(&v)) {
                let freq = self.frequency[key_idx][*c_idx];
                self.frequency[key_idx][*c_idx] =
                    freq * (1.0 - learning_rate) + 0.5 * learning_rate;
                updated.push(*c_idx);
            }

            // この2つのキー以外については、rateが0であるとして更新する
            for (idx, _) in self.frequency[key_idx].clone().iter().enumerate() {
                if updated.contains(&idx) {
                    continue;
                }

                self.frequency[key_idx][idx] *= 1.0 - (learning_rate + 0.075)
            }
        }
    }

    /// 指定した確率で、頻度に対して突然変異を実施する
    ///
    /// ここでの突然変異は、それぞれの回数に対して特定の割合をランダムに加減するものである
    pub fn mutate(&mut self, rng: &mut StdRng, mutate_shift: &f64, mutate_prob: &f64) {
        for row in self.frequency.iter_mut() {
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
