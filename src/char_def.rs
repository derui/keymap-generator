use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum CharDef {
    Normal {
        normal: char,
        turbid: Option<char>,
        semiturbid: Option<char>,
    },
    Turbid,
    SemiTurbid,
}

impl CharDef {
    /// 清音かどうかを返す
    pub fn is_cleartone(&self) -> bool {
        match self {
            CharDef::Normal {
                normal: _,
                turbid,
                semiturbid,
            } => turbid.is_none() && semiturbid.is_none(),
            CharDef::Turbid => true,
            CharDef::SemiTurbid => true,
        }
    }

    /// 句点かどうかを返す
    pub fn is_punctuation_mark(&self) -> bool {
        if let CharDef::Normal {
            normal,
            turbid: _,
            semiturbid: _,
        } = self
        {
            *normal == '、'
        } else {
            false
        }
    }

    /// 読点かどうかを返す
    pub fn is_reading_point(&self) -> bool {
        if let CharDef::Normal {
            normal,
            turbid: _,
            semiturbid: _,
        } = self
        {
            *normal == '。'
        } else {
            false
        }
    }

    /// 対象の文字に対応する定義を返す
    pub fn normal(&self) -> char {
        match self {
            CharDef::Normal {
                normal,
                turbid: _,
                semiturbid: _,
            } => *normal,
            CharDef::Turbid => '゛',
            CharDef::SemiTurbid => '゜',
        }
    }

    /// 対象の文字に対応する定義を返す
    pub fn turbid(&self) -> Option<char> {
        match self {
            CharDef::Normal {
                normal: _,
                turbid,
                semiturbid: _,
            } => *turbid,
            CharDef::Turbid => None,
            CharDef::SemiTurbid => None,
        }
    }

    /// 対象の文字に対応する定義を返す
    pub fn semiturbid(&self) -> Option<char> {
        match self {
            CharDef::Normal {
                normal: _,
                turbid: _,
                semiturbid,
            } => *semiturbid,
            CharDef::Turbid => None,
            CharDef::SemiTurbid => None,
        }
    }

    pub fn chars(&self) -> Vec<char> {
        let mut vec = Vec::with_capacity(3);
        vec.push(self.normal());

        if let Some(c) = self.turbid() {
            vec.push(c);
        }

        if let Some(c) = self.semiturbid() {
            vec.push(c)
        }
        vec
    }
}

impl From<&CharDef> for char {
    fn from(value: &CharDef) -> Self {
        value.normal()
    }
}

/// 文字種の定義一覧を返す
pub fn definitions() -> Vec<CharDef> {
    CHARS.to_vec()
}

/// 指定したひらがなの定義を返す
pub fn find(char: char) -> Option<CharDef> {
    CHARS.iter().find(|v| v.normal() == char).cloned()
}

/// すべての文字を返す
pub fn all_chars() -> Vec<char> {
    CHARS.to_vec().iter().flat_map(|c| c.chars()).collect()
}

/// ひらがなの一覧。評価で利用する
const CHARS: [CharDef; 53] = [
    CharDef::Normal {
        normal: 'あ',
        turbid: None,
        semiturbid: Some('ぁ'),
    },
    CharDef::Normal {
        normal: 'い',
        turbid: None,
        semiturbid: Some('ぃ'),
    },
    CharDef::Normal {
        normal: 'う',
        turbid: None,
        semiturbid: Some('ぅ'),
    },
    CharDef::Normal {
        normal: 'え',
        turbid: None,
        semiturbid: Some('ぇ'),
    },
    CharDef::Normal {
        normal: 'お',
        turbid: None,
        semiturbid: Some('ぉ'),
    },
    CharDef::Normal {
        normal: 'か',
        turbid: Some('が'),
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'き',
        turbid: Some('ぎ'),
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'く',
        turbid: Some('ぐ'),
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'け',
        turbid: Some('げ'),
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'こ',
        turbid: Some('ご'),
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'さ',
        turbid: Some('ざ'),
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'し',
        turbid: Some('じ'),
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'す',
        turbid: Some('ず'),
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'せ',
        turbid: Some('ぜ'),
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'そ',
        turbid: Some('ぞ'),
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'た',
        turbid: Some('だ'),
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'ち',
        turbid: Some('ぢ'),
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'つ',
        turbid: Some('づ'),
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'て',
        turbid: Some('で'),
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'と',
        turbid: Some('ど'),
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'な',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'に',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'ぬ',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'ね',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'の',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'は',
        turbid: Some('ば'),
        semiturbid: Some('ぱ'),
    },
    CharDef::Normal {
        normal: 'ひ',
        turbid: Some('び'),
        semiturbid: Some('ぴ'),
    },
    CharDef::Normal {
        normal: 'ふ',
        turbid: Some('ぶ'),
        semiturbid: Some('ぷ'),
    },
    CharDef::Normal {
        normal: 'へ',
        turbid: Some('べ'),
        semiturbid: Some('ぺ'),
    },
    CharDef::Normal {
        normal: 'ほ',
        turbid: Some('ぼ'),
        semiturbid: Some('ぽ'),
    },
    CharDef::Normal {
        normal: 'ま',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'み',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'む',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'め',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'も',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'や',
        turbid: None,
        semiturbid: Some('ゃ'),
    },
    CharDef::Normal {
        normal: 'ゆ',
        turbid: None,
        semiturbid: Some('ゅ'),
    },
    CharDef::Normal {
        normal: 'よ',
        turbid: None,
        semiturbid: Some('ょ'),
    },
    CharDef::Normal {
        normal: 'ら',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'り',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'る',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'れ',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'ろ',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'わ',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'を',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'ん',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'っ',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: '、',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: '。',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'ー',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Normal {
        normal: 'ヴ',
        turbid: None,
        semiturbid: None,
    },
    CharDef::Turbid,
    CharDef::SemiTurbid,
];

#[cfg(test)]
mod tests {

    #[test]
    fn all_chars_always_same_order() {
        // arrange
        let order1 = super::all_chars();
        let order2 = super::all_chars();

        // act

        // assert
        assert_eq!(order1.len(), order2.len());
        assert!(
            order1.iter().zip(&order2).all(|(v1, v2)| { v1 == v2 }),
            "all eleemnts should be same"
        )
    }
}
