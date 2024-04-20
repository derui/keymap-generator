use std::{collections::HashSet, fmt::Display};

use rand::{rngs::StdRng, seq::SliceRandom, Rng};

use crate::{
    char_def::{self, CharDef},
    key_def::KeyDef,
    layout::linear::{self, LINEAR_L_SHIFT_INDEX, LINEAR_R_SHIFT_INDEX},
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

/// 有効なキーマップ
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Keymap {
    layout: Vec<KeyAssignment>,
}

// layout上で利用しないキーの位置
pub const EXCLUDE_MAP: [(usize, usize); 4] = [(0, 0), (0, 4), (0, 5), (0, 9)];
// 左手シフトキーのindex
pub const LEFT_SHIFT_INDEX: (usize, usize) = (1, 2);
// 右手シフトキーのindex
pub const RIGHT_SHIFT_INDEX: (usize, usize) = (1, 7);
// 左手濁音シフトキーのindex
pub const LEFT_TURBID_INDEX: (usize, usize) = (1, 3);
// 右手濁音シフトキーのindex
pub const RIGHT_TURBID_INDEX: (usize, usize) = (1, 6);
// 左手半濁音シフトキーのindex
pub const LEFT_SEMITURBID_INDEX: (usize, usize) = (2, 3);
// 右手半濁音シフトキーのindex
pub const RIGHT_SEMITURBID_INDEX: (usize, usize) = (2, 6);

/// このkeymapにおける、有効なキーのインデックスを返す
pub fn key_indices() -> HashSet<(usize, usize)> {
    let mut indices: HashSet<(usize, usize)> = (0..3)
        .flat_map(|r| (0..10).map(move |c| (r as usize, c as usize)))
        .collect();

    for exclude in EXCLUDE_MAP {
        indices.remove(&exclude);
    }

    indices
}

mod constraints {
    use std::collections::HashSet;

    use crate::{
        char_def,
        layout::linear::{
            LINEAR_L_SEMITURBID_INDEX, LINEAR_L_SHIFT_INDEX, LINEAR_L_TURBID_INDEX,
            LINEAR_R_SEMITURBID_INDEX, LINEAR_R_SHIFT_INDEX, LINEAR_R_TURBID_INDEX,
        },
    };

    use super::{
        KeyAssignment, LEFT_SEMITURBID_INDEX, LEFT_SHIFT_INDEX, LEFT_TURBID_INDEX,
        RIGHT_SEMITURBID_INDEX, RIGHT_SHIFT_INDEX, RIGHT_TURBID_INDEX,
    };

    /// 左右のシフトキーに割り当てられている文字が同一であるか確認する
    pub(super) fn should_shift_having_same_key(layout: &[KeyAssignment]) -> bool {
        let left_shifted = layout[LINEAR_L_SHIFT_INDEX];
        let right_shifted = layout[LINEAR_R_SHIFT_INDEX];

        match (left_shifted, right_shifted) {
            (KeyAssignment::A(l), KeyAssignment::A(r)) => l.shifted() == r.shifted(),
            _ => false,
        }
    }

    /// 左右のシフトキーには清音しか設定されていないかどうかを確認する
    pub(super) fn should_shift_only_clear_tones(layout: &[KeyAssignment]) -> bool {
        let left_shifted = layout[LINEAR_L_SHIFT_INDEX];
        let right_shifted = layout[LINEAR_R_SHIFT_INDEX];

        match (left_shifted, right_shifted) {
            (KeyAssignment::A(l), KeyAssignment::A(r)) => {
                let l = l
                    .shifted()
                    .and_then(|c| char_def::find(c))
                    .map_or(false, |c| c.is_cleartone());
                let r = r
                    .shifted()
                    .and_then(|c| char_def::find(c))
                    .map_or(false, |c| c.is_cleartone());
                l && r
            }
            _ => false,
        }
    }

    /// 左右の濁音シフト間では、いずれかのキーにしか濁音が設定されていないかどうかを確認する
    pub(super) fn should_only_one_turbid(layout: &[KeyAssignment]) -> bool {
        let left_turbid = layout[LINEAR_L_TURBID_INDEX];
        let right_turbid = layout[LINEAR_R_TURBID_INDEX];

        match (left_turbid, right_turbid) {
            (KeyAssignment::A(l), KeyAssignment::A(r)) => {
                let l = l.turbid();
                let r = r.turbid();

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
        let mut chars: HashSet<char> = char_def::all_chars().collect();

        for assignment in layout.iter() {
            match assignment {
                KeyAssignment::A(k) => k.chars().iter().all(|c| chars.remove(c)),
                _ => (),
            }
        }

        chars.is_empty()
    }

    #[cfg(test)]
    mod tests {
        use crate::{key::Key, key_def::KeyDef};

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
            put_key(&mut layout, KeyDef::shifted_from(&c1), LINEAR_L_SHIFT_INDEX);
            put_key(&mut layout, KeyDef::shifted_from(&c1), LINEAR_R_SHIFT_INDEX);

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
            put_key(&mut layout, KeyDef::shifted_from(&c1), LINEAR_L_SHIFT_INDEX);
            put_key(&mut layout, KeyDef::shifted_from(&c1), LINEAR_R_SHIFT_INDEX);

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
            put_key(&mut layout, KeyDef::shifted_from(&c1), LINEAR_L_SHIFT_INDEX);
            put_key(&mut layout, KeyDef::shifted_from(&c2), LINEAR_R_SHIFT_INDEX);

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
            put_key(
                &mut layout,
                KeyDef::unshift_from(&c1),
                LINEAR_L_TURBID_INDEX,
            );
            put_key(
                &mut layout,
                KeyDef::unshift_from(&c2),
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
            let c1 = char_def::find('し').unwrap();
            let c2 = char_def::find('か').unwrap();
            put_key(
                &mut layout,
                KeyDef::unshift_from(&c1),
                LINEAR_L_TURBID_INDEX,
            );
            put_key(
                &mut layout,
                KeyDef::unshift_from(&c2),
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
            let c1 = char_def::find('か').unwrap();
            let c2 = char_def::find('ま').unwrap();
            put_key(
                &mut layout,
                KeyDef::unshift_from(&c1),
                LINEAR_L_TURBID_INDEX,
            );
            put_key(
                &mut layout,
                KeyDef::unshift_from(&c2),
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
            let c1 = char_def::find('か').unwrap();
            let c2 = char_def::find('ま').unwrap();
            put_key(
                &mut layout,
                KeyDef::unshift_from(&c1),
                LINEAR_R_TURBID_INDEX,
            );
            put_key(
                &mut layout,
                KeyDef::unshift_from(&c2),
                LINEAR_L_SEMITURBID_INDEX,
            );

            // act
            let ret = should_be_explicit_between_right_turbid_and_left_semiturbit(&layout);

            // assert
            assert!(ret, "should be valid")
        }
    }
}

/// [defs]から条件に合致する中で、ランダムに一つの定義を選択する
///
/// # Arguments
/// * `defs` - 文字のリスト
/// * `rng` - 乱数生成器
/// * `pred` - 条件を判定する関数
///
/// # Returns
/// ランダムに選択された文字定義
fn pick_def(defs: &mut Vec<CharDef>, rng: &mut StdRng, f: dyn Fn(&CharDef) -> bool) -> char {
    let mut with_idx = defs
        .iter()
        .enumerate()
        .filter(|(_, v)| f(v))
        .collect::<Vec<_>>();
    let idx = rng.gen_range(0..with_idx.len());
    let c = defs.remove(idx);
    c
}

impl Keymap {
    /// 指定されたseedを元にしてキーマップを生成する
    ///
    /// 生成されたkeymapは、あくまでランダムなキーマップであり、実際に利用するためには、[Keymap::meet_requirements]がtrueを返すことを前提としなければ
    /// ならない。
    pub fn generate(rng: &mut StdRng) -> Keymap {
        let mut layout = vec![KeyAssignment::U; 26];
        let mut chars = char_def::CharDef::definitions();

        // まずシフトキーのシフト面に対して割り当てる。ここでは清音しか割り当てられない。
        let def = pick_def(&mut chars, rng, |c| c.is_cleartone());
        layout[LINEAR_L_SHIFT_INDEX] = KeyAssignment::A(KeyDef::shifted_from(def));
        layout[LINEAR_R_SHIFT_INDEX] = KeyAssignment::A(KeyDef::shifted_from(def));

        // 次に、もっとも制約が強いハ行の文字から割り当てていく。ここでの割り当ては、シフトキーなどを除いて行っている
        Keymap::assign_ha_row(&mut layout, rng, &mut chars);

        // 残りの場所に追加していく。
        Keymap::assign_keys(&mut layout, rng, &mut chars);

        if !chars.is_empty() {
            panic!("Leave some chars: {:?}", chars)
        }

        if !constraints::should_be_able_to_all_input(&layout) {
            panic!("Leave some chars: {}", Keymap { layout });
        }

        Keymap { layout }
    }

    /// ハ行を割り当てる
    ///
    /// ハ行は、その特殊性から、最初に割り当てることが望ましい。ただし、各シフトの場所には割り当てられないものとする
    fn assign_ha_row(layout: &mut [KeyAssignment], rng: &mut StdRng, chars: &mut Vec<CharDef>) {
        let special_keys = linear::indices_of_special_keys();
        let ha_row = chars
            .iter()
            .filter(|c| {
                c.unshift() == 'は'
                    || c.unshift() == 'ひ'
                    || c.unshift() == 'ふ'
                    || c.unshift() == 'へ'
                    || c.unshift() == 'ほ'
            })
            .collect::<Vec<_>>();

        for ha_col in ha_row {
            let def = pick_def(&mut chars, rng, |c| *c == *ha_col);

            loop {
                let idx = rng.gen_range(0..layout.len());

                if special_keys.contains(&idx) {
                    continue;
                }

                // どっちかに割り当てる
                if rng.gen::<bool>() {
                    layout[idx] = KeyAssignment::A(KeyDef::unshift_from(def));
                } else {
                    layout[idx] = KeyAssignment::A(KeyDef::shifted_from(def));
                }
            }
        }
    }

    /// キー全体の配置を行う
    ///
    /// ここでの配置は、すでに制約の多い部分は事前に設定してある状態なので、そのまま入れられるところに入れていけばよい
    fn assign_keys(layout: &mut [KeyAssignment], rng: &mut StdRng, chars: &mut Vec<CharDef>) {
        // 各文字を設定していく。
        loop {
            let def = pick_def(&mut chars, rng, |_| true);

            // 入る場所を探す
            loop {
                let idx = rng.gen_range(0..layout.len());
                let assign_shift = rng.gen::<bool>();

                if let KeyAssignment::A(k) = layout[idx] {
                    // この場合は、対象の場所に対してmerge出来るかどうかを確認する
                    if let Some(k) = k.merge(&def) {
                        layout[idx] = KeyAssignment::A(k);
                        break;
                    } else {
                        continue;
                    }
                } else {
                    if assign_shift {
                        layout[idx] = KeyAssignment::A(KeyDef::shifted_from(def));
                    } else {
                        layout[idx] = KeyAssignment::A(KeyDef::unshift_from(def));
                    }
                    break;
                }
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
    pub fn meet_requirements(&self) -> bool {
        let checks = [
            constraints::should_shift_having_same_key,
            constraints::should_shift_only_clear_tones,
            constraints::should_be_explicit_between_left_turbid_and_right_semiturbit,
            constraints::should_only_one_turbid,
            constraints::should_only_one_semiturbit,
            constraints::should_be_explicit_between_right_turbid_and_left_semiturbit,
            constraints::should_be_able_to_all_input,
        ];

        checks.iter().all(|c| c(&self.layout))
    }

    /// 指定した文字を入力できるキーを返す
    ///
    /// 対象の文字が存在しない場合はNoneを返す
    pub fn get(&self, char: char) -> Option<(KeyKind, (usize, usize))> {
        for (r, c) in key_indices() {
            let key = &self.layout[r][c];

            if key.contains(char) {
                let kind = match key {
                    Key::Normal(_) => KeyKind::Normal,
                    Key::Shifter(_) => KeyKind::Shift,
                    Key::Turbid(_) => KeyKind::Turbid,
                    Key::Semiturbid(_) => KeyKind::Semiturbid,
                    Key::Empty => return None,
                };
                return Some((kind, (r, c)));
            }
        }

        None
    }
    /// ランダムに現状のキーマップを組み替える
    ///
    /// 単にランダムな交叉を実行した場合は、そもそも適応関数を満たさない可能性がほぼ100%であり、ほぼ実用に適さない。
    /// ただし、一部分ずつの変異を繰り返しても、局所最適解に陥った場合には、それを脱出することができない。
    ///
    /// # Arguments
    /// * `keymap` - 現在のキーマップ
    ///
    /// # Returns
    /// 組み替え後のキーマップ

    pub fn imitate_cross(&self, rng: &mut StdRng) -> Keymap {
        let mut keymap = self.clone();

        let mut indices = key_indices();
        for p in EXCLUDE_MAP.iter() {
            indices.remove(&p);
        }
        indices.remove(&LEFT_SHIFT_INDEX);
        indices.remove(&RIGHT_SHIFT_INDEX);
        indices.remove(&LEFT_TURBID_INDEX);
        indices.remove(&RIGHT_TURBID_INDEX);
        indices.remove(&LEFT_SEMITURBID_INDEX);
        indices.remove(&RIGHT_SEMITURBID_INDEX);

        let indices = indices.into_iter().collect::<Vec<_>>();
        let mut random_indices = (0..indices.len()).collect::<Vec<_>>();
        random_indices.shuffle(rng);

        // ここでの交叉は、一旦制約を無視してランダムに組み替える
        for i in 0..(indices.len() / 2) {
            if rng.gen_range(0..2) == 0 {
                continue;
            }
            let pos1 = indices[random_indices[i * 2]];
            let pos2 = indices[random_indices[i * 2 + 1]];

            let key1 = &self.layout[pos1.0][pos1.1];
            let key2 = &self.layout[pos2.0][pos2.1];

            keymap.layout[pos1.0][pos1.1] = key2.clone();
            keymap.layout[pos2.0][pos2.1] = key1.clone();
        }

        keymap
    }

    /// keymapに対して操作を実行して、実行した結果のkeymapを返す
    pub fn mutate(&self, rng: &mut StdRng) -> Keymap {
        let mut keymap = self.clone();

        let operation: u32 = rng.gen_range(0..5);

        match operation {
            0 => keymap.swap_unshift_between_keys(rng),
            1 => keymap.swap_shifted_between_keys(rng),
            2 => keymap.flip_key(rng),
            3 => keymap.swap_shift_of_shift_key(rng),
            4 => keymap.swap_shift_and_unshift(rng),
            _ => (),
        }

        keymap
    }

    /// 任意のkeyにおけるunshiftedを交換する。
    fn swap_unshift_between_keys(&mut self, rng: &mut StdRng) {
        let layout = Vec::from_iter(key_indices());

        loop {
            let idx1 = rng.gen_range(0..layout.len());
            let idx2 = rng.gen_range(0..layout.len());

            if idx1 == idx2 {
                continue;
            }
            let pos1 = layout[idx1];
            let pos2 = layout[idx2];

            let key1 = &self.layout[pos1.0][pos1.1];
            let key2 = &self.layout[pos2.0][pos2.1];

            if let Some((new_key1, new_key2)) = key1.swap_unshifted(key2) {
                self.layout[pos1.0][pos1.1] = new_key1;
                self.layout[pos2.0][pos2.1] = new_key2;
                break;
            }
        }
    }

    /// 任意のkeyにおけるshiftedを交換する。ただし、シフトキー自体が対象になる場合は対象外とする。
    fn swap_shifted_between_keys(&mut self, rng: &mut StdRng) {
        let layout = Vec::from_iter(key_indices());

        loop {
            let idx1 = rng.gen_range(0..layout.len());
            let idx2 = rng.gen_range(0..layout.len());

            if idx1 == idx2 {
                continue;
            }
            let pos1 = layout[idx1];
            let pos2 = layout[idx2];

            if pos1 == LEFT_SHIFT_INDEX
                || pos1 == RIGHT_SHIFT_INDEX
                || pos2 == LEFT_SHIFT_INDEX
                || pos2 == RIGHT_SHIFT_INDEX
            {
                continue;
            }

            let key1 = &self.layout[pos1.0][pos1.1];
            let key2 = &self.layout[pos2.0][pos2.1];

            if let Some((new_key1, new_key2)) = key1.swap_shifted(key2) {
                self.layout[pos1.0][pos1.1] = new_key1;
                self.layout[pos2.0][pos2.1] = new_key2;

                break;
            }
        }
    }

    /// 任意のキーとshiftのshift面を取り替える
    fn swap_shift_of_shift_key(&mut self, rng: &mut StdRng) {
        let layout = Vec::from_iter(key_indices());

        loop {
            let idx = rng.gen_range(0..layout.len());
            let pos = layout[idx];

            if pos == LEFT_SHIFT_INDEX || pos == RIGHT_SHIFT_INDEX {
                continue;
            }

            let key1 = &self.layout[pos.0][pos.1];
            let left_shift = &self.layout[LEFT_SHIFT_INDEX.0][LEFT_SHIFT_INDEX.1];
            let right_shift = &self.layout[RIGHT_SHIFT_INDEX.0][RIGHT_SHIFT_INDEX.1];

            if let Some((new_key1, new_key2)) = key1.swap_shifted(left_shift) {
                if let Some((_, right_shift)) = key1.swap_shifted(right_shift) {
                    self.layout[pos.0][pos.1] = new_key1;
                    self.layout[LEFT_SHIFT_INDEX.0][LEFT_SHIFT_INDEX.1] = new_key2;
                    self.layout[RIGHT_SHIFT_INDEX.0][RIGHT_SHIFT_INDEX.1] = right_shift;

                    break;
                }
            }
        }
    }

    /// 任意のキー同士のshiftとunshiftを交換する
    ///
    /// ただし、対象のキーがシフトキーの場合は除外する
    fn swap_shift_and_unshift(&mut self, rng: &mut StdRng) {
        let layout = Vec::from_iter(key_indices());

        loop {
            let idx1 = rng.gen_range(0..layout.len());
            let idx2 = rng.gen_range(0..layout.len());

            if idx1 == idx2 {
                continue;
            }
            let pos1 = layout[idx1];
            let pos2 = layout[idx2];

            if pos1 == LEFT_SHIFT_INDEX
                || pos1 == RIGHT_SHIFT_INDEX
                || pos2 == LEFT_SHIFT_INDEX
                || pos2 == RIGHT_SHIFT_INDEX
            {
                continue;
            }

            let key1 = &self.layout[pos1.0][pos1.1];
            let key2 = &self.layout[pos2.0][pos2.1];
            let key1 = key1.flip();

            if let Some((new_key1, new_key2)) = key1.swap_shifted(key2) {
                self.layout[pos1.0][pos1.1] = new_key1;
                self.layout[pos2.0][pos2.1] = new_key2;

                break;
            }
        }
    }

    /// 任意のkeyにおける無シフト面とシフト面を交換する。
    ///
    /// # Arguments
    /// * `rng` - 乱数生成機
    ///
    /// # Return
    /// 交換されたkeymap
    fn flip_key(&mut self, rng: &mut StdRng) {
        let layout = Vec::from_iter(key_indices());

        let idx = rng.gen_range(0..layout.len());
        let pos = layout[idx];

        self.layout[pos.0][pos.1] = self.layout[pos.0][pos.1].flip();

        // シフトキーの場合は、対応するキーも一緒にflipする
        if pos == LEFT_SHIFT_INDEX {
            self.layout[RIGHT_SHIFT_INDEX.0][RIGHT_SHIFT_INDEX.1] =
                self.layout[RIGHT_SHIFT_INDEX.0][RIGHT_SHIFT_INDEX.1].flip();
        } else if pos == RIGHT_SHIFT_INDEX {
            self.layout[LEFT_SHIFT_INDEX.0][LEFT_SHIFT_INDEX.1] =
                self.layout[LEFT_SHIFT_INDEX.0][LEFT_SHIFT_INDEX.1].flip();
        }
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

        for (r, rows) in self.layout.iter().enumerate() {
            for (c, key) in rows.iter().enumerate() {
                if matches!(key, Key::Empty) {
                    continue;
                }

                ret.push((key.unshifted().to_string(), key_layout[r][c].to_string()));

                // 各シフトの場合は、それぞれ逆手で押下するものとする
                if let Some(shifted) = key.shifted() {
                    let key = if c <= 4 {
                        key_layout[RIGHT_SHIFT_INDEX.0][RIGHT_SHIFT_INDEX.1]
                    } else {
                        key_layout[LEFT_SHIFT_INDEX.0][LEFT_SHIFT_INDEX.1]
                    };
                    ret.push((shifted.to_string(), format!("{}{}", key, key_layout[r][c])));
                }

                if let Some(turbid) = key.turbid() {
                    let key = if c <= 4 {
                        key_layout[RIGHT_TURBID_INDEX.0][RIGHT_TURBID_INDEX.1]
                    } else {
                        key_layout[LEFT_TURBID_INDEX.0][LEFT_TURBID_INDEX.1]
                    };
                    ret.push((turbid.to_string(), format!("{}{}", key, key_layout[r][c])));
                }

                if let Some(semiturbid) = key.semiturbid() {
                    let key = if c <= 4 {
                        key_layout[RIGHT_SEMITURBID_INDEX.0][RIGHT_SEMITURBID_INDEX.1]
                    } else {
                        key_layout[LEFT_SEMITURBID_INDEX.0][LEFT_SEMITURBID_INDEX.1]
                    };

                    ret.push((
                        semiturbid.to_string(),
                        format!("{}{}", key, key_layout[r][c]),
                    ));
                }
            }
        }

        ret
    }

    fn format_keymap(&self, layout: &Vec<Vec<Option<char>>>) -> String {
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

        let keys: Vec<String> = layout
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
            .map(|r| r.iter().map(|c| Some(c.unshifted())).collect())
            .collect();

        self.format_keymap(&keys)
    }

    fn format_shift(&self) -> String {
        let keys = self
            .layout
            .iter()
            .map(|r| r.iter().map(|c| c.shifted()).collect())
            .collect();

        self.format_keymap(&keys)
    }

    fn format_turbid(&self) -> String {
        let keys = self
            .layout
            .iter()
            .map(|r| r.iter().map(|c| c.turbid()).collect())
            .collect();

        self.format_keymap(&keys)
    }

    fn format_semiturbid(&self) -> String {
        let keys = self
            .layout
            .iter()
            .map(|r| r.iter().map(|c| c.semiturbid()).collect())
            .collect();

        self.format_keymap(&keys)
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
