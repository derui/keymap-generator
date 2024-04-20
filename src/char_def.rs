#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct CharDef {
    normal: char,
    turbid: Option<char>,
    semiturbid: Option<char>,
}

impl CharDef {
    /// 文字の定義上、同一キーに割り当てることができるかどうかを返す
    pub fn conflicts(&self, other: &Self) -> bool {
        match (self.turbid, other.turbid, self.semiturbid, other.semiturbid) {
            (Some(_), Some(_), _, _) => true,
            (_, _, Some(_), Some(_)) => true,
            _ => self.normal == other.normal,
        }
    }

    /// 清音かどうかを返す
    pub fn is_cleartone(&self) -> bool {
        self.turbid.is_none() && self.semiturbid.is_none()
    }

    /// 対象の文字に対応する定義を返す
    pub fn unshift(&self) -> char {
        self.normal
    }

    /// 対象の文字に対応する定義を返す
    pub fn turbid(&self) -> Option<char> {
        self.turbid
    }

    /// 対象の文字に対応する定義を返す
    pub fn semiturbid(&self) -> Option<char> {
        self.semiturbid
    }

    pub fn chars(&self) -> Vec<char> {
        let mut vec = Vec::with_capacity(3);
        vec.push(self.normal);

        if let Some(c) = self.turbid {
            vec.push(c);
        }

        if let Some(c) = self.semiturbid {
            vec.push(c)
        }
        vec
    }
}

/// 文字種の定義一覧を返す
pub fn definitions() -> Vec<CharDef> {
    CHARS.to_vec()
}

/// 指定したひらがなの定義を返す
pub fn find(char: char) -> Option<CharDef> {
    CHARS.iter().find(|v| v.normal == char).cloned()
}

/// すべての文字を返す
pub fn all_chars() -> Vec<char> {
    CHARS.to_vec().iter().map(|c| c.chars()).flatten().collect()
}

/// ひらがなの一覧。評価で利用する
const CHARS: [CharDef; 50] = [
    CharDef {
        normal: 'あ',
        turbid: None,
        semiturbid: Some('ぁ'),
    },
    CharDef {
        normal: 'い',
        turbid: None,
        semiturbid: Some('ぃ'),
    },
    CharDef {
        normal: 'う',
        turbid: None,
        semiturbid: Some('ぅ'),
    },
    CharDef {
        normal: 'え',
        turbid: None,
        semiturbid: Some('ぇ'),
    },
    CharDef {
        normal: 'お',
        turbid: None,
        semiturbid: Some('ぉ'),
    },
    CharDef {
        normal: 'か',
        turbid: Some('が'),
        semiturbid: None,
    },
    CharDef {
        normal: 'き',
        turbid: Some('ぎ'),
        semiturbid: None,
    },
    CharDef {
        normal: 'く',
        turbid: Some('ぐ'),
        semiturbid: None,
    },
    CharDef {
        normal: 'け',
        turbid: Some('げ'),
        semiturbid: None,
    },
    CharDef {
        normal: 'こ',
        turbid: Some('ご'),
        semiturbid: None,
    },
    CharDef {
        normal: 'さ',
        turbid: Some('ざ'),
        semiturbid: None,
    },
    CharDef {
        normal: 'し',
        turbid: Some('じ'),
        semiturbid: None,
    },
    CharDef {
        normal: 'す',
        turbid: Some('ず'),
        semiturbid: None,
    },
    CharDef {
        normal: 'せ',
        turbid: Some('ぜ'),
        semiturbid: None,
    },
    CharDef {
        normal: 'そ',
        turbid: Some('ぞ'),
        semiturbid: None,
    },
    CharDef {
        normal: 'た',
        turbid: Some('だ'),
        semiturbid: None,
    },
    CharDef {
        normal: 'ち',
        turbid: Some('ぢ'),
        semiturbid: None,
    },
    CharDef {
        normal: 'つ',
        turbid: Some('づ'),
        semiturbid: None,
    },
    CharDef {
        normal: 'て',
        turbid: Some('で'),
        semiturbid: None,
    },
    CharDef {
        normal: 'と',
        turbid: Some('ど'),
        semiturbid: None,
    },
    CharDef {
        normal: 'な',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: 'に',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: 'ぬ',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: 'ね',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: 'の',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: 'は',
        turbid: Some('ば'),
        semiturbid: Some('ぱ'),
    },
    CharDef {
        normal: 'ひ',
        turbid: Some('び'),
        semiturbid: Some('ぴ'),
    },
    CharDef {
        normal: 'ふ',
        turbid: Some('ぶ'),
        semiturbid: Some('ぷ'),
    },
    CharDef {
        normal: 'へ',
        turbid: Some('べ'),
        semiturbid: Some('ぺ'),
    },
    CharDef {
        normal: 'ほ',
        turbid: Some('ぼ'),
        semiturbid: Some('ぽ'),
    },
    CharDef {
        normal: 'ま',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: 'み',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: 'む',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: 'め',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: 'も',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: 'や',
        turbid: None,
        semiturbid: Some('ゃ'),
    },
    CharDef {
        normal: 'ゆ',
        turbid: None,
        semiturbid: Some('ゅ'),
    },
    CharDef {
        normal: 'よ',
        turbid: None,
        semiturbid: Some('ょ'),
    },
    CharDef {
        normal: 'ら',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: 'り',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: 'る',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: 'れ',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: 'ろ',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: 'わ',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: 'を',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: 'ん',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: 'っ',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: 'っ',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: '、',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        normal: '。',
        turbid: None,
        semiturbid: None,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_conflict_between_no_turbid() {
        // arrange
        let def1 = super::definitions()
            .into_iter()
            .find(|i| i.unshift() == 'ま')
            .unwrap();
        let def2 = super::definitions()
            .into_iter()
            .find(|i| i.unshift() == 'み')
            .unwrap();

        // act
        let ret = def1.conflicts(&def2);

        // assert
        assert!(!ret);
    }

    #[test]
    fn conflict_between_turbid() {
        // arrange
        let def1 = super::definitions()
            .into_iter()
            .find(|i| i.unshift() == 'か')
            .unwrap();
        let def2 = super::definitions()
            .into_iter()
            .find(|i| i.unshift() == 'し')
            .unwrap();

        // act
        let ret = def1.conflicts(&def2);

        // assert
        assert!(ret);
    }

    #[test]
    fn conflict_between_semiturbid() {
        // arrange
        let def1 = CharDef::definitions()
            .into_iter()
            .find(|i| i.unshift() == 'は')
            .unwrap();
        let def2 = CharDef::definitions()
            .into_iter()
            .find(|i| i.unshift() == 'あ')
            .unwrap();

        // act
        let ret = def1.conflicts(&def2);

        // assert
        assert!(ret);
    }
}
