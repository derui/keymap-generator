use crate::char_def::CharDef;

/// キー自体の基本定義。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyDef {
    unshift: Option<CharDef>,
    shifted: Option<CharDef>,
}

impl KeyDef {
    /// 対象のキー定義をflipしたものを返す。
    pub fn flip(&self) -> Self {
        KeyDef {
            unshift: self.shifted,
            shifted: self.unshift,
        }
    }

    /// 無シフトに対して[def]を設定した[KeyDef]を返す
    pub fn unshift_from(def: &CharDef) -> Self {
        KeyDef {
            unshift: Some(def.normal()),
            shifted: None,
        }
    }

    /// シフトに対して[def]を設定した[KeyDef]を返す
    pub fn shifted_from(def: &CharDef) -> Self {
        KeyDef {
            unshift: None,
            shifted: Some(def.normal()),
        }
    }

    /// 渡された[CharDef]をマージする。競合している場合はNoneを返す。
    ///
    /// シフト面がすでに埋まっていれば無シフト面、無シフト面が埋まっていればシフト面にマージする。
    ///
    /// # Arguments
    /// * `def` - マージする[CharDef]
    ///
    /// # Returns
    /// マージした結果の[KeyDef]。競合している場合はNone
    pub fn merge(&self, def: &CharDef) -> Option<Self> {
        // すでに埋まっている場合はマージできない。マージできる場合は、埋まっていない方にマージする
        match (self.unshift, self.shifted) {
            (Some(_), Some(_)) => None,
            (Some(unshifted), None) => {
                if unshifted.conflicts(def) {
                    None
                } else {
                    Some(KeyDef {
                        unshift: Some(unshifted),
                        shifted: Some(def),
                    })
                }
            }
            (None, Some(shifted)) => {
                if shifted.conflicts(def) {
                    None
                } else {
                    Some(KeyDef {
                        unshift: Some(def),
                        shifted: Some(shifted),
                    })
                }
            }
            _ => unreachable!("unshift and shifted are None. It should not be happened."),
        }
    }

    /// 無シフト面の文字があれば返す
    pub fn unshift(&self) -> Option<char> {
        self.unshift.map(|c| c.normal())
    }

    /// シフト面の文字があれば返す
    pub fn shifted(&self) -> Option<char> {
        self.shifted.map(|c| c.normal())
    }

    /// 濁点シフト面の文字があれば返す
    pub fn turbid(&self) -> Option<char> {
        match (
            self.unshift.and_then(|v| v.turbid()),
            self.shifted.and_then(|v| v.turbid()),
        ) {
            // 両方があるケースは存在しない
            (Some(unshifted), None) => Some(unshifted),
            (None, Some(shifted)) => Some(shifted),
            _ => None,
        }
    }

    /// 半濁点シフト面の文字があれば返す
    pub fn semiturbid(&self) -> Option<char> {
        match (
            self.unshift.and_then(|v| v.semiturbid()),
            self.shifted.and_then(|v| v.semiturbid()),
        ) {
            // 両方があるケースは存在しない
            (Some(unshifted), None) => Some(unshifted),
            (None, Some(shifted)) => Some(shifted),
            _ => None,
        }
    }
}
