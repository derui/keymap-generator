use std::{
    collections::{HashMap, HashSet},
    hash::BuildHasherDefault,
};

use rand::{rngs::StdRng, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    char_def::{self, definitions, CharDef},
    keymap::Keymap,
    layout::linear::{
        linear_layout, LINEAR_L_SEMITURBID_INDEX, LINEAR_L_SHIFT_INDEX, LINEAR_L_TURBID_INDEX,
        LINEAR_R_SEMITURBID_INDEX, LINEAR_R_SHIFT_INDEX, LINEAR_R_TURBID_INDEX,
    },
};

/// 存在する文字のシフト面と無シフト面に対する組み合わせにおける頻度を表す
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CombinationFrequency {
    /// 組み合わせの頻度。Noneの場合は、その組み合わせが存在しないということを表す
    ///
    /// 全体としては２次元配列として構成されていて、1次元目が無シフト面、２次元目がシフト面という扱いになっている。
    /// keyの定義上、必ず１キーには必ず無シフト面とシフト面の両方に文字が割り当てられるようになっている。
    combinations: Vec<Vec<Option<f64>>>,

    total: f64,
}
impl CombinationFrequency {
    /// 指定された組み合わせが有効であるとして、対応する文字の組み合わせに対して更新する
    fn update_frequency(&mut self, comb: (usize, usize), learning_rate: f64) {
        let (first_idx, second_idx) = comb;

        // 2次元配列自体は、unshift -> shiftで構成している
        let mut total = 0_f64;
        let _count = self
            .combinations
            .iter()
            .map(|v| v.iter().map(|v| v.map_or(0.0, |_| 1.0)).sum::<f64>())
            .sum::<f64>();

        for (ri, row) in self.combinations.iter_mut().enumerate() {
            for (ci, col) in row.iter_mut().enumerate() {
                let Some(v) = col else { continue };

                if ri == first_idx && ci == second_idx {
                    *v += learning_rate;
                }
                total += *v;
            }
        }

        self.total = total;
    }

    /// キーの分布に対して突然変異をおこす
    ///
    /// 突然変異は、最大と最小のindexの値を交換する。
    fn mutate(&mut self, rng: &mut StdRng) {
        let current_total = self.total;
        self.combinations.iter_mut().for_each(|row| {
            row.iter_mut().for_each(|v| {
                if let Some(v) = v {
                    *v = (*v / current_total * 10000.0).max(1.0).min(100.0);
                }
            })
        });
        self.total = self
            .combinations
            .iter()
            .flatten()
            .map(|v| match v {
                Some(v) => *v,
                None => 0.0,
            })
            .sum::<f64>();

        let combinations = self
            .combinations
            .iter()
            .flatten()
            .cloned()
            .collect::<Vec<_>>();
        let len = combinations.len();

        loop {
            let first = rng.gen_range(0..len);
            let second = rng.gen_range(0..len);

            if first == second {
                continue;
            }

            let Some(first_v) = combinations[first] else {
                continue;
            };
            let Some(second_v) = combinations[second] else {
                continue;
            };

            let first_row = first / self.combinations.len();
            let first_col = first % self.combinations.len();

            let second_row = second / self.combinations.len();
            let second_col = second % self.combinations.len();

            self.combinations[first_row][first_col] = Some(second_v);
            self.combinations[second_row][second_col] = Some(first_v);

            break;
        }
    }

    /// 指定された `ch` を含む組み合わせを無効にする
    fn disable(&mut self, character_map: &HashMap<char, usize>, ch: char) {
        let ch_idx = character_map[&ch];

        // 2次元配列自体は、unshift -> shiftで構成している
        for v in self.combinations[ch_idx].iter_mut() {
            self.total -= v.unwrap_or(0.0);
            *v = None;
        }

        for row in self.combinations.iter_mut() {
            self.total -= row[ch_idx].unwrap_or(0.0);
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
        let mut total = 0_f64;

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
                    total += 1.0;
                }
            }
        }

        CombinationFrequency {
            combinations: vec,
            total,
        }
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
    combinations: Vec<CombinationFrequency>,
    // 文字と頻度表におけるindexのマッピング
    character_index_map: Vec<CharDef>,
    // 文字からindexに変換するmap
    character_map: HashMap<char, usize>,
}

impl KeyAssigner {
    /// `freq_table` から[KeyAssigner]を生成する
    pub fn from_freq(freq_table: &FrequencyTable) -> Self {
        Self {
            combinations: freq_table.frequency.to_vec(),
            character_index_map: char_def::definitions().into_iter().collect(),
            character_map: freq_table.character_map.clone(),
        }
    }

    /// 有効なキーが少ない順に並べられたkeyのindexを返す
    pub fn ordered_key_indices(&self) -> Vec<usize> {
        let mut vec = self
            .combinations
            .iter()
            .enumerate()
            .map(|(idx, v)| {
                (
                    idx,
                    v.combinations
                        .iter()
                        .flatten()
                        .filter(|v| v.is_none())
                        .count(),
                )
            })
            .collect::<Vec<_>>();

        vec.sort_unstable_by(|(_, comb1), (_, comb2)| comb2.cmp(comb1));
        vec.into_iter().map(|(v, _)| v).collect()
    }

