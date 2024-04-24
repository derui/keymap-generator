use std::collections::HashMap;

use rand::{rngs::StdRng, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    char_def::{self, definitions, CharDef},
    keymap::Keymap,
    layout::linear::{
        LINEAR_L_SEMITURBID_INDEX, LINEAR_L_SHIFT_INDEX, LINEAR_R_SEMITURBID_INDEX,
        LINEAR_R_SHIFT_INDEX,
    },
};

/// 存在する文字のシフト面と無シフト面に対する組み合わせにおける頻度を表す
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CombinationFrequency {
    /// 組み合わせの頻度。Noneの場合は、その組み合わせが存在しないということを表す
    combinations: Vec<Vec<Option<f64>>>,
}

impl CombinationFrequency {
    /// 対象の文字の組み合わせに対する頻度を返す
    ///
    /// # Arguments
    /// * `first` - 最初の文字
    /// * `second` - 2番目の文字
    ///
    /// # Returns
    /// 対象の文字の組み合わせに対する頻度
    pub fn frequency_at(&self, first: char, second: char) -> Option<f64> {
        let first_idx = definitions()
            .iter()
            .position(|v| v.normal() == first)
            .unwrap();
        let second_idx = definitions()
            .iter()
            .position(|v| v.normal() == second)
            .unwrap();
        self.combinations[first_idx][second_idx]
    }

    /// 全体の頻度を返す
    pub fn total_count(&self) -> f64 {
        self.combinations
            .iter()
            .map(|v| v.iter().map(|v| v.unwrap_or(0.0)).sum::<f64>())
            .sum::<f64>()
    }

    /// 指定された 組み合わせに対する確率を更新する
    fn update_frequency(&mut self, comb: (usize, usize)) {
        let (first_idx, second_idx) = comb;

        // 2次元配列自体は、unshift -> shiftで構成している
        let count = self
            .combinations
            .iter()
            .map(|v| v.iter().map(|v| v.map_or(0.0, |_| 1.0)).sum::<f64>())
            .sum::<f64>();

        for (ri, row) in self.combinations.iter_mut().enumerate() {
            for (ci, col) in row.iter_mut().enumerate() {
                let Some(v) = col else { continue };
                if ri == first_idx && ci == second_idx {
                    *v = (*v + 1.0).min(count - 1.0)
                } else {
                    *v = (*v * (1.0 - 1.0 / count)).max(0.0000001);
                }
            }
        }
    }

    /// 指定された `ch` を含む組み合わせを無効にする
    fn disable(&mut self, character_map: &HashMap<char, usize>, ch: char) {
        let ch_idx = character_map[&ch];

        // 2次元配列自体は、unshift -> shiftで構成している
        for v in self.combinations[ch_idx].iter_mut() {
            *v = None;
        }

        for row in self.combinations.iter_mut() {
            row[ch_idx] = None;
        }
    }

    /// 指定したpredicateに対応する組み合わせの頻度を生成する
    ///
    /// # Arguments
    /// * `pred` - 組み合わせの頻度を生成するための条件。1つ目の引数が無シフト面、2つ目の引数がシフト面を表す
    ///
    /// # Returns
    /// 対象の文字の組み合わせに対する頻度
    pub fn new<F>(pred: F) -> CombinationFrequency
    where
        F: Fn(&CharDef, &CharDef) -> bool,
    {
        let mut vec = vec![vec![None; definitions().len()]; definitions().len()];

        for (fst_idx, fst) in definitions().iter().enumerate() {
            for (snd_idx, snd) in definitions().iter().enumerate() {
                // 同一の文字はそもそも設定できない
                if fst_idx == snd_idx {
                    continue;
                }

                // 全体の前提として、清濁同置であるので、それを満たさない場合は無効とする
                if matches!((fst.turbid(), snd.turbid()), (Some(_), Some(_))) {
                    continue;
                }

                // 半濁音同士も配置できない
                if matches!((fst.semiturbid(), snd.semiturbid()), (Some(_), Some(_))) {
                    continue;
                }

                if pred(fst, snd) {
                    vec[fst_idx][snd_idx] = Some(1.0);
                }
            }
        }

        CombinationFrequency { combinations: vec }
    }
}

#[derive(Debug)]
pub struct CharCombination(CharDef, CharDef);

impl CharCombination {
    pub fn new(unshift: &CharDef, shifted: &CharDef) -> Self {
        Self(*unshift, *shifted)
    }

    pub fn unshift(&self) -> CharDef {
        self.0
    }

