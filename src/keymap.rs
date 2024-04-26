use std::fmt::Display;

use rand::rngs::StdRng;

use crate::{
    char_def::{self},
    frequency_table::KeyAssigner,
    key_def::KeyDef,
    layout::{
        linear::{
            self, LINEAR_L_SEMITURBID_INDEX, LINEAR_L_SHIFT_INDEX, LINEAR_L_TURBID_INDEX,
            LINEAR_R_SEMITURBID_INDEX, LINEAR_R_SHIFT_INDEX, LINEAR_R_TURBID_INDEX,
        },
        Point,
    },
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum KeyKind {
    Normal,
    Shift,
    Turbid,
    Semiturbid,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum KeyAssignment {
    /// 割当済。変更することも出来る。
    A(KeyDef),

    /// 未割当
    U,
}

impl KeyAssignment {
    /// 対象の文字が入力可能であるかを返す
    fn contains(&self, c: char) -> bool {
        match self {
            KeyAssignment::A(k) => k.chars().contains(&c),
            KeyAssignment::U => false,
        }
    }

    /// assignされていればswapする
    fn swap(&mut self) {
        match self {
            KeyAssignment::A(k) => k.swap(),
            KeyAssignment::U => (),
        }
    }
}

/// 有効なキーマップ
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Keymap {
    layout: Vec<KeyAssignment>,
}

mod constraints {
    use std::collections::HashSet;

    use crate::{
        char_def::{self, definitions},
        layout::linear::{
            LINEAR_L_SEMITURBID_INDEX, LINEAR_L_SHIFT_INDEX, LINEAR_L_TURBID_INDEX,
            LINEAR_R_SEMITURBID_INDEX, LINEAR_R_SHIFT_INDEX, LINEAR_R_TURBID_INDEX,
        },
    };

    use super::KeyAssignment;

    /// 左右のシフトキーに割り当てられている文字が同一であるか確認する
    pub(super) fn should_shift_having_same_key(layout: &[KeyAssignment]) -> bool {
        let left_shifted = &layout[LINEAR_L_SHIFT_INDEX];
        let right_shifted = &layout[LINEAR_R_SHIFT_INDEX];

        match (left_shifted, right_shifted) {
            (KeyAssignment::A(l), KeyAssignment::A(r)) => l.shifted() == r.shifted(),
            _ => false,
        }
    }

    /// 左右のシフトキーには清音しか設定されていないかどうかを確認する
    pub(super) fn should_shift_only_clear_tones(layout: &[KeyAssignment]) -> bool {
        let left_shifted = &layout[LINEAR_L_SHIFT_INDEX];
        let right_shifted = &layout[LINEAR_R_SHIFT_INDEX];

        let cleartones = definitions()
            .into_iter()
            .filter(|v| v.is_cleartone())
            .map(|v| v.normal())
            .collect::<Vec<char>>();
        match (left_shifted, right_shifted) {
            (KeyAssignment::A(l), KeyAssignment::A(r)) => {
                let l = l.shifted();
                let r = r.shifted();
                cleartones.contains(&l) && cleartones.contains(&r)
            }
            _ => false,
        }
    }

    /// 左右の濁音シフト間では、いずれかのキーにしか濁音が設定されていないかどうかを確認する
    pub(super) fn should_only_one_turbid(layout: &[KeyAssignment]) -> bool {
        let left_turbid = &layout[LINEAR_L_TURBID_INDEX];
        let right_turbid = &layout[LINEAR_R_TURBID_INDEX];

        match (left_turbid, right_turbid) {
            (KeyAssignment::A(l), KeyAssignment::A(r)) => {
                let l = l.turbid();
                let r = r.turbid();

                matches!((l, r), (Some(_), None) | (None, Some(_)))
            }
            _ => true,
        }
    }

    /// 左右の濁音シフト間では、いずれかのキーにしか濁音が設定されていないかどうかを確認する
    pub(super) fn should_only_one_semiturbid(layout: &[KeyAssignment]) -> bool {
        let l = &layout[LINEAR_L_SEMITURBID_INDEX];
        let r = &layout[LINEAR_R_SEMITURBID_INDEX];

        match (l, r) {
            (KeyAssignment::A(l), KeyAssignment::A(r)) => {
                let l = l.semiturbid();
                let r = r.semiturbid();

                matches!((l, r), (Some(_), None) | (None, Some(_)))
            }
            _ => true,
        }
    }

    /// 左右の濁音・半濁音の間では、いずれかのキーにしか濁音と半濁音が設定されていないかどうかを確認する
    ///
    /// このキーが同時に押下されたとき、矛盾なく入力できるのは、
    /// * 濁音キーに半濁音が設定されていて、半濁音キーには濁音が設定されていない
    /// * 半濁音キーに濁音が設定されていて、濁音キーには半濁音が設定されていない
    /// のいずれかである
    pub(super) fn should_be_explicit_between_left_turbid_and_right_semiturbit(
        layout: &[KeyAssignment],
    ) -> bool {
        let left_turbid = &layout[LINEAR_L_TURBID_INDEX];
        let right_semiturbid = &layout[LINEAR_R_SEMITURBID_INDEX];

        match (left_turbid, right_semiturbid) {
            (KeyAssignment::A(l), KeyAssignment::A(r)) => {
                matches!(
                    (r.turbid(), l.semiturbid(),),
                    (None, None) | (Some(_), None) | (None, Some(_))
                )
            }
            _ => true,
        }
    }

    /// 左右の濁音・半濁音の間では、いずれかのキーにしか濁音と半濁音が設定されていないかどうかを確認する
    pub(super) fn should_be_explicit_between_right_turbid_and_left_semiturbit(
        layout: &[KeyAssignment],
    ) -> bool {
        // 濁音と半濁音を同時に押下したとき、両方に値が入っていると競合してしまうので、それを防ぐ
        let right_turbid = &layout[LINEAR_R_TURBID_INDEX];
        let left_semiturbid = &layout[LINEAR_L_SEMITURBID_INDEX];

        match (right_turbid, left_semiturbid) {
            (KeyAssignment::A(r), KeyAssignment::A(l)) => {
                matches!(
                    (l.turbid(), r.semiturbid(),),
                    (None, None) | (Some(_), None) | (None, Some(_))
                )
            }
            _ => true,
        }
    }

    /// すべての文字が入力できる状態であることを確認する
    pub(super) fn should_be_able_to_all_input(layout: &[KeyAssignment]) -> bool {
        let mut chars: HashSet<char> = char_def::all_chars().into_iter().collect();

        for assignment in layout.iter() {
            match assignment {
                KeyAssignment::A(k) => {
                    k.chars().iter().for_each(|c| {
                        chars.remove(c);
                    });
                }
                KeyAssignment::U => continue,
            }
        }

        chars.is_empty()
    }

    #[cfg(test)]
    mod tests {
        use crate::{frequency_table::CharCombination, key_def::KeyDef};

        use super::*;

        fn empty_layout() -> Vec<KeyAssignment> {
            vec![KeyAssignment::U; 26]
        }

        fn put_key(layout: &mut [KeyAssignment], key: KeyDef, pos: usize) {
            layout[pos] = KeyAssignment::A(key);
        }

        #[test]
        fn having_same_key_between_shift() {
            // arrange
            let mut layout = empty_layout();
            let c1 = char_def::find('を').unwrap();
            let c2 = char_def::find('る').unwrap();
            let c3 = char_def::find('ら').unwrap();
            let comb1 = CharCombination::new(&c2, &c1);
            let comb2 = CharCombination::new(&c3, &c1);
            put_key(
                &mut layout,
                KeyDef::from_combination(&comb1),
                LINEAR_L_SHIFT_INDEX,
            );
            put_key(
                &mut layout,
                KeyDef::from_combination(&comb2),
                LINEAR_R_SHIFT_INDEX,
            );

            // act
            let ret = should_shift_having_same_key(&layout);

            // assert
            assert!(ret, "should be valid")
        }

        #[test]
        fn having_clear_tone_only_in_shifts() {
            // arrange
            let mut layout = empty_layout();
            let c1 = char_def::find('を').unwrap();
            let c2 = char_def::find('る').unwrap();
            let c3 = char_def::find('ら').unwrap();
            let comb1 = CharCombination::new(&c2, &c1);
            let comb2 = CharCombination::new(&c3, &c1);
            put_key(
                &mut layout,
                KeyDef::from_combination(&comb1),
                LINEAR_L_SHIFT_INDEX,
            );
            put_key(
                &mut layout,
                KeyDef::from_combination(&comb2),
                LINEAR_R_SHIFT_INDEX,
            );

            // act
            let ret = should_shift_only_clear_tones(&layout);

            // assert
            assert!(ret, "should be valid")
        }

        #[test]
        fn not_having_same_key_between_shift() {
            // arrange
            let mut layout = empty_layout();
            let c1 = char_def::find('を').unwrap();
            let c2 = char_def::find('に').unwrap();
            let c3 = char_def::find('る').unwrap();
            let c4 = char_def::find('ら').unwrap();
            let comb1 = CharCombination::new(&c1, &c2);
            let comb2 = CharCombination::new(&c3, &c4);
            put_key(
                &mut layout,
                KeyDef::from_combination(&comb1),
                LINEAR_L_SHIFT_INDEX,
            );
            put_key(
                &mut layout,
                KeyDef::from_combination(&comb2),
                LINEAR_R_SHIFT_INDEX,
            );

            // act
            let ret = should_shift_having_same_key(&layout);

            // assert
            assert!(!ret, "should be valid")
        }

        #[test]
        fn only_one_turbid_between_turbid_keys() {
            // arrange
            let mut layout = empty_layout();
            let c1 = char_def::find('あ').unwrap();
            let c2 = char_def::find('か').unwrap();
            let c3 = char_def::find('い').unwrap();
            let c4 = char_def::find('ら').unwrap();
            put_key(
                &mut layout,
                KeyDef::from_combination(&CharCombination::new(&c1, &c2)),
                LINEAR_L_TURBID_INDEX,
            );
            put_key(
                &mut layout,
                KeyDef::from_combination(&CharCombination::new(&c3, &c4)),
                LINEAR_R_TURBID_INDEX,
            );

            // act
            let ret = should_only_one_turbid(&layout);

            // assert
            assert!(ret, "should be valid")
        }

        #[test]
        fn two_turbid_between_turbid_keys() {
            // arrange
            let mut layout = empty_layout();
            let c1 = char_def::find('あ').unwrap();
            let c2 = char_def::find('か').unwrap();
            let c3 = char_def::find('い').unwrap();
            let c4 = char_def::find('し').unwrap();
            put_key(
                &mut layout,
                KeyDef::from_combination(&CharCombination::new(&c1, &c2)),
                LINEAR_L_TURBID_INDEX,
            );
            put_key(
                &mut layout,
                KeyDef::from_combination(&CharCombination::new(&c3, &c4)),
                LINEAR_R_TURBID_INDEX,
            );

            // act
            let ret = should_only_one_turbid(&layout);

            // assert
            assert!(!ret, "should be valid")
        }

        #[test]
        fn only_one_turbid_and_semiturbid_set_between_left_turbid_and_right_semiturbid() {
            // arrange
            let mut layout = empty_layout();
            let c1 = char_def::find('あ').unwrap();
            let c2 = char_def::find('か').unwrap();
            let c3 = char_def::find('い').unwrap();
            let c4 = char_def::find('ら').unwrap();
            put_key(
                &mut layout,
                KeyDef::from_combination(&CharCombination::new(&c1, &c2)),
                LINEAR_L_TURBID_INDEX,
            );
            put_key(
                &mut layout,
                KeyDef::from_combination(&CharCombination::new(&c3, &c4)),
                LINEAR_R_SEMITURBID_INDEX,
            );

            // act
            let ret = should_be_explicit_between_left_turbid_and_right_semiturbit(&layout);

            // assert
            assert!(ret, "should be valid")
        }

        #[test]
        fn only_one_turbid_and_semiturbid_set_between_right_turbid_and_left_semiturbid() {
            // arrange
            let mut layout = empty_layout();

            let c1 = char_def::find('あ').unwrap();
            let c2 = char_def::find('か').unwrap();
            let c3 = char_def::find('い').unwrap();
            let c4 = char_def::find('ら').unwrap();
            put_key(
                &mut layout,
                KeyDef::from_combination(&CharCombination::new(&c1, &c2)),
                LINEAR_R_TURBID_INDEX,
            );
            put_key(
                &mut layout,
                KeyDef::from_combination(&CharCombination::new(&c3, &c4)),
                LINEAR_L_SEMITURBID_INDEX,
            );

            // act
            let ret = should_be_explicit_between_right_turbid_and_left_semiturbit(&layout);

            // assert
            assert!(ret, "should be valid")
        }
    }
}

impl Keymap {
    /// 指定されたseedを元にしてキーマップを生成する
    ///
    /// 生成されたkeymapは、あくまでランダムなキーマップであり、実際に利用するためには、[Keymap::meet_requirements]がtrueを返すことを前提としなければ
    /// ならない。
    pub fn generate(rng: &mut StdRng, assigner: &mut KeyAssigner) -> Option<Keymap> {
        let mut layout = vec![KeyAssignment::U; 26];
        let _chars = char_def::definitions()
            .into_iter()
            .map(Some)
            .collect::<Vec<_>>();

        // まずシフトキーのシフト面に対して割り当てる。ここでは清音しか割り当てられない。
        let left = assigner.left_shift_key(rng);
        layout[LINEAR_L_SHIFT_INDEX] = KeyAssignment::A(KeyDef::from_combination(&left));
        let right = assigner.right_shift_key(rng, &left);
        layout[LINEAR_R_SHIFT_INDEX] = KeyAssignment::A(KeyDef::from_combination(&right));

        // 各場所にassignする
        Keymap::assign_keys(&mut layout, rng, assigner);
        let keymap = Keymap { layout };

        if !keymap.meet_requirements() {
            None
        } else {
            Some(keymap)
        }
    }

    /// キー全体の配置を行う
    ///
    /// ここでの配置は、すでに制約の多い部分は事前に設定してある状態なので、そのまま入れられるところに入れていけばよい
    fn assign_keys(layout: &mut [KeyAssignment], rng: &mut StdRng, assigner: &mut KeyAssigner) {
        // 各文字を設定していく。
        for idx in assigner.ordered_key_indices() {
            let assignment = &mut layout[idx];
            if idx == LINEAR_L_SHIFT_INDEX || idx == LINEAR_R_SHIFT_INDEX {
                continue;
            }

            if let Some(key) = assigner.pick_key(rng, idx) {
                *assignment = KeyAssignment::A(KeyDef::from_combination(&key));
            }
        }
    }

    /// keymap自体が、全体の要求を満たしているかどうかを確認する
    ///
    /// 制約条件としては以下となる。これらはconstraint moduleで定義されている
    /// * 左右のシフト文字が同一である
    /// * 濁音シフトのキーにはいずれかにしか濁音が設定されていない
    /// * 半濁音シフトのキー自体には、いずれかにしか半濁音が設定されていない
    /// * 左右の濁音・半濁音の間では、濁音と半濁音がいずれかにしか設定されていない
    ///
    /// # Returns
    /// 制約を満たしていたらtrue
    fn meet_requirements(&self) -> bool {
        let checks = [
            constraints::should_shift_having_same_key,
            constraints::should_shift_only_clear_tones,
            constraints::should_be_explicit_between_left_turbid_and_right_semiturbit,
            constraints::should_only_one_turbid,
            constraints::should_be_explicit_between_right_turbid_and_left_semiturbit,
            constraints::should_only_one_semiturbid,
            constraints::should_be_able_to_all_input,
        ];

        checks.iter().all(|c| c(&self.layout))
    }

    /// 指定したindex間でキーを入れ替える
    ///
    /// #Return
    /// 入れ替え後のキーマップ。制約を満たさない場合はNoneを返す
    pub fn swap_keys(&self, idx1: usize, idx2: usize) -> Vec<Self> {
        let mut new = self.clone();
        let mut vec = Vec::new();

        new.layout[idx1].swap();
        if new.meet_requirements() {
            vec.push(new.clone());
            new.layout[idx1].swap();
        }
        new.layout[idx2].swap();
        if new.meet_requirements() {
            vec.push(new.clone());
            new.layout[idx2].swap();
        }

        new.layout.swap(idx1, idx2);
        if new.meet_requirements() {
            vec.push(new.clone());
        }

        new.layout[idx1].swap();
        if new.meet_requirements() {
            vec.push(new.clone());
        }

        new.layout[idx1].swap();
        new.layout[idx2].swap();
        if new.meet_requirements() {
            vec.push(new.clone());
        }

        new.layout[idx1].swap();
        if new.meet_requirements() {
            vec.push(new.clone());
        }

        vec
    }

    /// 指定した文字を入力できるキーを返す
    ///
    /// 対象の文字が存在しない場合はNoneを返す
    pub fn get(&self, char: char) -> Option<(KeyKind, Point)> {
        let layout = linear::linear_layout();

        for (idx, assignment) in self.layout.iter().enumerate() {
            if assignment.contains(char) {
                let kind = match idx {
                    LINEAR_L_SHIFT_INDEX | LINEAR_R_SHIFT_INDEX => KeyKind::Shift,
                    LINEAR_L_TURBID_INDEX => KeyKind::Turbid,
                    LINEAR_R_TURBID_INDEX => KeyKind::Turbid,
                    LINEAR_L_SEMITURBID_INDEX => KeyKind::Semiturbid,
                    LINEAR_R_SEMITURBID_INDEX => KeyKind::Semiturbid,
                    _ => KeyKind::Normal,
                };

                return Some((kind, layout[idx]));
            }
        }

        None
    }

    /// https://github.com/mobitan/chutoro/tree/main/tools
    /// 上記での評価用にpairを生成する。
    ///
    /// # Arguments
    /// * `key_layout` - マッピング対象のalphabetキー
    ///
    /// # Returns
    /// 最初のセルにひらがな、２番めのセルにkeyのcombinationを返す
    pub fn key_combinations(&self, key_layout: &[[char; 10]; 3]) -> Vec<(String, String)> {
        let mut ret: Vec<(String, String)> = Vec::new();
        let layout = linear::linear_layout();

        for (r, key) in self.layout.iter().enumerate() {
            match key {
                KeyAssignment::A(key) => {
                    let (r, c): (usize, usize) = layout[r].into();
                    let unshift = key.unshift();
                    ret.push((unshift.to_string(), key_layout[r][c].to_string()));

                    let shifted = key.shifted();
                    {
                        let (sr, sc) = if c <= 4 {
                            layout[LINEAR_R_SHIFT_INDEX].into()
                        } else {
                            layout[LINEAR_L_SHIFT_INDEX].into()
                        };
                        let key = key_layout[sr][sc];
                        ret.push((shifted.to_string(), format!("{}{}", key, key_layout[r][c])));
                    }

                    if let Some(turbid) = key.turbid() {
                        let (sr, sc) = if c <= 4 {
                            layout[LINEAR_R_TURBID_INDEX].into()
                        } else {
                            layout[LINEAR_L_TURBID_INDEX].into()
                        };
                        let key = key_layout[sr][sc];
                        ret.push((turbid.to_string(), format!("{}{}", key, key_layout[r][c])));
                    }

                    if let Some(semiturbid) = key.semiturbid() {
                        let (sr, sc) = if c <= 4 {
                            layout[LINEAR_R_SEMITURBID_INDEX].into()
                        } else {
                            layout[LINEAR_L_SEMITURBID_INDEX].into()
                        };
                        let key = key_layout[sr][sc];

                        ret.push((
                            semiturbid.to_string(),
                            format!("{}{}", key, key_layout[r][c]),
                        ));
                    }
                }
                KeyAssignment::U => continue,
            }
        }

        ret
    }

    fn format_keymap(&self, layout: &[Option<char>]) -> String {
        let layout_mapping = linear::linear_layout();
        let header: String = (0..9)
            .map(|_| "┳".to_string())
            .collect::<Vec<_>>()
            .join("━");
        let header = format!("{}{}{}", "┏━", header, "━┓");

        let separator = format!(
            "{}{}{}\n",
            "┣━",
            (0..9)
                .map(|_| { "╋".to_string() })
                .collect::<Vec<String>>()
                .join("━"),
            "━┫"
        );
        let mut square_layout = vec![vec![None; 10]; 3];
        for (idx, ch) in layout.iter().enumerate() {
            let (r, c): (usize, usize) = layout_mapping[idx].into();

            square_layout[r][c] = *ch;
        }

        let keys: Vec<String> = square_layout
            .iter()
            .map(|row| {
                let row: Vec<String> = row
                    .iter()
                    .map(|k| k.map(|c| c.to_string()).unwrap_or("　".to_string()))
                    .collect();

                format!("┃{}┃\n", row.join("┃"))
            })
            .collect();

        let keys = keys.join(&separator);
        let footer = format!(
            "{}{}{}",
            "┗━",
            (0..9)
                .map(|_| "┻".to_string())
                .collect::<Vec<_>>()
                .join("━"),
            "━┛"
        );

        format!("{}\n{}{}\n", header, keys, footer)
    }

    fn format_unshift(&self) -> String {
        let keys = self
            .layout
            .iter()
            .map(|r| match r {
                KeyAssignment::A(k) => Some(k.unshift()),
                KeyAssignment::U => None,
            })
            .collect::<Vec<_>>();

        self.format_keymap(&keys)
    }

    fn format_shift(&self) -> String {
        let keys = self
            .layout
            .iter()
            .map(|r| match r {
                KeyAssignment::A(k) => Some(k.shifted()),
                KeyAssignment::U => None,
            })
            .collect::<Vec<_>>();

        self.format_keymap(&keys)
    }

    fn format_turbid(&self) -> String {
        let keys = self
            .layout
            .iter()
            .map(|r| match r {
                KeyAssignment::A(k) => k.turbid(),
                KeyAssignment::U => None,
            })
            .collect::<Vec<_>>();

        self.format_keymap(&keys)
    }

    fn format_semiturbid(&self) -> String {
        let keys = self
            .layout
            .iter()
            .map(|r| match r {
                KeyAssignment::A(k) => k.semiturbid(),
                KeyAssignment::U => None,
            })
            .collect::<Vec<_>>();

        self.format_keymap(&keys)
    }

    /// key defをiterateできるiteratorを返す
    pub fn iter(&self) -> KeymapIterator {
        KeymapIterator {
            keymap: self,
            index: 0,
        }
    }
}

pub struct KeymapIterator<'a> {
    keymap: &'a Keymap,
    index: usize,
}

impl<'a> Iterator for KeymapIterator<'a> {
    type Item = &'a KeyDef;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.keymap.layout.len() {
            // 割当のない場所がありうる
            if let KeyAssignment::A(k) = &self.keymap.layout[self.index] {
                self.index += 1;
                return Some(k);
            }
            self.index += 1;
        }

        None
    }
}

impl Display for Keymap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "generated layout\n\n
unshift:\n{}\n
shifted:\n{}\n
turbid:\n{}\n
semiturbid\n{}\n
",
            self.format_unshift(),
            self.format_shift(),
            self.format_turbid(),
            self.format_semiturbid()
        )
    }
}
