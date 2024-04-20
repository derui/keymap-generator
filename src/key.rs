use crate::char_def;

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
        KeyDef::from_chars(unshifted, shifted).map(Key::Normal)
    }

    /// シフトキーに対応する[Key]を返す
    pub fn new_shift(unshifted: char, shifted: Option<char>) -> Option<Key> {
        KeyDef::from_chars(unshifted, shifted).map(Key::Shifter)
    }

    /// 濁音シフトキーに対応する[Key]を返す
    pub fn new_turbid(unshifted: char, shifted: Option<char>) -> Option<Key> {
        KeyDef::from_chars(unshifted, shifted).map(Key::Turbid)
    }

    /// 半濁音シフトキーに対応する[Key]を返す
    pub fn new_semiturbid(unshifted: char, shifted: Option<char>) -> Option<Key> {
        KeyDef::from_chars(unshifted, shifted).map(Key::Semiturbid)
    }

    /// 単独押下で送出する文字をかえす
    pub fn unshifted(&self) -> char {
        match self {
            Key::Normal(k) => k.unshifted,
            Key::Shifter(k) => k.unshifted,
            Key::Turbid(k) => k.unshifted,
            Key::Semiturbid(k) => k.unshifted,
            Key::Empty => '　',
        }
    }

    /// シフト時に送出する文字をかえす
    pub fn shifted(&self) -> Option<char> {
        match self {
            Key::Normal(k) => k.shifted,
            Key::Shifter(k) => k.shifted,
            Key::Turbid(k) => k.shifted,
            Key::Semiturbid(k) => k.shifted,
            Key::Empty => None,
        }
    }

    /// 濁音シフト時に送出する文字をかえす
    pub fn turbid(&self) -> Option<char> {
        match self {
            Key::Normal(k) => k.turbid,
            Key::Shifter(k) => k.turbid,
            Key::Turbid(k) => k.turbid,
            Key::Semiturbid(k) => k.turbid,
            Key::Empty => None,
        }
    }

    /// 半濁音シフト時に送出する文字をかえす
    pub fn semiturbid(&self) -> Option<char> {
        match self {
            Key::Normal(k) => k.semiturbid,
            Key::Shifter(k) => k.semiturbid,
            Key::Turbid(k) => k.semiturbid,
            Key::Semiturbid(k) => k.semiturbid,
            Key::Empty => None,
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

    /// 指定した文字がこのキーに含まれるかどうかを返す
    pub fn contains(&self, char: char) -> bool {
        self.unshifted() == char
            || self.shifted() == Some(char)
            || self.turbid() == Some(char)
            || self.semiturbid() == Some(char)
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
            Key::new_normal('か', Some('い')).is_some(),
            "should not be able to make key"
        );
        assert!(
            Key::new_normal('は', Some('い')).is_none(),
            "should not be able to make key"
        );
        assert!(
            Key::new_normal('い', Some('か')).is_some(),
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
