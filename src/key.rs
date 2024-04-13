use std::fmt::Display;

use crate::char_def;

/// キー自体の基本定義。
#[derive(Debug, Clone, PartialEq, Eq)]
struct KeyDef {
    unshifted: char,
    shifted: Option<char>,
    turbid: Option<char>,
    semiturbid: Option<char>,
}

impl KeyDef {
    /// 対象のキー定義をflipしたものを返す。
    /// ただし、シフト面がない場合はflipできないため、同じ定義を返す
    fn flip(&self) -> Self {
        if let Some(shifted) = self.shifted {
            KeyDef {
                unshifted: shifted,
                shifted: Some(self.unshifted),
                turbid: self.turbid,
                semiturbid: self.semiturbid,
            }
        } else {
            self.clone()
        }
    }

    /// シフト面と無シフト面の文字から、定義を生成する。競合している場合は作成できない。
    ///
    /// # Arguments
    /// * `unshifted` - 無シフト面の文字
    /// * `shifted` - シフト面の文字
    ///
    /// # Returns
    /// キーの定義。競合している場合はNone
    fn from_chars(unshifted: char, shifted: Option<char>) -> Option<KeyDef> {
        let unshifted = char_def::CharDef::find(unshifted);
        let shifted = shifted.map(|v| char_def::CharDef::find(v));

        match shifted {
            Some(shifted) => {
                if unshifted.conflicted(&shifted) {
                    None
                } else {
                    Some(
                        (KeyDef {
                            unshifted: unshifted.normal(),
                            shifted: Some(shifted.normal()),
                            turbid: unshifted.turbid().or_else(|| shifted.turbid()),
                            semiturbid: unshifted.semiturbid().or_else(|| shifted.semiturbid()),
                        }),
                    )
                }
            }
            None => Some(
                (KeyDef {
                    unshifted: unshifted.normal(),
                    shifted: None,
                    turbid: unshifted.turbid(),
                    semiturbid: unshifted.semiturbid(),
                }),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    /// 通常のキー
    Normal(KeyDef),

    /// シフトキー
    ///
    /// シフトキーは、単独で謳歌された場合はキーを送出する、という形であるのと、シフトキー自体をシフト時に
    /// 押下することができる。この場合は同じ制約として扱う必要がある。
    Shifter(KeyDef),

    /// 濁音シフト
    Turbid(KeyDef),

    /// 半濁音シフト
    Semiturbid(KeyDef),

    Empty,
}

impl Key {
    /// 利用しないキーを返す
    pub fn empty() -> Self {
        Key::Empty
    }

    /// 通常のキーに対応するキーを返す
    pub fn new_normal(unshifted: char, shifted: Option<char>) -> Option<Key> {
        KeyDef::from_chars(unshifted, shifted).map(|v| Key::Normal(v))
    }

    /// シフトキーに対応する[Key]を返す
    pub fn new_shift(unshifted: char, shifted: Option<char>) -> Option<Key> {
        KeyDef::from_chars(unshifted, shifted).map(|v| Key::Shifter(v))
    }

    /// 濁音シフトキーに対応する[Key]を返す
    pub fn new_turbid(unshifted: char, shifted: Option<char>) -> Option<Key> {
        KeyDef::from_chars(unshifted, shifted).map(|v| Key::Turbid(v))
    }

    /// 半濁音シフトキーに対応する[Key]を返す
    pub fn new_semiturbid(unshifted: char, shifted: Option<char>) -> Option<Key> {
        KeyDef::from_chars(unshifted, shifted).map(|v| Key::Semiturbid(v))
    }

    /// 単独押下で送出する文字をかえす
    pub fn unshifted(&self) -> char {
        match self {
            Key::Normal(k) => k.unshifted,
            Key::Shifter(k) => k.unshifted,
            Key::Turbid(k) => k.unshifted,
            Key::Semiturbid(k) => k.unshifted,
            Key::Empty => panic!("Can not get any char from empty"),
        }
    }

    /// シフト時に送出する文字をかえす
    pub fn shifted(&self) -> Option<char> {
        match self {
            Key::Normal(k) => k.shifted,
            Key::Shifter(k) => k.shifted,
            Key::Turbid(k) => k.shifted,
            Key::Semiturbid(k) => k.shifted,
            Key::Empty => panic!("Can not get any char from empty"),
        }
    }

    /// 濁音シフト時に送出する文字をかえす
    pub fn turbid(&self) -> Option<char> {
        match self {
            Key::Normal(k) => k.turbid,
            Key::Shifter(k) => k.turbid,
            Key::Turbid(k) => k.turbid,
            Key::Semiturbid(k) => k.turbid,
            Key::Empty => panic!("Can not get any char from empty"),
        }
    }

    /// 半濁音シフト時に送出する文字をかえす
    pub fn semiturbid(&self) -> Option<char> {
        match self {
            Key::Normal(k) => k.semiturbid,
            Key::Shifter(k) => k.semiturbid,
            Key::Turbid(k) => k.semiturbid,
            Key::Semiturbid(k) => k.semiturbid,
            Key::Empty => panic!("Can not get any char from empty"),
        }
    }

    fn change_unshift(&mut self, unshift: char) -> Option<Self> {
        match self {
            Key::Normal(k) => Key::new_normal(unshift, k.shifted),
            Key::Shifter(k) => Key::new_shift(unshift, k.shifted),
            Key::Turbid(k) => Key::new_turbid(unshift, k.shifted),
            Key::Semiturbid(k) => Key::new_semiturbid(unshift, k.shifted),
            Key::Empty => None,
        }
    }

    fn change_shifted(&mut self, shifted: Option<char>) -> Option<Self> {
        match self {
            Key::Normal(k) => Key::new_normal(k.unshifted, shifted),
            Key::Shifter(k) => Key::new_shift(k.unshifted, shifted),
            Key::Turbid(k) => Key::new_turbid(k.unshifted, shifted),
            Key::Semiturbid(k) => Key::new_semiturbid(k.unshifted, shifted),
            Key::Empty => None,
        }
    }

    /// [other]とunshiftedで創出する文字を交換する
    ///
    /// # Arguments
    /// * `other` - 交換するキー
    ///
    /// # Returns
    /// 交換後のキーのタプル。最初の要素が自身、次の要素が[other]となる。
    pub fn swap_unshifted(&self, other: &Key) -> Option<(Key, Key)> {
        let new_self = self.clone().change_unshift(other.unshifted());
        let new_other = other.clone().change_unshift(self.unshifted());

        match (new_self, new_other) {
            (Some(new_self), Some(new_other)) => Some((new_self, new_other)),
            _ => None,
        }
    }

    /// [other]とunshiftedで創出する文字を交換する
    ///
    /// # Arguments
    /// * `other` - 交換するキー
    ///
    /// # Returns
    /// 交換後のキーのタプル。最初の要素が自身、次の要素が[other]となる。
    pub fn swap_shifted(&self, other: &Key) -> Option<(Key, Key)> {
        let new_self = self.clone().change_shifted(other.shifted());
        let new_other = other.clone().change_shifted(self.shifted());

        match (new_self, new_other) {
            (Some(new_self), Some(new_other)) => Some((new_self, new_other)),
            _ => None,
        }
    }

    /// 無シフト面とシフト面のキーを反転したキーを返す
    ///
    /// # Returns
    /// 反転したキー
    pub fn flip(&self) -> Key {
        match self {
            Key::Normal(k) => Key::Normal(k.flip()),
            Key::Shifter(k) => Key::Shifter(k.flip()),
            Key::Turbid(k) => Key::Turbid(k.flip()),
            Key::Semiturbid(k) => Key::Semiturbid(k.flip()),
            Key::Empty => Key::Empty,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_unshifted() {
        // arrange
        let key1 = Key::new_normal('あ', None).unwrap();
        let key2 = Key::new_normal('い', None).unwrap();

        // act
        let (new_key1, new_key2) = key1.swap_unshifted(&key2).unwrap();

        // assert
        assert_eq!(new_key1.unshifted(), 'い');
        assert_eq!(new_key2.unshifted(), 'あ');
    }

    #[test]
    fn can_not_swap_unshifted() {
        // arrange
        let key1 = Key::new_normal('か', None).unwrap();
        let key2 = Key::new_normal('ま', Some('し')).unwrap();

        // act
        let ret = key1.swap_unshifted(&key2);

        // assert
        assert!(ret.is_none(), "should not be swappable");
    }

    #[test]
    fn can_not_new_normal_if_each_key_conflicted() {
        // arrange

        // act
        let ret = Key::new_normal('あ', Some('い'));

        // assert
        assert!(ret.is_none(), "should not be able to make key");
    }

    #[test]
    fn can_not_merge_having_turbid_and_semiturbid() {
        // arrange

        // act

        // assert
        assert!(
            Key::new_normal('か', Some('い')).is_none(),
            "should not be able to make key"
        );
        assert!(
            Key::new_normal('は', Some('い')).is_none(),
            "should not be able to make key"
        );
        assert!(
            Key::new_normal('い', Some('か')).is_none(),
            "should not be able to make key"
        );
        assert!(
            Key::new_normal('い', Some('は')).is_none(),
            "should not be able to make key"
        );
    }

    #[test]
    fn turbid_and_semi_turbid() {
        // arrange

        // act
        let turbid = Key::new_normal('か', None).unwrap();
        let semiturbid = Key::new_normal('は', None).unwrap();

        // assert
        assert_eq!(turbid.unshifted(), 'か');
        assert_eq!(turbid.shifted(), None);
        assert_eq!(turbid.turbid(), Some('が'));
        assert_eq!(turbid.semiturbid(), None);
        assert_eq!(semiturbid.unshifted(), 'は');
        assert_eq!(semiturbid.shifted(), None);
        assert_eq!(semiturbid.turbid(), Some('ば'));
        assert_eq!(semiturbid.semiturbid(), Some('ぱ'));
    }

    #[test]
    fn merge_shift_and_unshift() {
        // arrange

        // act
        let ret = Key::new_normal('ま', Some('か')).unwrap();

        // assert
        assert_eq!(ret.unshifted(), 'ま');
        assert_eq!(ret.shifted(), Some('か'));
        assert_eq!(ret.turbid(), Some('が'));
        assert_eq!(ret.semiturbid(), None);
    }

    #[test]
    fn flip_key() {
        // arrange

        // act
        let ret = Key::new_normal('ま', Some('か')).unwrap().flip();

        // assert
        assert_eq!(ret.unshifted(), 'か');
        assert_eq!(ret.shifted(), Some('ま'));
        assert_eq!(ret.turbid(), Some('が'));
        assert_eq!(ret.semiturbid(), None);
    }
}
