use std::fmt::Display;

use rand::{rngs::StdRng, Rng};

use crate::{
    char_def::{self, CharDef},
    frequency_table::FrequencyTable,
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

    /// assignのキーについて、内容をflipする
    fn flip(&self) -> Self {
        match self {
            KeyAssignment::A(k) => KeyAssignment::A(k.flip()),
            KeyAssignment::U => KeyAssignment::U,
        }
    }

    /// assignされているキーのうち、無シフト面を交換する
    fn swap_unshift(&self, other: &Self) -> Option<(Self, Self)> {
        match (self, other) {
            (KeyAssignment::A(l), KeyAssignment::A(r)) => {
                if l.unshift() == r.unshift() {
                    return None;
                }

                match (l.replace_unshift(r), r.replace_unshift(l)) {
                    (Some(l), Some(r)) => Some((KeyAssignment::A(l), KeyAssignment::A(r))),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// assignされているキーのうち、無シフト面を交換する
    fn swap_shifted(&self, other: &Self) -> Option<(Self, Self)> {
        match (self, other) {
            (KeyAssignment::A(l), KeyAssignment::A(r)) => {
                if l.unshift() == r.unshift() {
                    return None;
                }

                match (l.replace_shifted(r), r.replace_shifted(l)) {
                    (Some(l), Some(r)) => Some((KeyAssignment::A(l), KeyAssignment::A(r))),
                    _ => None,
                }
            }
            _ => None,
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
        char_def,
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

        match (left_shifted, right_shifted) {
            (KeyAssignment::A(l), KeyAssignment::A(r)) => {
                let l = l
                    .shifted()
                    .and_then(char_def::find)
                    .map_or(false, |c| c.is_cleartone());
                let r = r
                    .shifted()
                    .and_then(char_def::find)
                    .map_or(false, |c| c.is_cleartone());
                l && r
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
        use crate::key_def::KeyDef;

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

/// [defs]から条件に合致する中で、ランダムに一つの定義を選択する。選択された定義は `defs` から削除される
///
/// # Arguments
/// * `defs` - 文字のリスト
/// * `rng` - 乱数生成器
/// * `key_idx` - 対象のキーのindex
/// * `pred` - 条件を判定する関数
///
/// # Returns
/// ランダムに選択された文字定義
fn pick_def<F>(
    defs: &mut Vec<Option<CharDef>>,
    rng: &mut StdRng,
    key_idx: usize,
    freq_table: &FrequencyTable,
    f: F,
) -> (usize, CharDef)
where
    F: Fn(&CharDef) -> bool,
{
    let chars = defs
        .iter()
        .cloned()
        .map(|v| v.filter(|v| f(v)))
        .collect::<Vec<_>>();
    let (idx, char) = freq_table.get_char(&chars, key_idx, rng.gen::<f64>());

    defs[idx] = None;
    (
        idx,
        char_def::find(char).expect("can not found char: {char}"),
    )
}

impl Keymap {
    /// 指定されたseedを元にしてキーマップを生成する
    ///
    /// 生成されたkeymapは、あくまでランダムなキーマップであり、実際に利用するためには、[Keymap::meet_requirements]がtrueを返すことを前提としなければ
    /// ならない。
    pub fn generate(rng: &mut StdRng, freq_table: &FrequencyTable) -> Keymap {
        let mut layout = vec![KeyAssignment::U; 26];
        let mut chars = char_def::definitions().into_iter().map(Some).collect();

        // まずシフトキーのシフト面に対して割り当てる。ここでは清音しか割り当てられない。
        let (_, def) = pick_def(&mut chars, rng, LINEAR_L_SHIFT_INDEX, freq_table, |c| {
            c.is_cleartone()
        });
        layout[LINEAR_L_SHIFT_INDEX] = KeyAssignment::A(KeyDef::shifted_from(&def));
        layout[LINEAR_R_SHIFT_INDEX] = KeyAssignment::A(KeyDef::shifted_from(&def));

        // もっとも制約が強いハ行の文字から割り当てていく。ここでの割り当ては、シフトキーなどを除いて行っている
        Keymap::assign_ha_row(&mut layout, rng, &mut chars, freq_table);

        // 制約が多い半濁音を設定する
        Keymap::assign_semiturbids(&mut layout, rng, &mut chars, freq_table);

        // 制約が多い濁音を設定する
        Keymap::assign_turbids(&mut layout, rng, &mut chars, freq_table);

        // 残りの場所に追加していく。
        Keymap::assign_keys(&mut layout, rng, &mut chars, freq_table);

        if !chars.iter().all(|v| v.is_none()) {
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
    fn assign_ha_row(
        layout: &mut [KeyAssignment],
        rng: &mut StdRng,
        chars: &mut Vec<Option<CharDef>>,
        freq_table: &FrequencyTable,
    ) {
        let special_keys = linear::indices_of_special_keys();
        let ha_row = char_def::definitions()
            .into_iter()
            .filter(|c| {
                c.normal() == 'は'
                    || c.normal() == 'ひ'
                    || c.normal() == 'ふ'
                    || c.normal() == 'へ'
                    || c.normal() == 'ほ'
            })
            .collect::<Vec<_>>();

        for ha_col in ha_row {
            loop {
                let idx = rng.gen_range(0..layout.len());

                if special_keys.contains(&idx) || layout[idx] != KeyAssignment::U {
                    continue;
                }

                let (_, def) = pick_def(chars, rng, idx, freq_table, |c| {
                    c.normal() == ha_col.normal()
                });

                // どっちかに割り当てる
                if rng.gen::<bool>() {
                    layout[idx] = KeyAssignment::A(KeyDef::unshift_from(&def));
                } else {
                    layout[idx] = KeyAssignment::A(KeyDef::shifted_from(&def));
                }
                break;
            }
        }
    }

    /// 半濁音があるキーを設定する
    ///
    /// 半濁音は、濁音と半濁音キーに対しては設定しないものとする。これは、互いに濁音と半濁音に割り当ててしまうと、
    /// 確定することができなくなるためである。
    fn assign_semiturbids(
        layout: &mut [KeyAssignment],
        rng: &mut StdRng,
        chars: &mut Vec<Option<CharDef>>,
        freq_table: &FrequencyTable,
    ) {
        let special_keys = linear::indices_of_turbid_related_keys();
        let semiturbids = chars
            .iter()
            .cloned()
            .filter_map(|c| c.filter(|c| c.semiturbid().is_some()))
            .collect::<Vec<_>>();

        for ch in semiturbids {
            loop {
                let idx = rng.gen_range(0..layout.len());

                if special_keys.contains(&idx) || layout[idx] != KeyAssignment::U {
                    continue;
                }
                let (_, def) = pick_def(chars, rng, idx, freq_table, |c| *c == ch);

                // どっちかに割り当てる
                if rng.gen::<bool>() {
                    layout[idx] = KeyAssignment::A(KeyDef::unshift_from(&def));
                } else {
                    layout[idx] = KeyAssignment::A(KeyDef::shifted_from(&def));
                }
                break;
            }
        }
    }

    /// 濁音があるキーを設定する
    ///
    /// 濁音は、濁音シフト間では排他にしなければならない。
    fn assign_turbids(
        layout: &mut [KeyAssignment],
        rng: &mut StdRng,
        char_defs: &mut Vec<Option<CharDef>>,
        freq_table: &FrequencyTable,
    ) {
        // どっちかにすでに設定していたらそれ以上はやらないようにする
        let mut assigned_to_turbid = false;
        let turbids = char_defs
            .iter()
            .cloned()
            .filter_map(|c| c.filter(|c| c.turbid().is_some()))
            .collect::<Vec<_>>();

        for ch in turbids {
            loop {
                let idx = rng.gen_range(0..layout.len());

                if (idx == LINEAR_L_TURBID_INDEX || idx == LINEAR_R_TURBID_INDEX)
                    && assigned_to_turbid
                {
                    continue;
                }

                let (def_idx, def) = pick_def(char_defs, rng, idx, freq_table, |c| *c == ch);

                if let KeyAssignment::A(k) = &layout[idx] {
                    if let Some(k) = k.merge(&def) {
                        if idx == LINEAR_L_TURBID_INDEX || idx == LINEAR_R_TURBID_INDEX {
                            assigned_to_turbid = true;
                        }
                        layout[idx] = KeyAssignment::A(k);
                        break;
                    }
                } else {
                    if idx == LINEAR_L_TURBID_INDEX || idx == LINEAR_R_TURBID_INDEX {
                        assigned_to_turbid = true;
                    }

                    // どっちかに割り当てる
                    if rng.gen::<bool>() {
                        layout[idx] = KeyAssignment::A(KeyDef::unshift_from(&def));
                    } else {
                        layout[idx] = KeyAssignment::A(KeyDef::shifted_from(&def));
                    }
                    break;
                }

                // 入れられなかったら戻す
                char_defs[def_idx] = Some(def);
            }
        }
    }

    /// キー全体の配置を行う
    ///
    /// ここでの配置は、すでに制約の多い部分は事前に設定してある状態なので、そのまま入れられるところに入れていけばよい
    fn assign_keys(
        layout: &mut [KeyAssignment],
        rng: &mut StdRng,
        chars: &mut Vec<Option<CharDef>>,
        freq_table: &FrequencyTable,
    ) {
        // 各文字を設定していく。
        while !chars.iter().all(|v| v.is_none()) {
            // 入る場所を探す
            loop {
                let idx = rng.gen_range(0..layout.len());
                let assign_shift: bool = rng.gen();
                let (def_idx, def) = pick_def(chars, rng, idx, freq_table, |_| true);

                if let KeyAssignment::A(k) = &layout[idx] {
                    // この場合は、対象の場所に対してmerge出来るかどうかを確認する
                    if let Some(k) = k.merge(&def) {
                        layout[idx] = KeyAssignment::A(k);
                        break;
                    }
                } else {
                    if assign_shift {
                        layout[idx] = KeyAssignment::A(KeyDef::shifted_from(&def));
                    } else {
                        layout[idx] = KeyAssignment::A(KeyDef::unshift_from(&def));
                    }
                    break;
                }

                chars[def_idx] = Some(def)
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
            constraints::should_be_explicit_between_right_turbid_and_left_semiturbit,
            constraints::should_be_able_to_all_input,
        ];

        checks.iter().all(|c| c(&self.layout))
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

    /// keymapに対して操作を実行して、実行した結果のkeymapを返す
    pub fn mutate(&self, rng: &mut StdRng) -> Keymap {
        let mut keymap = self.clone();

        let operation: u32 = rng.gen_range(0..5);

        match operation {
            0 => keymap.swap_unshift_between_keys(rng),
            1 => keymap.swap_shifted_between_keys(rng),
            2 => keymap.flip_key(rng),
            _ => (),
        }

        keymap
    }

    /// 任意のkeyにおけるunshiftedを交換する。
    fn swap_unshift_between_keys(&mut self, rng: &mut StdRng) {
        loop {
            let idx1 = rng.gen_range(0..self.layout.len());
            let idx2 = rng.gen_range(0..self.layout.len());

            if idx1 == idx2 {
                continue;
            }
            let key1 = &self.layout[idx1];
            let key2 = &self.layout[idx2];

            if let Some((new_key1, new_key2)) = key1.swap_unshift(key2) {
                self.layout[idx1] = new_key1;
                self.layout[idx2] = new_key2;
                break;
            }
        }
    }

    /// 任意のkeyにおけるshiftedを交換する。ただし、シフトキー自体が対象になる場合は対象外とする。
    fn swap_shifted_between_keys(&mut self, rng: &mut StdRng) {
        loop {
            let idx1 = rng.gen_range(0..self.layout.len());
            let idx2 = rng.gen_range(0..self.layout.len());

            if idx1 == idx2
                || idx1 == LINEAR_L_SHIFT_INDEX
                || idx1 == LINEAR_R_SHIFT_INDEX
                || idx2 == LINEAR_L_SHIFT_INDEX
                || idx2 == LINEAR_R_SHIFT_INDEX
            {
                continue;
            }

            let key1 = &self.layout[idx1];
            let key2 = &self.layout[idx2];

            if let Some((new_key1, new_key2)) = key1.swap_shifted(key2) {
                self.layout[idx1] = new_key1;
                self.layout[idx2] = new_key2;

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
        loop {
            let idx = rng.gen_range(0..self.layout.len());
            if idx == LINEAR_L_SHIFT_INDEX || idx == LINEAR_R_SHIFT_INDEX {
                continue;
            }
            let key = &self.layout[idx];

            self.layout[idx] = key.flip();

            // シフトキーの場合は、対応するキーも一緒にflipする
            if idx == LINEAR_L_SHIFT_INDEX {
                self.layout[LINEAR_R_SHIFT_INDEX] = self.layout[LINEAR_R_SHIFT_INDEX].flip();
            } else if idx == LINEAR_R_SHIFT_INDEX {
                self.layout[LINEAR_L_SHIFT_INDEX] = self.layout[LINEAR_L_SHIFT_INDEX].flip();
            }
            break;
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
        let layout = linear::linear_layout();

        for (r, key) in self.layout.iter().enumerate() {
            match key {
                KeyAssignment::A(key) => {
                    let (r, c): (usize, usize) = layout[r].into();
                    if let Some(unshift) = key.unshift() {
                        ret.push((unshift.to_string(), key_layout[r][c].to_string()));
                    } else {
                        continue;
                    }

                    // 各シフトの場合は、それぞれ逆手で押下するものとする
                    if let Some(shifted) = key.shifted() {
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
                KeyAssignment::A(k) => k.unshift(),
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
                KeyAssignment::A(k) => k.shifted(),
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
