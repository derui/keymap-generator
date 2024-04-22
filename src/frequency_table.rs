use std::collections::HashMap;

use rand::{rngs::StdRng, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    char_def::{self, all_chars, definitions, CharDef},
    keymap::Keymap,
    layout::linear::{
        LINEAR_L_SEMITURBID_INDEX, LINEAR_L_SHIFT_INDEX, LINEAR_R_SEMITURBID_INDEX,
        LINEAR_R_SHIFT_INDEX,
    },
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
        let first_idx = all_chars().iter().position(|v| *v == first).unwrap();
        let second_idx = all_chars().iter().position(|v| *v == second).unwrap();
        self.combinations[first_idx][second_idx]
    }

    /// 全体の頻度を返す
    pub fn total_count(&self) -> f64 {
        self.combinations
            .iter()
            .map(|v| v.iter().map(|v| v.unwrap_or(0.0)).sum::<f64>())
            .sum::<f64>()
    }

    /// 指定された `ch` を含む組み合わせを無効にする
    fn disable(&mut self, character_map: &HashMap<char, usize>, ch: char) {
        let ch_idx = character_map[&ch];

        // 2次元配列自体は、unshift -> shiftで構成している
        for (idx, v) in self.combinations[ch_idx].iter_mut().enumerate() {
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
        let mut vec = vec![vec![None; 51]; 51];

        for (fst_idx, fst) in definitions().iter().enumerate() {
            for (snd_idx, snd) in definitions().iter().enumerate() {
                // 同一の文字はそもそも設定できない
                if fst_idx == snd_idx {
                    continue;
                }

                // 全体の前提として、清濁同置であるので、それを満たさない場合は無効とする
                if !fst.is_cleartone() && !snd.is_cleartone() {
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

pub struct CharCombination(CharDef, CharDef);

impl CharCombination {
    pub fn unshift(&self) -> CharDef {
        self.0
    }

    pub fn shifted(&self) -> CharDef {
        self.1
    }
}

/// キーの配置についての基本制約を頻度で表現し、それに追従するキーを返す構造体
/// この構造体は、FrequencyTable自体から作成される。
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
    fn from_freq(freq_table: &FrequencyTable) -> Self {
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

    /// 指定された `key_idx` において、選択確率に応じた [CharCombination] を返す
    ///
    /// 選択されたキーに含まれている文字は、無シフト面の両方から使用できなくなる
    pub fn pick_key(&self, rng: &mut StdRng, key_idx: usize) -> CharCombination {
        let (total, freq) = self.combinations[key_idx];

        let prob = rng.gen::<f64>();
        let mut accum = 0.0;
        for (first_idx, first) in freq.combinations.iter().enumerate() {
            for (second_idx, second) in first.iter().enumerate() {
                let Some(second) = second else { continue };

                if accum + (*second as f64 / total as f64) >= accum {
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

                    return CharCombination(
                        self.character_index_map[first_idx],
                        self.character_index_map[second_idx],
                    );
                }
                accum += *second as f64 / total as f64;
            }
        }

        unreachable!("do not come here");
    }

    /// 左シフトキーに対する組み合わせを返す。
    ///
    /// この関数は、right_shift_keyとセットで利用することを前提としている。
    pub fn left_shift_key(&self, rng: &mut StdRng) -> CharCombination {
        let (total, freq) = self.combinations[LINEAR_L_SHIFT_INDEX];

        let prob = rng.gen::<f64>();
        let mut accum = 0.0;
        for (first_idx, first) in freq.combinations.iter().enumerate() {
            for (second_idx, second) in first.iter().enumerate() {
                let Some(second) = second else { continue };

                if accum + (*second as f64 / total as f64) >= accum {
                    return CharCombination(
                        self.character_index_map[first_idx],
                        self.character_index_map[second_idx],
                    );
                }
                accum += *second as f64 / total as f64;
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
        let unshift_idx = self.character_map[&left_combination.unshift().normal()];
        let shift_idx = self.character_map[&left_combination.shifted().normal()];

        // まずshift面で同じ文字を選択できないようにする
        for (total, freq) in self.combinations.iter_mut() {
            freq.disable(&self.character_map, left_combination.unshift().normal());

            *total = freq.total_count();
        }

        let (_, freq) = self.combinations[LINEAR_R_SHIFT_INDEX];

        // シフト面が同一のキーだけに絞る
        let mut total = 0.0;
        for row in freq.combinations.iter() {
            for (idx, col) in row.iter().enumerate() {
                if idx != shift_idx {
                    continue;
                }
                total += col.unwrap_or(0.0);
            }
        }

        let prob = rng.gen::<f64>();
        let mut accum = 0.0;
        for (first_idx, first) in freq.combinations.iter().enumerate() {
            for (second_idx, second) in first.iter().enumerate() {
                if second_idx != shift_idx {
                    continue;
                };
                let Some(second) = second else { continue };

                if accum + (*second as f64 / total as f64) >= accum {
                    // 選択できたら、対象の場所を全体からdisableする
                    for (total, freq) in self.combinations.iter_mut() {
                        freq.disable(
                            &self.character_map,
                            self.character_index_map[first_idx].normal(),
                        );
                        freq.disable(&self.character_map, left_combination.shifted().normal());

                        *total = freq.total_count();
                    }

                    return CharCombination(
                        self.character_index_map[first_idx],
                        self.character_index_map[second_idx],
                    );
                }
                accum += *second as f64 / total as f64;
            }
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
        let ret = all_chars()
            .iter()
            .all(|c| orders.frequency_at(*c, *c).is_none());

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

const UNSHIFT_LAYER: usize = 0;
const SHIFT_LAYER: usize = 1;

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

        // 半濁音キー自体には、半濁音および濁音を持つ文字を割り当てない。
        combinations[LINEAR_L_SEMITURBID_INDEX] = CombinationFrequency::new(|ch1, ch2| {
            !(matches!(
                (
                    ch1.semiturbid(),
                    ch2.semiturbid(),
                    ch1.turbid(),
                    ch2.turbid()
                ),
                (Some(_), _, _, _) | (_, Some(_), _, _) | (_, _, Some(_), _) | (_, _, _, Some(_))
            ))
        });
        combinations[LINEAR_R_SEMITURBID_INDEX] = CombinationFrequency::new(|ch1, ch2| {
            !(matches!(
                (
                    ch1.semiturbid(),
                    ch2.semiturbid(),
                    ch1.turbid(),
                    ch2.turbid()
                ),
                (Some(_), _, _, _) | (_, Some(_), _, _) | (_, _, Some(_), _) | (_, _, _, Some(_))
            ))
        });

        FrequencyTable {
            // シフト面と無シフト面でそれぞれ別にする
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
        let mut checked_in_best = vec![vec![vec![false; all_chars().len()]; all_chars().len()]; 26];

        // 構成上、すべてのキーがshift/unshiftを持っている
        for (key_idx, def) in best_keymap.iter().enumerate() {
            let unshift_idx = self.character_map[&def.unshift()];
            let shift_idx = self.character_map[&def.shifted()];

            let freq = &mut self.frequency[key_idx].combinations[unshift_idx][shift_idx];
            if let Some(v) = freq {
                checked_in_best[key_idx][unshift_idx][shift_idx] = true;
                *v += 1.0;
            }
        }

        for (key_idx, def) in worst_keymap.iter().enumerate() {
            let unshift_idx = self.character_map[&def.unshift()];
            let shift_idx = self.character_map[&def.shifted()];

            if checked_in_best[key_idx][unshift_idx][shift_idx] {
                continue;
            }

            let freq = &mut self.frequency[key_idx].combinations[unshift_idx][shift_idx];
            if let Some(v) = freq {
                checked_in_best[key_idx][unshift_idx][shift_idx] = true;
                *v -= 0.5;
            }
        }
    }
}
