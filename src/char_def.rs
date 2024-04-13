/// ひらがなの一覧。[Key]で入力可能な全文字を定義する
const CHARS: [char; 84] = [
    'あ', 'い', 'う', 'え', 'お', 'か', 'き', 'く', 'け', 'こ', 'さ', 'し', 'す', 'せ', 'そ', 'た',
    'ち', 'つ', 'て', 'と', 'な', 'に', 'ぬ', 'ね', 'の', 'は', 'ひ', 'ふ', 'へ', 'ほ', 'ま', 'み',
    'む', 'め', 'も', 'や', 'ゆ', 'よ', 'ら', 'り', 'る', 'れ', 'ろ', 'わ', 'を', 'ん', 'が', 'ぎ',
    'ぐ', 'げ', 'ご', 'ざ', 'じ', 'ず', 'ぜ', 'ぞ', 'だ', 'ぢ', 'づ', 'で', 'ど', 'ば', 'び', 'ぶ',
    'べ', 'ぼ', 'ぱ', 'ぴ', 'ぷ', 'ぺ', 'ぽ', 'ぁ', 'ぃ', 'ぅ', 'ぇ', 'ぉ', 'ゃ', 'ゅ', 'ょ', 'っ',
    'ー', '、', '。', '・',
];

/// 文字に対応する濁音を返す
fn get_turbid(c: char) -> Option<char> {
    match c {
        'か' => Some('が'),
        'き' => Some('ぎ'),
        'く' => Some('ぐ'),
        'け' => Some('げ'),
        'こ' => Some('ご'),
        'さ' => Some('ざ'),
        'し' => Some('じ'),
        'す' => Some('ず'),
        'せ' => Some('ぜ'),
        'そ' => Some('ぞ'),
        'た' => Some('だ'),
        'ち' => Some('ぢ'),
        'つ' => Some('づ'),
        'て' => Some('で'),
        'と' => Some('ど'),
        'は' => Some('ば'),
        'ひ' => Some('び'),
        'ふ' => Some('ぶ'),
        'へ' => Some('べ'),
        'ほ' => Some('ぼ'),
        _ => None,
    }
}

/// 文字に対応する半濁音を返す
fn get_semiturbid(c: char) -> Option<char> {
    match c {
        'は' => Some('ぱ'),
        'ひ' => Some('ぴ'),
        'ふ' => Some('ぷ'),
        'へ' => Some('ぺ'),
        'ほ' => Some('ぽ'),
        'ほ' => Some('ぽ'),
        // これらの小書き文字は、頻度が低いのがわかっているので、半濁音シフトとして使う
        'あ' => Some('ぁ'),
        'い' => Some('ぃ'),
        'う' => Some('ぅ'),
        'え' => Some('ぇ'),
        'お' => Some('ぉ'),
        _ => None,
    }
}

/// 各文字における定義
///
/// あくまで現状のものでしか無い
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct CharDef {
    normal: char,
    turbid: Option<char>,
    semiturbid: Option<char>,
}

impl CharDef {
    /// 通常のキーに対応するキーを返す
    pub fn find(normal: char) -> CharDef {
        assert!(CHARS.contains(&normal), "Do not contains {}", normal);

        CharDef {
            normal,
            turbid: get_turbid(normal),
            semiturbid: get_semiturbid(normal),
        }
    }

    /// 単独押下で送出する文字をかえす
    pub fn normal(&self) -> char {
        self.normal
    }

    /// 濁音シフト時に送出する文字をかえす
    pub fn turbid(&self) -> Option<char> {
        self.turbid
    }

    /// 半濁音シフト時に送出する文字をかえす
    pub fn semiturbid(&self) -> Option<char> {
        self.semiturbid
    }

    /// 文字を設定する上で、他の文字と競合するかどうかを返す
    ///
    /// # Arguments
    /// * `other` - 競合するかどうかを調べる文字
    ///
    /// # Returns
    /// 競合する場合は`true`、しない場合は`false`
    pub fn conflicted(&self, other: &Self) -> bool {
        match (self.turbid, other.turbid) {
            (Some(_), Some(_)) => true,
            _ => match (self.semiturbid, other.semiturbid) {
                (Some(_), Some(_)) => true,
                _ => self.normal == other.normal,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_conflict_between_no_turbid() {
        // arrange
        let def1 = CharDef::find('ま');
        let def2 = CharDef::find('み');

        // act
        let ret = def1.conflicted(&def2);

        // assert
        assert_eq!(ret, false);
    }

    #[test]
    fn conflict_between_turbid() {
        // arrange
        let def1 = CharDef::find('か');
        let def2 = CharDef::find('し');

        // act
        let ret = def1.conflicted(&def2);

        // assert
        assert_eq!(ret, true);
    }

    #[test]
    fn conflict_between_semiturbid() {
        // arrange
        let def1 = CharDef::find('は');
        let def2 = CharDef::find('あ');

        // act
        let ret = def1.conflicted(&def2);

        // assert
        assert_eq!(ret, true);
    }
}
