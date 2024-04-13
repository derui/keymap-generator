use std::{collections::HashSet, fmt::Display};

use rand::{rngs::StdRng, Rng};

use crate::{char_def, key::Key};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum KeyKind {
    Normal,
    Shift,
    Turbid,
    Semiturbid,
}

/// 有効なキーマップ
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Keymap {
    /// keymapの世代
    generation: u64,
    layout: Vec<Vec<Key>>,
}

/// [chars]からランダムに一文字取得する
///
/// # Arguments
/// * `chars` - 文字のリスト
/// * `rng` - 乱数生成器
///
/// # Returns
/// ランダムに選択された文字
fn pick_char(chars: &mut Vec<char>, rng: &mut StdRng) -> char {
    let idx = rng.gen_range(0..chars.len());
    let c = chars[idx];
    chars.remove(idx);
    c
}

/// [chars]からランダムにキーを生成する
///
/// # Arguments
/// * `chars` - 文字のリスト
/// * `rng` - 乱数生成器
/// * `generator` - キーを生成する関数
///
/// # Returns
/// ランダムに選択され、generatorで生成されたキー
fn get_key(
    chars: &mut Vec<char>,
    rng: &mut StdRng,
    generator: fn(char, Option<char>) -> Option<Key>,
) -> Option<Key> {
    if chars.is_empty() {
        panic!("Invalid sequence")
    }

    let mut key: Option<Key> = None;
    while key.is_none() {
        let (c, idx) = peek_char(chars, rng);
        let shifted_char = peek_char_optional(chars, rng);
        key = generator(c, shifted_char.map(|v| v.0));

        if key.is_some() {
            chars.remove(idx);
            if let Some((_, idx)) = shifted_char {
                chars.remove(idx);
            }
        }
    }

    key
}

/// [chars]からランダムに一文字確認する
///
/// この処理では[chars]は変更されない。
///
/// # Arguments
/// * `chars` - 文字のリスト
/// * `rng` - 乱数生成器
///
/// # Returns
/// ランダムに選択された文字とそのindex
fn peek_char(chars: &Vec<char>, rng: &mut StdRng) -> (char, usize) {
    let idx = rng.gen_range(0..chars.len());
    let c = chars[idx];
    (c, idx)
}

/// [chars]からランダムに一文字確認する
///
/// この処理では[chars]は変更されない。
///
/// # Arguments
/// * `chars` - 文字のリスト
/// * `rng` - 乱数生成器
///
/// # Returns
/// ランダムに選択された文字とそのindex
fn peek_char_optional(chars: &Vec<char>, rng: &mut StdRng) -> Option<(char, usize)> {
    if !rng.gen::<bool>() || chars.is_empty() {
        None
    } else {
        Some(peek_char(chars, rng))
    }
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
        .map(|r| (0..10).map(move |c| (r as usize, c as usize)))
        .flatten()
        .collect();

    for exclude in EXCLUDE_MAP {
        indices.remove(&exclude);
    }

    indices
}

mod constraints {
    use crate::key::Key;

    use super::{
        LEFT_SEMITURBID_INDEX, LEFT_SHIFT_INDEX, LEFT_TURBID_INDEX, RIGHT_SEMITURBID_INDEX,
        RIGHT_SHIFT_INDEX, RIGHT_TURBID_INDEX,
    };

    /// 左右のシフトキーが同一のキーを指しているかどうかを確認する
    pub(super) fn should_shift_having_same_key(layout: &Vec<Vec<Key>>) -> bool {
        let left_shifted = layout[LEFT_SHIFT_INDEX.0][LEFT_SHIFT_INDEX.1].shifted();
        let right_shifted = layout[RIGHT_SHIFT_INDEX.0][RIGHT_SHIFT_INDEX.1].shifted();

        left_shifted == right_shifted
    }

