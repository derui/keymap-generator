use crate::{char_def::CharDef, frequency_table::CharCombination};

/// キー自体の基本定義。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyDef {
    unshift: CharDef,
    shifted: CharDef,
}

impl KeyDef {
    /// 無シフトに対して[def]を設定した[KeyDef]を返す
    pub fn from_combination(combination: &CharCombination) -> Self {
        KeyDef {
            unshift: combination.unshift(),
            shifted: combination.shifted(),
        }
    }

    /// 無シフト面の文字を返す
    pub fn unshift(&self) -> char {
        self.unshift.normal()
    }

    /// シフト面の文字を返す
    pub fn shifted(&self) -> char {
        self.shifted.normal()
    }

    /// 濁点シフト面の文字があれば返す
    pub fn turbid(&self) -> Option<char> {
        match (self.unshift.turbid(), self.shifted.turbid()) {
            // 両方があるケースは存在しない
            (Some(c), None) => Some(c),
            (None, Some(c)) => Some(c),
            _ => None,
        }
    }

    /// 半濁点シフト面の文字があれば返す
    pub fn semiturbid(&self) -> Option<char> {
        match (self.unshift.semiturbid(), self.shifted.semiturbid()) {
            // 両方があるケースは存在しない
            (Some(c), None) => Some(c),
            (None, Some(c)) => Some(c),
            _ => None,
        }
    }

    /// キーから入力可能なすべての文字を返す
    pub fn chars(&self) -> Vec<char> {
        let mut vec = Vec::with_capacity(4);
        vec.push(self.unshift());
        vec.push(self.shifted());

        if let Some(c) = self.turbid() {
            vec.push(c);
        }

        if let Some(c) = self.semiturbid() {
            vec.push(c);
        }

        vec
    }
}