    pub fn shifted(&self) -> CharDef {
        self.1
    }
}

/// キーの配置についての基本制約を頻度で表現し、それに追従するキーを返す構造体
/// この構造体は、FrequencyTable自体から作成される。
#[derive(Debug)]
pub struct KeyAssigner {
    /// 組み合わせの頻度。内容はCombinationFrequencyと同一である
    combinations: Vec<(f64, CombinationFrequency)>,
    // 文字と頻度表におけるindexのマッピング
    character_index_map: Vec<CharDef>,
    // 文字からindexに変換するmap
    character_map: HashMap<char, usize>,
}

impl KeyAssigner {
    /// `freq_table` から[KeyAssigner]を生成する
    pub fn from_freq(freq_table: &FrequencyTable) -> Self {
        Self {
            combinations: freq_table
                .frequency
                .iter()
                .cloned()
                .map(|v| (v.total_count(), v))
                .collect(),
            character_index_map: freq_table.character_index_map.clone(),
            character_map: freq_table.character_map.clone(),
        }
    }

    /// 有効なキーが少ない順に並べられたkeyのindexを返す
    pub fn ordered_key_indices(&self) -> Vec<usize> {
        let mut vec = self.combinations.iter().enumerate().collect::<Vec<_>>();

        vec.sort_by(|(_, (_, comb1)), (_, (_, comb2))| {
            let number_of_none1 = comb1
                .combinations
                .iter()
                .flatten()
                .filter(|v| v.is_none())
                .count();
            let number_of_none2 = comb2
                .combinations
                .iter()
                .flatten()
                .filter(|v| v.is_none())
                .count();

            number_of_none1.cmp(&number_of_none2)
        });
        vec.into_iter().map(|(v, _)| v).collect()
    }

    /// 指定された `key_idx` において、選択確率に応じた [CharCombination] を返す
    ///
    /// 選択されたキーに含まれている文字は、無シフト面の両方から使用できなくなる
    pub fn pick_key(&mut self, rng: &mut StdRng, key_idx: usize) -> Option<CharCombination> {
        let (total, freq) = &self.combinations[key_idx];

        let prob = rng.gen::<f64>();
        let mut accum = 0.0;
        for (first_idx, first) in freq.combinations.iter().enumerate() {
            for (second_idx, second) in first.iter().enumerate() {
                let Some(second) = second else { continue };

                if accum + (*second / *total) >= prob {
                    for (total, freq) in self.combinations.iter_mut() {
                        freq.disable(
                            &self.character_map,
                            self.character_index_map[first_idx].normal(),
                        );
                        freq.disable(
                            &self.character_map,
                            self.character_index_map[second_idx].normal(),
                        );

                        *total = freq.total_count();
                    }

                    return Some(CharCombination(
                        self.character_index_map[first_idx],
                        self.character_index_map[second_idx],
                    ));
                }
                accum += *second / *total;
            }
        }

        None
    }

    /// 左シフトキーに対する組み合わせを返す。
    ///
    /// この関数は、right_shift_keyとセットで利用することを前提としている。
    pub fn left_shift_key(&self, rng: &mut StdRng) -> CharCombination {
        let (total, freq) = &self.combinations[LINEAR_L_SHIFT_INDEX];

        let _prob = rng.gen::<f64>();
        let mut accum = 0.0;
        for (first_idx, first) in freq.combinations.iter().enumerate() {
            for (second_idx, second) in first.iter().enumerate() {
                let Some(second) = second else { continue };

                if accum + (*second / *total) >= accum {
                    return CharCombination(
                        self.character_index_map[first_idx],
                        self.character_index_map[second_idx],
                    );
                }
                accum += *second / *total;
            }
        }

        unreachable!("do not come here");
    }