    /// 左右の濁音シフト間では、いずれかのキーにしか濁音が設定されていないかどうかを確認する
    pub(super) fn should_only_one_turbid(layout: &Vec<Vec<Key>>) -> bool {
        let left_turbid = layout[LEFT_TURBID_INDEX.0][LEFT_TURBID_INDEX.1].turbid();
        let right_turbid = layout[RIGHT_TURBID_INDEX.0][RIGHT_TURBID_INDEX.1].turbid();

        match (left_turbid, right_turbid) {
            (Some(_), Some(_)) => false,
            _ => true,
        }
    }

    /// 左右の半濁音シフト間では、いずれかのキーにしか半濁音が設定されていないかどうかを確認する
    pub(super) fn should_only_one_semiturbit(layout: &Vec<Vec<Key>>) -> bool {
        let left_semiturbid = layout[LEFT_SEMITURBID_INDEX.0][LEFT_SEMITURBID_INDEX.1].semiturbid();
        let right_semiturbid =
            layout[RIGHT_SEMITURBID_INDEX.0][RIGHT_SEMITURBID_INDEX.1].semiturbid();

        match (left_semiturbid, right_semiturbid) {
            (Some(_), Some(_)) => false,
            _ => true,
        }
    }

    /// 左右の濁音・半濁音の間では、いずれかのキーにしか濁音と半濁音が設定されていないかどうかを確認する
    pub(super) fn should_be_explicit_between_left_turbid_and_right_semiturbit(
        layout: &Vec<Vec<Key>>,
    ) -> bool {
        // 濁音と半濁音を同時に押下したとき、両方に値が入っていると競合してしまうので、それを防ぐ
        let left_turbid = &layout[LEFT_TURBID_INDEX.0][LEFT_TURBID_INDEX.1];
        let right_semiturbid = &layout[RIGHT_SEMITURBID_INDEX.0][RIGHT_SEMITURBID_INDEX.1];

        match (
            left_turbid.turbid(),
            right_semiturbid.turbid(),
            left_turbid.semiturbid(),
            right_semiturbid.semiturbid(),
        ) {
            (Some(_), None, None, None) => true,
            (None, Some(_), None, None) => true,
            (None, None, Some(_), None) => true,
            (None, None, None, Some(_)) => true,
            _ => false,
        }
    }

