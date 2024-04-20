use std::collections::HashMap;

use crate::{char_def, keymap::Keymap};

/// キーの出現回数を記録するテーブル
#[derive(Debug)]
pub struct FrequencyTable {
    // 各キーごとに、どの文字がどれだけ出現したかを記録する
    //
    // キーの数として、シフトへの割当も１キーとしてカウントしている。シフト自体は +26 のオフセットとしている
    frequency: [[u64; 50]; 26],

    // 文字と頻度表におけるindexのマッピング
    character_map: HashMap<char, usize>,

    // 文字と頻度表におけるindexのマッピング
    character_index_map: HashMap<usize, char>,
}

impl FrequencyTable {
    /// 頻度表を新規に作成する。
    ///
    /// 確率が0にならないように、初期値は1としている
    pub fn new() -> Self {
        FrequencyTable {
            frequency: [[1; 50]; 26],
            character_map: char_def::definitions()
                .into_iter()
                .enumerate()
                .map(|(i, c)| (c.normal(), i))
                .collect(),
            character_index_map: char_def::definitions()
                .into_iter()
                .enumerate()
                .map(|(i, c)| (i, c.normal()))
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

        let total_availability = chars.iter().fold(0, |acc, c| {
            if let Some(c) = c {
                let key: char = c.into();
                acc + target_row[self.character_map[&key]]
            } else {
                acc
            }
        });

        for (idx, freq) in target_row.iter().enumerate() {
            let prob = *freq as f64 / total_availability as f64;

            if prob >= probability {
                return (idx, self.character_index_map[&idx].clone());
            }
        }

        return (0, self.character_index_map[&0].clone());
    }

    /// `keymap` にある文字から、頻度表を更新する
    ///
    /// このとき、全体のrankに対する順位を考慮する。1 - rank/total_keymaps が回数に加算される
    pub fn update(&mut self, keymap: &Keymap, rank: u64, total_keymaps: usize) {
        let coefficient = ((1.0 - rank as f64 / total_keymaps as f64) * 100.0 as f64) as u64;

        // シフトの場合でも同じキーへの割当として扱う
        for (c, idx) in self.character_map.iter() {
            if let Some((key_idx, _)) = keymap
                .iter()
                .enumerate()
                .find(|(_, k)| k.unshift() == Some(*c))
            {
                self.frequency[key_idx][*idx] += coefficient;
            }

            if let Some((key_idx, _)) = keymap
                .iter()
                .enumerate()
                .find(|(_, k)| k.shifted() == Some(*c))
            {
                self.frequency[key_idx][*idx] += coefficient;
            }
        }
    }
}
