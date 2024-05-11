use std::{collections::HashMap, fmt::Display};

use rand::rngs::StdRng;

use crate::{
    char_def,
    frequency_table::KeyAssigner,
    key_def::KeyDef,
    key_seq::KeySeq,
    layout::{
        linear::{self, LINEAR_L_SHIFT_INDEX, LINEAR_R_SHIFT_INDEX},
        Point,
    },
};

#[derive(Debug, PartialEq, Eq, Clone)]
enum KeyAssignment {
    /// 割当済。変更することも出来る。
    A(KeyDef),

    /// 未割当
    U,
}

impl KeyAssignment {
    /// 対象の文字の入力時の種別を返す
    fn as_turbid(&self, point: &Point) -> Option<KeySeq> {
        match self {
            KeyAssignment::A(k) => k.as_turbid(point),
            KeyAssignment::U => None,
        }
    }

    /// 対象の文字の入力時の種別を返す
    fn as_semiturbid(&self, point: &Point) -> Option<KeySeq> {
        match self {
            KeyAssignment::A(k) => k.as_semiturbid(point),
            KeyAssignment::U => None,
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

mod constraints {
    use std::collections::HashSet;

    use crate::{
        char_def::{self, definitions},
        layout::linear::{LINEAR_L_SHIFT_INDEX, LINEAR_R_SHIFT_INDEX},
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

    /// すべての文字が入力できる状態であることを確認する
    pub(super) fn should_be_able_to_all_input(layout: &[KeyAssignment]) -> bool {
        let mut chars: HashSet<char> = char_def::all_chars()
            .into_iter()
            .filter(|v| *v != '、' && *v != '。')
            .collect();

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
    }
}

/// 有効なキーマップ
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Keymap {
    layout: Vec<KeyAssignment>,
    sequences: HashMap<char, KeySeq>,
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

        // まずシフトキーに対して割り当てる
        let left = assigner.left_shift_key(rng);
        layout[LINEAR_L_SHIFT_INDEX] = KeyAssignment::A(KeyDef::from_combination(&left));
        let right = assigner.right_shift_key(rng, &left);
        layout[LINEAR_R_SHIFT_INDEX] = KeyAssignment::A(KeyDef::from_combination(&right));

        // 各場所にassignする
        Keymap::assign_keys(&mut layout, rng, assigner);

        if !Keymap::meet_requirements(&layout) {
            None
        } else {
            let sequences = Keymap::build_sequences(&layout);

            let keymap = Keymap { layout, sequences };
            Some(keymap)
        }
    }

    /// charとsequenceのmappingを生成する
    fn build_sequences(layout: &[KeyAssignment]) -> HashMap<char, KeySeq> {
        let mut sequences = HashMap::new();
        let linear_layout = linear::linear_layout();

        let turbid_pos = layout
            .iter()
            .position(|v| v.as_turbid(&From::from((0, 0))).is_some())
            .expect("should have turbid");
        let semiturbid_pos = layout
            .iter()
            .position(|v| v.as_semiturbid(&From::from((0, 0))).is_some())
            .expect("should have turbid");
        let turbid_seq = KeySeq::from_shift(
            char_def::CharDef::Turbid.normal(),
            &linear_layout[turbid_pos],
        );
        let semiturbid_seq = KeySeq::from_shift(
            char_def::CharDef::SemiTurbid.normal(),
            &linear_layout[semiturbid_pos],
        );

        for (idx, assignment) in layout.iter().enumerate() {
            if let KeyAssignment::A(k) = assignment {
                let p = linear_layout[idx];
                let unshift = KeySeq::from_unshift(k.unshift(), &p);
                let shifted = KeySeq::from_shift(k.shifted(), &p);
                sequences.insert(k.unshift(), unshift);
                sequences.insert(k.shifted(), shifted);

                if let Some(turbid) = k.turbid() {
                    let turbid_seq = KeySeq::from_turbid_like(turbid, &p, &turbid_seq);
                    sequences.insert(turbid, turbid_seq);
                }

                if let Some(semiturbid) = k.semiturbid() {
                    let semiturbid_seq = KeySeq::from_turbid_like(semiturbid, &p, &semiturbid_seq);
                    sequences.insert(semiturbid, semiturbid_seq);
                }
            }
        }

        {
            let reading_pos = linear::reading_point_points();
            sequences.insert(
                '、',
                KeySeq::from_shift_like('。', &reading_pos[0], &reading_pos[1]),
            );

            let punctuation_pos = linear::punctuation_mark_points();
            sequences.insert(
                '。',
                KeySeq::from_shift_like('、', &punctuation_pos[0], &punctuation_pos[1]),
            );
        }
        sequences
    }

    /// キー全体の配置を行う
    ///
    /// ここでの配置は、すでに制約の多い部分は事前に設定してある状態なので、そのまま入れられるところに入れていけばよい
    fn assign_keys(layout: &mut [KeyAssignment], rng: &mut StdRng, assigner: &mut KeyAssigner) {
        // 各文字を設定していく。
        for idx in assigner.ordered_key_indices() {
            if idx == LINEAR_L_SHIFT_INDEX || idx == LINEAR_R_SHIFT_INDEX {
                continue;
            }

            let assignment = &mut layout[idx];
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
    fn meet_requirements(layout: &[KeyAssignment]) -> bool {
        let checks = [
            constraints::should_shift_having_same_key,
            // constraints::should_shift_only_clear_tones,
            // constraints::should_be_explicit_between_left_turbid_and_right_semiturbit,
            // constraints::should_only_one_turbid,
            // constraints::should_be_explicit_between_right_turbid_and_left_semiturbit,
            // constraints::should_only_one_semiturbid,
            constraints::should_be_able_to_all_input,
        ];

        checks.iter().all(|c| c(layout))
    }

    /// 指定したindex間でキーを入れ替える
    ///
    /// #Return
    /// 入れ替え後のキーマップ。制約を満たさない場合はNoneを返す
    pub fn swap_keys(&self, idx1: usize, idx2: usize) -> Vec<Self> {
        let mut new = self.clone();
        let mut vec = Vec::new();

        new.layout[idx1].swap();
        if Keymap::meet_requirements(&new.layout) {
            vec.push(Keymap {
                layout: new.layout.clone(),
                sequences: Keymap::build_sequences(&new.layout),
            });
            new.layout[idx1].swap();
        }
        new.layout[idx2].swap();
        if Keymap::meet_requirements(&new.layout) {
            vec.push(Keymap {
                layout: new.layout.clone(),
                sequences: Keymap::build_sequences(&new.layout),
            });
            new.layout[idx2].swap();
        }

        new.layout.swap(idx1, idx2);
        if Keymap::meet_requirements(&new.layout) {
            vec.push(Keymap {
                layout: new.layout.clone(),
                sequences: Keymap::build_sequences(&new.layout),
            });
        }

        new.layout[idx1].swap();
        if Keymap::meet_requirements(&new.layout) {
            vec.push(Keymap {
                layout: new.layout.clone(),
                sequences: Keymap::build_sequences(&new.layout),
            });
        }

        new.layout[idx1].swap();
        new.layout[idx2].swap();
        if Keymap::meet_requirements(&new.layout) {
            vec.push(Keymap {
                layout: new.layout.clone(),
                sequences: Keymap::build_sequences(&new.layout),
            });
        }

        new.layout[idx1].swap();
        if Keymap::meet_requirements(&new.layout) {
            vec.push(Keymap {
                layout: new.layout.clone(),
                sequences: Keymap::build_sequences(&new.layout),
            });
        }

        vec
    }

    /// 指定した文字を入力できるキーを返す
    ///
    /// 対象の文字が存在しない場合はNoneを返す
    pub fn get(&self, char: char) -> Option<KeySeq> {
        self.sequences.get(&char).cloned()
    }

    /// https://github.com/mobitan/chutoro/tree/main/tools
    /// 上記での評価用にpairを生成する。生成されるkeymapは、qwerty配列である
    ///
    /// # Arguments
    /// * `key_layout` - マッピング対象のalphabetキー
    ///
    /// # Returns
    /// 最初のセルにひらがな、２番めのセルにkeyのcombinationを返す
    pub fn key_combinations(&self, key_layout: &[[char; 10]; 3]) -> Vec<(String, String)> {
        let mut ret: Vec<(String, String)> = Vec::new();

        for (r, (_, seq)) in self.sequences.iter().enumerate() {
            ret.push((seq.char().to_string(), seq.to_char_sequence()))
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
        if self.index >= self.keymap.layout.len() {
            return None;
        }

        if let KeyAssignment::A(k) = &self.keymap.layout[self.index] {
            self.index += 1;
            return Some(k);
        }
        unreachable!("All keys must be assigned");
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