    /// 左シフトキーに対する組み合わせを返す。
    ///
    /// この関数は、right_shift_keyとセットで利用することを前提としている。
    pub fn right_shift_key(
        &mut self,
        rng: &mut StdRng,
        left_combination: &CharCombination,
    ) -> CharCombination {
        // シフトキーは、シフト面が同一であることが要件になる。
        let shift_idx = self.character_map[&left_combination.shifted().normal()];

        // まず無シフト面で同じ文字を選択できないようにする
        for (total, freq) in self.combinations.iter_mut() {
            freq.disable(&self.character_map, left_combination.unshift().normal());

            *total = freq.total_count();
        }

        let (_, freq) = &self.combinations[LINEAR_R_SHIFT_INDEX];

        // シフト面が同一のキーだけに絞る
        let mut total = 0.0;
        for row in freq.combinations.iter() {
            total += row[shift_idx].unwrap_or(0.0);
        }

        let prob = rng.gen::<f64>();
        let mut accum = 0.0;
        for (first_idx, first) in freq.combinations.iter().enumerate() {
            let Some(second) = first[shift_idx] else {
                continue;
            };

            if accum + (second / total) >= prob {
                // 選択できたら、対象の場所を全体からdisableする
                for (total, freq) in self.combinations.iter_mut() {
                    freq.disable(
                        &self.character_map,
                        self.character_index_map[first_idx].normal(),
                    );
                    freq.disable(&self.character_map, left_combination.shifted().normal());

                    *total = freq.total_count();
                }
                // println!("first {:?}", &self.combinations[0]);

                return CharCombination::new(
                    &self.character_index_map[first_idx],
                    &self.character_index_map[shift_idx],
                );
            }
            accum += second / total;
        }

        unreachable!("do not come here");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_char_combinations_always_same_order() {
        // arrange
        let order1 = super::CombinationFrequency::new(|_, _| true);
        let order2 = super::CombinationFrequency::new(|_, _| true);

        // act

        // assert
        assert!(order1 == order2, "all eleemnts should be same")
    }

    #[test]
    fn same_key_combination_is_all_disabled() {
        // arrange
        let orders = super::CombinationFrequency::new(|_, _| true);

        // act
        let ret = definitions()
            .iter()
            .all(|c| orders.frequency_at(c.normal(), c.normal()).is_none());

        // assert
        assert!(ret, "all same-char combinations are disabled")
    }

    #[test]
    fn turbid_combination_is_disabled() {
        // arrange
        let orders = super::CombinationFrequency::new(|_, _| true);

        // act
        let ret = orders.frequency_at('か', 'し').is_none();

        // assert
        assert!(ret, "all same-char combinations are disabled")
    }
}

/// 条件に一致する文字の組み合わせを返す

/// キーの出現回数を記録するテーブル
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrequencyTable {
    // 各キーごとに、どの文字がどれだけ出現したかを記録する
    //
    // キーの数として、シフトへの割当も１キーとしてカウントしている。シフト自体は +26 のオフセットとしている
    frequency: Vec<CombinationFrequency>,

    // 文字と頻度表におけるindexのマッピング
    character_map: HashMap<char, usize>,

    // 文字と頻度表におけるindexのマッピング
    character_index_map: Vec<CharDef>,
}

impl FrequencyTable {
    /// 頻度表を新規に作成する。
    ///
    /// 一様な変更として認識するため、0.5で初期化している
    pub fn new() -> Self {
        // 可能なキーの位置は26個なので、その分の分布を設定する
        let mut combinations = vec![CombinationFrequency::new(|_, _| true); 26];

        // シフトキーのシフト面に対しては、清音しか許容できない。
        combinations[LINEAR_L_SHIFT_INDEX] = CombinationFrequency::new(|_, ch2| ch2.is_cleartone());
        combinations[LINEAR_R_SHIFT_INDEX] = CombinationFrequency::new(|_, ch2| ch2.is_cleartone());

        FrequencyTable {
            frequency: combinations,
            character_map: char_def::definitions()
                .into_iter()
                .enumerate()
                .map(|(i, c)| (c.normal(), i))
                .collect(),
            character_index_map: char_def::definitions().into_iter().collect(),
        }
    }

    /// `keymap` にある文字から、頻度表を更新する
    pub fn update(&mut self, best_keymap: &Keymap, worst_keymap: &Keymap, learning_rate: f64) {
        let mut checked_in_worst =
            vec![vec![vec![false; definitions().len()]; definitions().len()]; 26];

        // 構成上、すべてのキーがshift/unshiftを持っている
        for (key_idx, def) in worst_keymap.iter().enumerate() {
            let unshift_idx = self.character_map[&def.unshift()];
            let shift_idx = self.character_map[&def.shifted()];

            checked_in_worst[key_idx][unshift_idx][shift_idx] = true;
        }

        for (key_idx, def) in best_keymap.iter().enumerate() {
            let unshift_idx = self.character_map[&def.unshift()];
            let shift_idx = self.character_map[&def.shifted()];

            if checked_in_worst[key_idx][unshift_idx][shift_idx] {
                continue;
            }

            self.frequency[key_idx].update_frequency((unshift_idx, shift_idx))
        }
    }
}
