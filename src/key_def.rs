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
            unshift: Some(*def),
            shifted: None,
        }
    }

    /// シフトに対して[def]を設定した[KeyDef]を返す
    pub fn shifted_from(def: &CharDef) -> Self {
        KeyDef {
            unshift: None,
            shifted: Some(*def),
        }
    }

    /// 文字の定義上、同一キー上でマージ可能であるかどうか返す
    fn conflicts(&self, other: &CharDef) -> bool {
        if matches!((self.unshift, self.shifted), (Some(_), Some(_))) {
            return true;
        }

        match (
            self.turbid(),
            other.turbid(),
            self.semiturbid(),
            other.semiturbid(),
        ) {
            (Some(_), Some(_), _, _) => true,
            (_, _, Some(_), Some(_)) => true,
            _ => self.unshift == Some(*other) || self.shifted == Some(*other),
        }
    }

    /// 無シフト面を、 `def` で置き換えた結果を返す。
    ///
    /// # Arguments
    /// * `def` - 無シフト面を置き換える[KeyDef]
    ///
    /// # Returns
    /// 置き換えた結果干渉する場合はNoneを返す
    pub fn replace_unshift(&self, def: &Self) -> Option<Self> {
        let tmp = KeyDef {
            unshift: None,
            shifted: self.shifted,
        };

        match def.unshift {
            Some(unshift) => tmp.merge(&unshift),
            None => Some(tmp),
        }
    }

    /// シフト面を、 `def` で置き換えた結果を返す。
    ///
    /// # Arguments
    /// * `def` - シフト面を置き換える[KeyDef]
    ///
    /// # Returns
    /// 置き換えた結果干渉する場合はNoneを返す
    pub fn replace_shifted(&self, def: &Self) -> Option<Self> {
        let tmp = KeyDef {
            unshift: self.unshift,
            shifted: None,
        };

        match def.shifted {
            Some(shifted) => tmp.merge(&shifted),
            None => Some(tmp),
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
        if self.conflicts(def) {
            return None;
        }

        match (self.unshift, self.shifted) {
            (Some(_), Some(_)) => None,
            (Some(unshift), None) => Some(KeyDef {
                unshift: Some(unshift),
                shifted: Some(*def),
            }),
            (None, Some(shifted)) => Some(KeyDef {
                unshift: Some(*def),
                shifted: Some(shifted),
            }),
            _ => Some(KeyDef {
                unshift: Some(def.clone()),
                shifted: None,
            }),
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
            (Some(c), None) => Some(c),
            (None, Some(c)) => Some(c),
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
            (Some(c), None) => Some(c),
            (None, Some(c)) => Some(c),
            _ => None,
        }
    }

    /// キーから入力可能なすべての文字を返す
    pub fn chars(&self) -> Vec<char> {
        let mut vec = Vec::with_capacity(4);

        if let Some(c) = self.unshift() {
            vec.push(c);
        }

        if let Some(c) = self.shifted() {
            vec.push(c);
        }

        if let Some(c) = self.turbid() {
            vec.push(c);
        }

        if let Some(c) = self.semiturbid() {
            vec.push(c);
        }

        vec
    }
}

#[cfg(test)]
mod tests {
    use crate::char_def;

    use super::*;

    #[test]
    fn should_be_mergeable_with_turbid_and_cleartone() {
        // arrange
        let key = KeyDef::unshift_from(&char_def::find('ま').unwrap());

        // act
        let ret = key.merge(&char_def::find('か').unwrap()).unwrap();

        // assert
        assert_eq!(ret.unshift(), Some('ま'));
        assert_eq!(ret.shifted(), Some('か'));
        assert_eq!(ret.turbid(), Some('が'));
        assert_eq!(ret.semiturbid(), None);
    }

    #[test]
    fn should_be_mergeable_with_turbid_and_semiturbid() {
        // arrange
        let key = KeyDef::unshift_from(&char_def::find('あ').unwrap());

        // act
        let ret = key.merge(&char_def::find('か').unwrap()).unwrap();

        // assert
        assert_eq!(ret.unshift(), Some('あ'));
        assert_eq!(ret.shifted(), Some('か'));
        assert_eq!(ret.turbid(), Some('が'));
        assert_eq!(ret.semiturbid(), Some('ぁ'));
    }

    #[test]
    fn can_not_merge_turbids() {
        // arrange
        let key = KeyDef::unshift_from(&char_def::find('か').unwrap());

        // act
        let ret = key.merge(&char_def::find('さ').unwrap());

        // assert
        assert_eq!(ret, None);
    }

    #[test]
    fn can_not_merge_same_char_def() {
        // arrange
        let key = KeyDef::unshift_from(&char_def::find('か').unwrap());

        // act
        let ret = key.merge(&char_def::find('か').unwrap());

        // assert
        assert_eq!(ret, None);
    }

    #[test]
    fn can_not_merge_full_defined() {
        // arrange
        let key = KeyDef::unshift_from(&char_def::find('か').unwrap());
        let key = key.merge(&char_def::find('あ').unwrap()).unwrap();

        // act
        let ret = key.merge(&char_def::find('ま').unwrap());

        // assert
        assert_eq!(ret, None);
    }
}
