use std::{collections::HashSet, io::Stderr};

use rand::{rngs::StdRng, Rng};

use crate::char_def::{self, CharDef};

/// 使用済みのキープール。値は [char_def::definitions] のインデックスである
type UsedKeyPool = HashSet<usize>;

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

    /// 頻度表を更新する
    fn update(&mut self, index: &usize, frequency: f64) {
        self.frequencies[*index] += frequency;

        self.total += frequency;
    }

    /// 確率に応じて、文字の定義を返す
    ///
    /// 利用可能なキーがない場合はNoneを返す
    fn get_char(&self, rng: &mut StdRng, key_pool: &UsedKeyPool) -> Option<CharDef> {
        if key_pool.len() >= self.frequencies.len() {
            return None;
        }

        let new_total = self
            .frequencies
            .iter()
            .enumerate()
            .filter_map(|(idx, v)| {
                if key_pool.contains(&idx) {
                    None
                } else {
                    Some(*v)
                }
            })
            .sum();

        let prob = rng.gen_range(0.0..new_total);
        let mut accum = 0.0;

        for (idx, freq) in self.frequencies.iter().enumerate() {
            if key_pool.contains(&idx) {
                continue;
            }

            accum += freq;
            if accum >= prob {
                return Some(char_def::definitions()[idx].clone());
            }
        }

        return None;
    }
}

/// 頻度レイヤーを束ねたもの。各キー毎にわりあてられる。
pub struct LayeredFrequency {
    /// 頻度レイヤーのリスト
    layers: Vec<Layer>,
}

impl LayeredFrequency {
    /// 新規に作成する。
    pub fn new(layers: &[&str]) -> Self {
        LayeredFrequency {
            layers: layers.iter().map(|v| Layer::new(*v)).collect(),
        }
    }

    /// 各レイヤー毎に取得した文字を返す
    pub fn get_assignment(
        &self,
        rng: &mut StdRng,
        key_pool: &UsedKeyPool,
    ) -> Vec<(String, Option<CharDef>)> {
        let mut ret = vec![];
        let mut key_pool = key_pool.clone();
        let def = char_def::definitions();

        for layer in &self.layers {
            let char = layer.get_char(rng, &key_pool);

            if let Some(c) = char {
                key_pool.insert(def.iter().position(|v| v == &c).expect("should be found"));
            }

            ret.push((layer.name.clone(), char));
        }

        ret
    }

    /// 指定されたlayerと文字の組み合わせから、頻度表を更新する
    pub fn update(&mut self, keys: &[(&str, Option<CharDef>)], rate: f64) {
        let def = char_def::definitions();

        let keys = keys
            .iter()
            .cloned()
            .filter_map(|(name, char_def)| {
                if let Some(c) = char_def {
                    Some((
                        name,
                        def.iter().position(|v| *v == c).expect("should be found"),
                    ))
                } else {
                    None
                }
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
}
