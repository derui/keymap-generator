use std::collections::HashSet;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct CharDef {
    unshift: char,
    turbid: Option<char>,
    semiturbid: Option<char>,
}

impl CharDef {
    /// 文字種の定義一覧を返す
    pub fn definitions() -> Vec<CharDef> {
        CHARS.to_vec()
    }

    /// 文字の定義上、同一キーに割り当てることができるかどうかを返す
    pub fn conflicts(&self, other: &Self) -> bool {
        match (self.turbid, other.turbid, self.semiturbid, other.semiturbid) {
            (Some(_), Some(_), _, _) => true,
            (_, _, Some(_), Some(_)) => true,
            _ => self.unshift == other.unshift,
        }
    }

    /// 清音かどうかを返す
    pub fn is_cleartone(&self) -> bool {
        self.turbid.is_none() && self.semiturbid.is_none()
    }

    /// 対象の文字に対応する定義を返す
    pub fn unshift(&self) -> char {
        self.unshift
    }

    /// 対象の文字に対応する定義を返す
    pub fn turbid(&self) -> Option<char> {
        self.turbid
    }

    /// 対象の文字に対応する定義を返す
    pub fn semiturbid(&self) -> Option<char> {
        self.semiturbid
    }
}

/// ひらがなの一覧。評価で利用する
const CHARS: [CharDef; 50] = [
    CharDef {
        unshift: 'あ',
        turbid: None,
        semiturbid: Some('ぁ'),
    },
    CharDef {
        unshift: 'い',
        turbid: None,
        semiturbid: Some('ぃ'),
    },
    CharDef {
        unshift: 'う',
        turbid: None,
        semiturbid: Some('ぅ'),
    },
    CharDef {
        unshift: 'え',
        turbid: None,
        semiturbid: Some('ぇ'),
    },
    CharDef {
        unshift: 'お',
        turbid: None,
        semiturbid: Some('ぉ'),
    },
    CharDef {
        unshift: 'か',
        turbid: Some('が'),
        semiturbid: None,
    },
    CharDef {
        unshift: 'き',
        turbid: Some('ぎ'),
        semiturbid: None,
    },
    CharDef {
        unshift: 'く',
        turbid: Some('ぐ'),
        semiturbid: None,
    },
    CharDef {
        unshift: 'け',
        turbid: Some('げ'),
        semiturbid: None,
    },
    CharDef {
        unshift: 'こ',
        turbid: Some('ご'),
        semiturbid: None,
    },
    CharDef {
        unshift: 'さ',
        turbid: Some('ざ'),
        semiturbid: None,
    },
    CharDef {
        unshift: 'し',
        turbid: Some('じ'),
        semiturbid: None,
    },
    CharDef {
        unshift: 'す',
        turbid: Some('ず'),
        semiturbid: None,
    },
    CharDef {
        unshift: 'せ',
        turbid: Some('ぜ'),
        semiturbid: None,
    },
    CharDef {
        unshift: 'そ',
        turbid: Some('ぞ'),
        semiturbid: None,
    },
    CharDef {
        unshift: 'た',
        turbid: Some('だ'),
        semiturbid: None,
    },
    CharDef {
        unshift: 'ち',
        turbid: Some('ぢ'),
        semiturbid: None,
    },
    CharDef {
        unshift: 'つ',
        turbid: Some('づ'),
        semiturbid: None,
    },
    CharDef {
        unshift: 'て',
        turbid: Some('で'),
        semiturbid: None,
    },
    CharDef {
        unshift: 'と',
        turbid: Some('ど'),
        semiturbid: None,
    },
    CharDef {
        unshift: 'な',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: 'に',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: 'ぬ',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: 'ね',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: 'の',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: 'は',
        turbid: Some('ば'),
        semiturbid: Some('ぱ'),
    },
    CharDef {
        unshift: 'ひ',
        turbid: Some('び'),
        semiturbid: Some('ぴ'),
    },
    CharDef {
        unshift: 'ふ',
        turbid: Some('ぶ'),
        semiturbid: Some('ぷ'),
    },
    CharDef {
        unshift: 'へ',
        turbid: Some('べ'),
        semiturbid: Some('ぺ'),
    },
    CharDef {
        unshift: 'ほ',
        turbid: Some('ぼ'),
        semiturbid: Some('ぽ'),
    },
    CharDef {
        unshift: 'ま',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: 'み',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: 'む',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: 'め',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: 'も',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: 'や',
        turbid: None,
        semiturbid: Some('ゃ'),
    },
    CharDef {
        unshift: 'ゆ',
        turbid: None,
        semiturbid: Some('ゅ'),
    },
    CharDef {
        unshift: 'よ',
        turbid: None,
        semiturbid: Some('ょ'),
    },
    CharDef {
        unshift: 'ら',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: 'り',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: 'る',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: 'れ',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: 'ろ',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: 'わ',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: 'を',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: 'ん',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: 'っ',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: 'っ',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: '、',
        turbid: None,
        semiturbid: None,
    },
    CharDef {
        unshift: '。',
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
        let def1 = CharDef::definitions()
            .into_iter()
            .find(|i| i.unshift() == 'ま')
            .unwrap();
        let def2 = CharDef::definitions()
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
        let def1 = CharDef::definitions()
            .into_iter()
            .find(|i| i.unshift() == 'か')
            .unwrap();
        let def2 = CharDef::definitions()
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
