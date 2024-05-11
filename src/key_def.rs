use crate::{char_def::CharDef, frequency_table::CharCombination, key_seq::KeySeq, layout::Point};

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

    // 濁点を持つかを返す
    pub fn as_turbid(&self, point: &Point) -> Option<KeySeq> {
        match (self.unshift, self.shifted) {
            (CharDef::Turbid, _) => Some(KeySeq::from_unshift(self.unshift.normal(), point)),
            (_, CharDef::Turbid) => Some(KeySeq::from_shift(self.shifted.normal(), point)),
            _ => None,
        }
    }

    // 半濁点を持つかを返す
    pub fn as_semiturbid(&self, point: &Point) -> Option<KeySeq> {
        match (self.unshift, self.shifted) {
            (CharDef::SemiTurbid, _) => Some(KeySeq::from_unshift(self.unshift.normal(), point)),
            (_, CharDef::SemiTurbid) => Some(KeySeq::from_shift(self.shifted.normal(), point)),
            _ => None,
        }
    }

    /// unshift/shiftedを交換する
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.unshift, &mut self.shifted);
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
