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
