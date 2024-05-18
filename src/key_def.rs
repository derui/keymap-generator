use crate::{
    char_def::CharDef, frequency_layer::LayeredCharCombination, key_seq::KeySeq, layout::Point,
};

/// キー自体の基本定義。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyDef {
    unshift: Option<CharDef>,
    shifted: Option<CharDef>,
}

impl KeyDef {
    /// 無シフトに対して[def]を設定した[KeyDef]を返す
    pub fn from_combination(combination: &LayeredCharCombination) -> Self {
        KeyDef {
            unshift: combination.char_of_layer("normal"),
            shifted: combination.char_of_layer("shift"),
        }
    }

    /// unshift/shiftedを交換する
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.unshift, &mut self.shifted);
    }

    /// 無シフト面の文字定義を返す
    pub fn unshift_def(&self) -> Option<CharDef> {
        self.unshift.clone()
    }

    /// シフト面の文字定義を返す
    pub fn shifted_def(&self) -> Option<CharDef> {
        self.shifted.clone()
    }

    /// 無シフト面の文字を返す
    pub fn unshift(&self) -> char {
        self.unshift.map(|v| v.normal()).unwrap_or(' ')
    }

    /// シフト面の文字を返す
    pub fn shifted(&self) -> char {
        self.shifted.map(|v| v.normal()).unwrap_or(' ')
    }

    /// 濁点シフト面の文字があれば返す
    pub fn turbid(&self) -> Option<char> {
        let unshift = self.unshift.and_then(|v| v.turbid());
        let shifted = self.shifted.and_then(|v| v.turbid());
        match (unshift, shifted) {
            // 両方があるケースは存在しない
            (Some(c), None) => Some(c),
            (None, Some(c)) => Some(c),
            _ => None,
        }
    }

    /// 半濁点シフト面の文字があれば返す
    pub fn semiturbid(&self) -> Option<char> {
        let unshift = self.unshift.and_then(|v| v.semiturbid());
        let shifted = self.shifted.and_then(|v| v.semiturbid());
        match (unshift, shifted) {
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
