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
        let unshifted = char_def::CharDef::find(unshifted);
        let shifted = shifted.map(|v| char_def::CharDef::find(v));

        match shifted {
            Some(shifted) => {
                if unshifted.conflicted(&shifted) {
                    None
                } else {
                    Some(Key::Normal(KeyDef {
                        unshifted: unshifted.normal(),
                        shifted: Some(shifted.normal()),
                        turbid: unshifted.turbid().or_else(|| shifted.turbid()),
                        semiturbid: unshifted.semiturbid().or_else(|| shifted.semiturbid()),
                    }))
                }
            }
            None => Some(Key::Normal(KeyDef {
                unshifted: unshifted.normal(),
                shifted: None,
                turbid: unshifted.turbid(),
                semiturbid: unshifted.semiturbid(),
            })),
        }
    }

    /// シフトキーに対応する[Key]を返す
    pub fn new_shift(unshifted: char, shifted: Option<char>) -> Key {
        Key::Shifter(KeyDef {
            unshifted,
            shifted,
            turbid: None,
            semiturbid: None,
        })
    }

    /// 濁音シフトキーに対応する[Key]を返す
    pub fn new_turbid(unshifted: char, shifted: Option<char>) -> Key {
        Key::Turbid(KeyDef {
            unshifted,
            shifted: shifted,
            turbid: None,
            semiturbid: None,
        })
    }

    /// 半濁音シフトキーに対応する[Key]を返す
    pub fn new_semiturbid(unshifted: char, shifted: Option<char>) -> Key {
        Key::Semiturbid(KeyDef {
            unshifted,
            shifted: shifted,
            turbid: None,
            semiturbid: None,
        })
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

    fn def_mut(&mut self) -> &mut KeyDef {
        match self {
            Key::Normal(k) => k,
            Key::Shifter(k) => k,
            Key::Turbid(k) => k,
            Key::Semiturbid(k) => k,
            Key::Empty => panic!("Can not get any char from empty"),
        }
    }

    /// [other]とunshiftedで創出する文字を交換する
    ///
    /// # Arguments
    /// * `other` - 交換するキー
    ///
    /// # Returns
    /// 交換後のキーのタプル。最初の要素が自身、次の要素が[other]となる。
    pub fn swap_unshifted(&self, other: &Key) -> (Key, Key) {
        let mut new_self = self.clone();
        let mut new_other = other.clone();

        let tmp = new_self.unshifted();
        new_self.def_mut().unshifted = new_other.unshifted();
        new_other.def_mut().unshifted = tmp;

        (new_self, new_other)
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
        let (new_key1, new_key2) = key1.swap_unshifted(&key2);

        // assert
        assert_eq!(new_key1.unshifted(), 'い');
        assert_eq!(new_key2.unshifted(), 'あ');
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
        let ret = Key::new_normal('あ', Some('か')).unwrap();

        // assert
        assert_eq!(ret.unshifted(), 'あ');
        assert_eq!(ret.shifted(), Some('か'));
        assert_eq!(ret.turbid(), Some('が'));
        assert_eq!(ret.semiturbid(), Some('ぁ'));
    }
}