    /// 指定された `key_idx` において、選択確率に応じた [CharCombination] を返す
    ///
    /// 選択されたキーに含まれている文字は、無シフト面の両方から使用できなくなる
    pub fn pick_key(&mut self, rng: &mut StdRng, key_idx: usize) -> Option<CharCombination> {
        let freq = &self.combinations[key_idx];
        let mut rows = (0..freq.combinations.len()).collect::<Vec<_>>();
        rows.sort_by_key(|_| rng.gen::<i32>());

        let prob = rng.gen::<f64>();
        let mut accum = 0.0;
        for first_idx in rows.iter() {
            let first = &freq.combinations[*first_idx];

            let mut cols = (0..freq.combinations.len()).collect::<Vec<_>>();
            cols.sort_by_key(|_| rng.gen::<i32>());

            for second_idx in cols.iter() {
                let Some(second) = first[*second_idx] else {
                    continue;
                };

                if accum + (second / freq.total) >= prob {
                    for freq in self.combinations.iter_mut() {
                        freq.disable(
                            &self.character_map,
                            self.character_index_map[*first_idx].normal(),
                        );
                        freq.disable(
                            &self.character_map,
                            self.character_index_map[*second_idx].normal(),
                        );
                    }

                    return Some(CharCombination(
                        self.character_index_map[*first_idx],
                        self.character_index_map[*second_idx],
                    ));
                }
                accum += second / freq.total;
            }
        }

        None
    }

    /// 左シフトキーに対する組み合わせを返す。
    ///
    /// この関数は、right_shift_keyとセットで利用することを前提としている。
    pub fn left_shift_key(&self, rng: &mut StdRng) -> CharCombination {
        let freq = &self.combinations[LINEAR_L_SHIFT_INDEX];

        let prob = rng.gen::<f64>();
        let mut accum = 0.0;
        for (first_idx, first) in freq.combinations.iter().enumerate() {
            for (second_idx, second) in first.iter().enumerate() {
                let Some(second) = second else { continue };

                if accum + (*second / freq.total) >= prob {
                    return CharCombination(
                        self.character_index_map[first_idx],
                        self.character_index_map[second_idx],
                    );
                }
                accum += *second / freq.total;
            }
        }

        unreachable!("do not come here");
    }

    /// 右シフトキーに対する組み合わせを返す。
    ///
    /// この関数は、left_shift_keyとセットで利用することを前提としている。
    pub fn right_shift_key(
        &mut self,
        rng: &mut StdRng,
        left_combination: &CharCombination,
    ) -> CharCombination {
        // シフトキーは、シフト面が同一であることが要件になる。
        let shift_idx = self.character_map[&left_combination.shifted().normal()];

        // まず無シフト面で同じ文字を選択できないようにする
        for freq in self.combinations.iter_mut() {
            freq.disable(&self.character_map, left_combination.unshift().normal());
        }

        let freq = &self.combinations[LINEAR_R_SHIFT_INDEX];

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

            if accum + (second / total) < prob {
                accum += second / total;
                continue;
            }

            // 選択できたら、対象の場所を全体からdisableする
            for freq in self.combinations.iter_mut() {
                freq.disable(
                    &self.character_map,
                    self.character_index_map[first_idx].normal(),
                );
                freq.disable(&self.character_map, left_combination.shifted().normal());
            }

            return CharCombination::new(
                &self.character_index_map[first_idx],
                &self.character_index_map[shift_idx],
            );
        }

        unreachable!("do not come here");
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn all_char_combinations_always_same_order() {
        // arrange
        let order1 = super::CombinationFrequency::new(|_, _| true);
        let order2 = super::CombinationFrequency::new(|_, _| true);

        // act

        // assert
        assert!(order1 == order2, "all eleemnts should be same")
    }
}

/// 条件に一致する文字の組み合わせを返す

/// キーの出現回数を記録するテーブル
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrequencyTable {
    // 各キーごとに、どの文字がどれだけ出現したかを記録する
    //
    // キーの数として、シフトへの割当も１キーとしてカウントしている。
    frequency: Vec<CombinationFrequency>,

    // 文字と頻度表におけるindexのマッピング
    character_map: HashMap<char, usize>,
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

        // 濁音シフトには、半濁音または濁音のキーは配置できない
        [LINEAR_L_TURBID_INDEX, LINEAR_R_TURBID_INDEX]
            .iter()
            .for_each(|idx| {
                combinations[*idx] =
                    CombinationFrequency::new(|ch1, ch2| ch1.is_cleartone() && ch2.is_cleartone());
            });

        // 半濁音キーのシフト面は句読点を配置する
        // これは、半濁音と濁音に上記の制約を付加すると、単純にキーが足らないので、ここだけは固定ではないこととするため。
        combinations[LINEAR_L_SEMITURBID_INDEX] =
            CombinationFrequency::new(|ch1, ch2| ch1.is_cleartone() && ch2.is_punctuation_mark());
        combinations[LINEAR_R_SEMITURBID_INDEX] =
            CombinationFrequency::new(|ch1, ch2| ch1.is_cleartone() && ch2.is_reading_point());

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
            let unshift_idx = self.character_map[&def.unshift()];
            let shift_idx = self.character_map[&def.shifted()];

            self.frequency[key_idx].update_frequency((unshift_idx, shift_idx), learning_rate)
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
