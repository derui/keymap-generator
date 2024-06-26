use rand::{rngs::StdRng, Rng};
use serde::{Deserialize, Serialize};

use crate::char_def::{self, CharDef};

/// 使用済みのキープール。サイズは[char_def::definitions]と同一で、trueであれば使用済みである
pub type UsedKeyPool = Vec<bool>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Layer {
    /// 各キーにおける文字の頻度
    frequencies: Vec<f64>,

    /// layerの名前。
    name: String,

    /// [frequencies]から確率を計算するための合計値
    total: f64,
}

impl Layer {
    fn new(name: &str) -> Self {
        let frequencies = vec![1.0; char_def::definitions().len()];

        Layer {
            frequencies,
            name: name.to_string(),
            total: char_def::definitions().len() as f64,
        }
    }

    /// キーの分布に対して突然変異をおこす
    ///
    /// 突然変異は、最大と最小のindexの値を交換する。
    fn mutate(&mut self, rng: &mut StdRng) {
        let current_total = self.total;
        self.frequencies.iter_mut().for_each(|v| {
            *v = (*v / current_total * 10000.0).max(1.0).min(100.0);
        });

        self.total = self.frequencies.iter().sum::<f64>();

        let len = self.frequencies.len();

        loop {
            let first = rng.gen_range(0..len);
            let second = rng.gen_range(0..len);

            if first == second {
                continue;
            }

            self.frequencies.swap(first, second);

            break;
        }
    }

    /// 頻度表を更新する
    fn update(&mut self, index: &usize, frequency: f64) {
        if frequency < 0.0 {
            self.frequencies[*index] *= 1.0 + frequency;
        } else {
            self.frequencies[*index] += frequency;
        }

        self.total = self.frequencies.iter().sum();
    }

    /// 確率に応じて、文字の定義を返す
    ///
    /// 利用可能なキーがない場合はNoneを返す
    fn get_char(&self, rng: &mut StdRng, key_pool: &UsedKeyPool) -> Option<CharDef> {
        if key_pool.iter().all(|v| *v) {
            return None;
        }

        let new_total = self
            .frequencies
            .iter()
            .enumerate()
            .filter_map(|(idx, v)| if key_pool[idx] { None } else { Some(*v) })
            .sum();

        let prob = rng.gen_range(0.0..new_total);
        let mut accum = 0.0;

        for (idx, freq) in self.frequencies.iter().enumerate() {
            if key_pool[idx] {
                continue;
            }

            accum += freq;
            if accum >= prob {
                return Some(char_def::definitions()[idx]);
            }
        }

        None
    }
}

/// 頻度レイヤーを束ねたもの。各キー毎にわりあてられる。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayeredFrequency {
    /// 頻度レイヤーのリスト
    layers: Vec<Layer>,
}

impl LayeredFrequency {
    /// 新規に作成する。
    pub fn new(layers: &[&str]) -> Self {
        LayeredFrequency {
            layers: layers.iter().map(|v| Layer::new(v)).collect(),
        }
    }

    /// 各レイヤー毎に取得した文字を返す
    pub fn get_assignment<F>(
        &self,
        rng: &mut StdRng,
        key_pool: &UsedKeyPool,
        pre_defined: &[(&str, Option<CharDef>)],
        predicates: &[F],
    ) -> LayeredCharCombination
    where
        F: Fn(&LayeredCharCombination) -> bool,
    {
        let mut key_pool_cache: UsedKeyPool;
        let def = char_def::definitions();
        let mut count = 0;

        while {
            count += 1;
            count < 10
        } {
            let mut ret = vec![];
            key_pool_cache = key_pool.clone();

            for layer in &self.layers {
                if let Some((_, c)) = pre_defined.iter().find(|(name, _)| *name == layer.name) {
                    ret.push((layer.name.clone(), *c));
                    continue;
                }

                let char = layer.get_char(rng, &key_pool_cache);

                if let Some(c) = char {
                    key_pool_cache[def.iter().position(|v| v == &c).expect("should be found")] =
                        true;
                }

                ret.push((layer.name.clone(), char));
            }

            let ret = LayeredCharCombination::new(&ret);
            if predicates.iter().all(|f| f(&ret)) {
                return ret;
            }
        }

        LayeredCharCombination::new(&Vec::new())
    }

    /// 指定されたlayerと文字の組み合わせから、頻度表を更新する
    pub fn update(&mut self, keys: &[(&str, Option<CharDef>)], rate: f64) {
        let def = char_def::definitions();

        let keys = keys
            .iter()
            .cloned()
            .filter_map(|(name, char_def)| {
                char_def.map(|c| {
                    (
                        name,
                        def.iter().position(|v| *v == c).expect("should be found"),
                    )
                })
            })
            .collect::<Vec<_>>();

        for (name, idx) in keys.iter() {
            for layer in self.layers.iter_mut() {
                if layer.name != *name {
                    continue;
                }

                layer.update(idx, rate)
            }
        }
    }

    pub fn mutate(&mut self, rng: &mut StdRng) {
        for layer in self.layers.iter_mut() {
            layer.mutate(rng)
        }
    }
}

/// layerごとに文字を割り当てた、キーごとの組み合わせ
#[derive(Debug)]
pub struct LayeredCharCombination(Vec<(String, Option<CharDef>)>);

impl LayeredCharCombination {
    pub fn new(chars: &[(String, Option<CharDef>)]) -> Self {
        LayeredCharCombination(chars.to_vec())
    }

    /// layerに割り当てられた文字を取得する
    ///
    /// # Returns
    /// layerが存在しないか、割り当てられていなかった場合はNone
    #[inline]
    pub fn char_of_layer(&self, layer: &str) -> Option<CharDef> {
        self.0
            .iter()
            .find(|(name, _)| *name == layer)
            .and_then(|(_, c)| *c)
    }
}