    /// 左右の濁音・半濁音の間では、いずれかのキーにしか濁音と半濁音が設定されていないかどうかを確認する
    pub(super) fn should_be_explicit_between_right_turbid_and_left_semiturbit(
        layout: &Vec<Vec<Key>>,
    ) -> bool {
        // 濁音と半濁音を同時に押下したとき、両方に値が入っていると競合してしまうので、それを防ぐ
        let right_turbid = &layout[RIGHT_TURBID_INDEX.0][RIGHT_TURBID_INDEX.1];
        let left_semiturbid = &layout[LEFT_SEMITURBID_INDEX.0][LEFT_SEMITURBID_INDEX.1];

        match (
            right_turbid.turbid(),
            left_semiturbid.turbid(),
            right_turbid.semiturbid(),
            left_semiturbid.semiturbid(),
        ) {
            (Some(_), None, None, None) => true,
            (None, Some(_), None, None) => true,
            (None, None, Some(_), None) => true,
            (None, None, None, Some(_)) => true,
            _ => false,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn empty_layout() -> Vec<Vec<Key>> {
            vec![vec![Key::empty(); 10]; 3]
        }

        fn put_key(layout: &mut Vec<Vec<Key>>, key: Key, pos: (usize, usize)) {
            layout[pos.0][pos.1] = key;
        }

        #[test]
        fn having_same_key_between_shift() {
            // arrange
            let mut layout = empty_layout();
            put_key(
                &mut layout,
                Key::new_shift('あ', Some('を')).unwrap(),
                LEFT_SHIFT_INDEX,
            );
            put_key(
                &mut layout,
                Key::new_shift('か', Some('を')).unwrap(),
                RIGHT_SHIFT_INDEX,
            );

            // act
            let ret = should_shift_having_same_key(&layout);

            // assert
            assert!(ret, "should be valid")
        }

        #[test]
        fn not_having_same_key_between_shift() {
            // arrange
            let mut layout = empty_layout();
            put_key(
                &mut layout,
                Key::new_shift('あ', Some('ぬ')).unwrap(),
                LEFT_SHIFT_INDEX,
            );
            put_key(
                &mut layout,
                Key::new_shift('か', Some('を')).unwrap(),
                RIGHT_SHIFT_INDEX,
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
            put_key(
                &mut layout,
                Key::new_turbid('あ', Some('ぬ')).unwrap(),
                LEFT_TURBID_INDEX,
            );
            put_key(
                &mut layout,
                Key::new_turbid('か', Some('を')).unwrap(),
                RIGHT_TURBID_INDEX,
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
            put_key(
                &mut layout,
                Key::new_turbid('し', Some('ぬ')).unwrap(),
                LEFT_TURBID_INDEX,
            );
            put_key(
                &mut layout,
                Key::new_turbid('か', Some('を')).unwrap(),
                RIGHT_TURBID_INDEX,
            );

            // act
            let ret = should_only_one_turbid(&layout);

            // assert
            assert!(!ret, "should be valid")
        }

        #[test]
        fn only_one_semiturbid_between_semiturbid_keys() {
            // arrange
            let mut layout = empty_layout();
            put_key(
                &mut layout,
                Key::new_semiturbid('あ', Some('ぬ')).unwrap(),
                LEFT_SEMITURBID_INDEX,
            );
            put_key(
                &mut layout,
                Key::new_semiturbid('か', Some('を')).unwrap(),
                RIGHT_SEMITURBID_INDEX,
            );

            // act
            let ret = should_only_one_semiturbit(&layout);

            // assert
            assert!(ret, "should be valid")
        }

        #[test]
        fn two_semiturbid_between_semiturbid_keys() {
            // arrange
            let mut layout = empty_layout();
            put_key(
                &mut layout,
                Key::new_semiturbid('は', Some('ぬ')).unwrap(),
                LEFT_SEMITURBID_INDEX,
            );
            put_key(
                &mut layout,
                Key::new_semiturbid('ひ', Some('を')).unwrap(),
                RIGHT_SEMITURBID_INDEX,
            );

            // act
            let ret = should_only_one_semiturbit(&layout);

            // assert
            assert!(!ret, "should be valid")
        }

        #[test]
        fn only_one_turbid_and_semiturbid_set_between_left_turbid_and_right_semiturbid() {
            // arrange
            let mut layout = empty_layout();
            put_key(
                &mut layout,
                Key::new_semiturbid('か', Some('ぬ')).unwrap(),
                LEFT_TURBID_INDEX,
            );
            put_key(
                &mut layout,
                Key::new_semiturbid('ま', Some('を')).unwrap(),
                RIGHT_SEMITURBID_INDEX,
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
            put_key(
                &mut layout,
                Key::new_semiturbid('か', Some('ぬ')).unwrap(),
                RIGHT_TURBID_INDEX,
            );
            put_key(
                &mut layout,
                Key::new_semiturbid('ま', Some('を')).unwrap(),
                LEFT_SEMITURBID_INDEX,
            );

            // act
            let ret = should_be_explicit_between_right_turbid_and_left_semiturbit(&layout);

            // assert
            assert!(ret, "should be valid")
        }
    }
}

impl Keymap {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// 指定されたseedを元にしてキーマップを生成する
    ///
    /// 生成されたkeymapは、あくまでランダムなキーマップであり、実際に利用するためには、[Keymap::meet_requirements]がtrueを返すことを前提としなければ
    /// ならない。
    pub fn generate(rng: &mut StdRng) -> Keymap {
        let mut layout = vec![vec![Key::empty(); 10]; 3];
        let mut assignable_chars = char_def::assignable_chars();
        // 対応するインデックスは最初になんとかしておく
        let mut indices = key_indices();
        indices.remove(&LEFT_SHIFT_INDEX);
        indices.remove(&RIGHT_SHIFT_INDEX);
        indices.remove(&LEFT_TURBID_INDEX);
        indices.remove(&RIGHT_TURBID_INDEX);
        indices.remove(&LEFT_SEMITURBID_INDEX);
        indices.remove(&RIGHT_SEMITURBID_INDEX);

        // シフトの位置だけは固定しておくので、最初に生成しておく
        let shifted_char = pick_char(&mut assignable_chars, rng);

        // 左右シフトがshiftedになるケースは一通りしかないので、ここは常に同一になる
        let key = Key::new_shift(pick_char(&mut assignable_chars, rng), Some(shifted_char));
        layout[LEFT_SHIFT_INDEX.0][LEFT_SHIFT_INDEX.1] =
            key.expect("should be generate shift key in initial generation");
        let key = Key::new_shift(pick_char(&mut assignable_chars, rng), Some(shifted_char));
        layout[RIGHT_SHIFT_INDEX.0][RIGHT_SHIFT_INDEX.1] =
            key.expect("should be generate shift key in initial generation");

        layout[LEFT_TURBID_INDEX.0][LEFT_TURBID_INDEX.1] =
            get_key(&mut assignable_chars, rng, Key::new_turbid).expect("should be key");
        layout[RIGHT_TURBID_INDEX.0][RIGHT_TURBID_INDEX.1] =
            get_key(&mut assignable_chars, rng, Key::new_turbid).expect("should be key");
        layout[LEFT_SEMITURBID_INDEX.0][LEFT_SEMITURBID_INDEX.1] =
            get_key(&mut assignable_chars, rng, Key::new_semiturbid).expect("should be key");
        layout[RIGHT_SEMITURBID_INDEX.0][RIGHT_SEMITURBID_INDEX.1] =
            get_key(&mut assignable_chars, rng, Key::new_semiturbid).expect("should be key");

        // 残りの場所に追加していく。基本的に単打はふやすべきではあるので、一旦単打だけ埋める
        for (r, c) in indices.iter() {
            let char = pick_char(&mut assignable_chars, rng);

            layout[*r][*c] = Key::new_normal(char, None).unwrap();
        }

        // 2週目で、入るところから入れていく
        while !assignable_chars.is_empty() {
            for (r, c) in indices.iter() {
                if assignable_chars.is_empty() {
                    break;
                }

                let current = &layout[*r][*c];
                if current.shifted().is_some() {
                    continue;
                }
                let char = pick_char(&mut assignable_chars, rng);

                if let Some(key) = Key::new_normal(current.unshifted(), Some(char)) {
                    layout[*r][*c] = key;
                }
            }
        }

        if !assignable_chars.is_empty() {
            panic!("Leave some chars: {:?}", assignable_chars)
        }

        Keymap {
            generation: 1,
            layout,
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
            constraints::should_be_explicit_between_left_turbid_and_right_semiturbit,
            constraints::should_only_one_turbid,
            constraints::should_only_one_semiturbit,
            constraints::should_be_explicit_between_right_turbid_and_left_semiturbit,
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

    /// keymapに対して操作を実行して、実行した結果のkeymapを返す
    pub fn mutate(&self, rng: &mut StdRng) -> Keymap {
        let mut keymap = self.clone();

        // いくつか定義されている処理をランダムに実行する
        let operation: i32 = rng.gen_range(0..3);

        match operation {
            0 => keymap.swap_unshifted_between_keys(rng),
            1 => keymap.swap_shifted_between_keys(rng),
            2 => keymap.flip_key(rng),
            _ => panic!("invalid case"),
        }

        keymap.generation += 1;
        keymap
    }

    /// 任意のkeyにおけるunshiftedを交換する。
    fn swap_unshifted_between_keys(&mut self, rng: &mut StdRng) {
        let layout = Vec::from_iter(key_indices().into_iter());

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
        let layout = Vec::from_iter(key_indices().into_iter());

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

    /// 任意のkeyにおける無シフト面とシフト面を交換する。
    ///
    /// # Arguments
    /// * `rng` - 乱数生成機
    ///
    /// # Return
    /// 交換されたkeymap
    fn flip_key(&mut self, rng: &mut StdRng) {
        let layout = Vec::from_iter(key_indices().into_iter());

        let idx = rng.gen_range(0..layout.len());
        let pos = layout[idx];

        self.layout[pos.0][pos.1] = self.layout[pos.0][pos.1].flip();
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
