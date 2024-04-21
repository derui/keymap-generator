use std::collections::HashMap;

use rand::{rngs::StdRng, Rng};

use crate::{char_def, keymap::Keymap};

/// キーの出現回数を記録するテーブル
#[derive(Debug)]
pub struct FrequencyTable {
    // 各キーごとに、どの文字がどれだけ出現したかを記録する
    //
    // キーの数として、シフトへの割当も１キーとしてカウントしている。シフト自体は +26 のオフセットとしている
    frequency: [[f64; 50]; 26],

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
            frequency: [[1.0 / 50.0; 50]; 26],
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
        probability: f64,
    ) -> (usize, char)
    where
        for<'a> char: From<&'a T>,
    {
        let target_row = self.frequency[key_idx];

        let total_availability = chars.iter().fold(0.0, |acc, c| {
            if let Some(c) = c {
                let key: char = c.into();
                acc + target_row[self.character_map[&key]]
            } else {
                acc
            }
        });

        let mut past_freq = 0.0;
        for (idx, target) in chars.iter().enumerate() {
            if target.is_some() {
                let freq = &self.frequency[key_idx][idx];
                let prob = (past_freq + *freq) / total_availability;

                if prob >= probability {
                    return (idx, self.character_index_map[idx]);
                }
                past_freq += *freq;
            }
        }

        unreachable!("should return some character before comes here")
    }

    /// `keymap` にある文字から、頻度表を更新する
    ///
    /// このとき、全体のrankに対する順位を考慮する。1.0 / rank  が回数に加算される
    pub fn update(&mut self, keymap: &Keymap, rank: usize) {
        let coefficient = 1.0 / (1.0 + rank as f64) * 10.0;

        // シフトの場合でも同じキーへの割当として扱う。このため、若干歪な頻度になる。
        for (key_idx, def) in keymap.iter().enumerate() {
            if let Some((c_idx)) = def.unshift().and_then(|v| self.character_map.get(&v)) {
                self.frequency[key_idx][*c_idx] += coefficient;
            }

            if let Some((c_idx)) = def.shifted().and_then(|v| self.character_map.get(&v)) {
                self.frequency[key_idx][*c_idx] += coefficient;
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
                    let shift = if rng.gen() { current } else { 0.0 };

                    *freq = current * (1.0 - *mutate_shift) + (shift * *mutate_shift);
                }
            }
        }
    }
}
