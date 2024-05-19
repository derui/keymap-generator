use std::{collections::HashMap, fmt::Display};

use rand::rngs::StdRng;

use crate::{
    char_def,
    frequency_table::KeyAssigner,
    key_def::KeyDef,
    key_seq::KeySeq,
    layout::linear::{
        self, LINEAR_L_SEMITURBID_INDEX, LINEAR_L_SHIFT_INDEX, LINEAR_L_TURBID_INDEX,
        LINEAR_R_SEMITURBID_INDEX, LINEAR_R_SHIFT_INDEX, LINEAR_R_TURBID_INDEX,
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
            .filter(|v| v.is_cleartone() && !v.is_sulphuric())
            .map(|v| v.normal())
            .collect::<Vec<char>>();
        match (left_shifted, right_shifted) {
            (KeyAssignment::A(l), KeyAssignment::A(r)) => {
                let shifted_l = l.shifted();
                let shifted_r = r.shifted();
                let unshift_l = l.unshift();
                let unshift_r = r.unshift();
                cleartones.contains(&shifted_l)
                    && cleartones.contains(&shifted_r)
                    && cleartones.contains(&unshift_l)
                    && cleartones.contains(&unshift_r)
            }
            _ => false,
        }
    }

    /// 各キーには、濁音が一つ以下しか設定されていないかどうかを確認する
    pub(super) fn should_have_only_one_turbid(layout: &[KeyAssignment]) -> bool {
        layout.iter().all(|v| match v {
            KeyAssignment::A(k) => {
                let unshift = k.unshift_def().and_then(|v| v.turbid());
                let shifted = k.shifted_def().and_then(|v| v.turbid());
                matches!(
                    (unshift, shifted),
                    (Some(_), None) | (None, Some(_)) | (None, None)
                )
            }
            KeyAssignment::U => true,
        })
    }

    /// 各キーには、拗音対象が一つ以下しか設定されていないかどうかを確認する
    pub(super) fn should_have_only_one_sulphuric(layout: &[KeyAssignment]) -> bool {
        layout.iter().all(|v| match v {
            KeyAssignment::A(k) => {
                let unshift = k.unshift_def().map_or(false, |v| v.is_sulphuric());
                let shifted = k.shifted_def().map_or(false, |v| v.is_sulphuric());
                matches!(
                    (unshift, shifted),
                    (false, true) | (true, false) | (false, false)
                )
            }
            KeyAssignment::U => true,
        })
    }

    /// 濁音シフトには、濁音が一つ以下しか設定されていないかどうかを確認する
    pub(super) fn should_have_only_one_turbid_in_turbid_shifts(layout: &[KeyAssignment]) -> bool {
        let left = &layout[LINEAR_L_TURBID_INDEX];
        let right = &layout[LINEAR_R_TURBID_INDEX];

        match (left, right) {
            (KeyAssignment::A(left), KeyAssignment::A(right)) => {
                let left_unshift = left.unshift_def().and_then(|v| v.turbid());
                let left_shifted = left.shifted_def().and_then(|v| v.turbid());
                let right_unshift = right.unshift_def().and_then(|v| v.turbid());
                let right_shifted = right.shifted_def().and_then(|v| v.turbid());

                matches!(
                    (left_unshift, left_shifted, right_unshift, right_shifted),
                    (Some(_), None, None, None)
                        | (None, Some(_), None, None)
                        | (None, None, Some(_), None)
                        | (None, None, None, Some(_))
                        | (None, None, None, None),
                )
            }
            _ => true,
        }
    }

    /// 半濁音シフトには、半濁音が一つ以下しか設定されていないかどうかを確認する
    pub(super) fn should_have_only_one_semiturbid_in_semiturbid_shifts(
        layout: &[KeyAssignment],
    ) -> bool {
        let left = &layout[LINEAR_L_SEMITURBID_INDEX];
        let right = &layout[LINEAR_R_SEMITURBID_INDEX];

        match (left, right) {
            (KeyAssignment::A(left), KeyAssignment::A(right)) => {
                let left_unshift = left.unshift_def().and_then(|v| v.semiturbid());
                let left_shifted = left.shifted_def().and_then(|v| v.semiturbid());
                let right_unshift = right.unshift_def().and_then(|v| v.semiturbid());
                let right_shifted = right.shifted_def().and_then(|v| v.semiturbid());

                matches!(
                    (left_unshift, left_shifted, right_unshift, right_shifted),
                    (Some(_), None, None, None)
                        | (None, Some(_), None, None)
                        | (None, None, Some(_), None)
                        | (None, None, None, Some(_))
                        | (None, None, None, None),
                )
            }
            _ => true,
        }
    }

    /// 各キーには、半濁音は一つ以下しか設定されていないかどうかを確認する
    pub(super) fn should_have_only_one_semiturbid(layout: &[KeyAssignment]) -> bool {
        layout.iter().all(|v| match v {
            KeyAssignment::A(k) => {
                let unshift = k.unshift_def().and_then(|v| v.semiturbid());
                let shifted = k.shifted_def().and_then(|v| v.semiturbid());
                matches!(
                    (unshift, shifted),
                    (Some(_), None) | (None, Some(_)) | (None, None)
                )
            }
            KeyAssignment::U => true,
        })
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
        use crate::{frequency_layer::LayeredCharCombination, key_def::KeyDef};

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
            let comb1 = LayeredCharCombination::new(&[
                ("normal".to_string(), Some(c2)),
                ("shift".to_string(), Some(c1)),
            ]);
            let comb2 = LayeredCharCombination::new(&[
                ("normal".to_string(), Some(c3)),
                ("shift".to_string(), Some(c1)),
            ]);
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
            let comb1 = LayeredCharCombination::new(&[
                ("normal".to_string(), Some(c2)),
                ("shift".to_string(), Some(c1)),
            ]);
            let comb2 = LayeredCharCombination::new(&[
                ("normal".to_string(), Some(c3)),
                ("shift".to_string(), Some(c1)),
            ]);
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
            let comb1 = LayeredCharCombination::new(&[
                ("normal".to_string(), Some(c1)),
                ("shift".to_string(), Some(c2)),
            ]);
            let comb2 = LayeredCharCombination::new(&[
                ("normal".to_string(), Some(c3)),
                ("shift".to_string(), Some(c4)),
            ]);
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

        // まずシフトキーに対して割り当てる
        let left = assigner.left_shift_key(rng);
        layout[LINEAR_L_SHIFT_INDEX] = KeyAssignment::A(KeyDef::from_combination(&left));
        let right = assigner.right_shift_key(rng, &left);
        layout[LINEAR_R_SHIFT_INDEX] = KeyAssignment::A(KeyDef::from_combination(&right));
        layout[LINEAR_L_TURBID_INDEX] = KeyAssignment::A(KeyDef::from_combination(
            &assigner.pick_key(rng, LINEAR_L_TURBID_INDEX),
        ));
        layout[LINEAR_R_TURBID_INDEX] = KeyAssignment::A(KeyDef::from_combination(
            &assigner.pick_key(rng, LINEAR_R_TURBID_INDEX),
        ));
        layout[LINEAR_L_SEMITURBID_INDEX] = KeyAssignment::A(KeyDef::from_combination(
            &assigner.pick_key(rng, LINEAR_L_SEMITURBID_INDEX),
        ));
        layout[LINEAR_R_SEMITURBID_INDEX] = KeyAssignment::A(KeyDef::from_combination(
            &assigner.pick_key(rng, LINEAR_R_SEMITURBID_INDEX),
        ));

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

        for (idx, assignment) in layout.iter().enumerate() {
            if let KeyAssignment::A(k) = assignment {
                let p = linear_layout[idx];
                let unshift = KeySeq::from_unshift(k.unshift(), &p);
                let shifted = KeySeq::from_shift(k.shifted(), &p);
                sequences.insert(k.unshift(), unshift);
                sequences.insert(k.shifted(), shifted);

                if let Some(turbid) = k.turbid() {
                    let turbid_pos = match linear::get_hand_of_point(&p) {
                        crate::layout::Hand::Right => linear_layout[LINEAR_L_TURBID_INDEX],
                        crate::layout::Hand::Left => linear_layout[LINEAR_R_TURBID_INDEX],
                    };
                    let turbid_seq = KeySeq::from_shift_like(turbid, &p, &turbid_pos);
                    sequences.insert(turbid, turbid_seq);
                }

                if let Some(semiturbid) = k.semiturbid() {
                    let semiturbid_pos = match linear::get_hand_of_point(&p) {
                        crate::layout::Hand::Right => linear_layout[LINEAR_L_SEMITURBID_INDEX],
                        crate::layout::Hand::Left => linear_layout[LINEAR_R_SEMITURBID_INDEX],
                    };
                    let semiturbid_seq = KeySeq::from_shift_like(semiturbid, &p, &semiturbid_pos);
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
        for idx in assigner.ordered_key_indices(rng) {
            if let KeyAssignment::A(_) = layout[idx] {
                continue;
            }

            let assignment = &mut layout[idx];
            let key = assigner.pick_key(rng, idx);
            *assignment = KeyAssignment::A(KeyDef::from_combination(&key));
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
            constraints::should_shift_only_clear_tones,
            constraints::should_have_only_one_turbid,
            constraints::should_have_only_one_semiturbid,
            constraints::should_have_only_one_turbid_in_turbid_shifts,
            constraints::should_have_only_one_semiturbid_in_semiturbid_shifts,
            constraints::should_have_only_one_sulphuric,
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
        }
        new.layout[idx1].swap();

        new.layout[idx2].swap();
        if Keymap::meet_requirements(&new.layout) {
            vec.push(Keymap {
                layout: new.layout.clone(),
                sequences: Keymap::build_sequences(&new.layout),
            });
        }
        new.layout[idx2].swap();

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
    pub fn key_combinations(&self) -> Vec<(String, String)> {
        let mut ret: Vec<(String, String)> = Vec::new();

        for (_, seq) in self.sequences.iter() {
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
