use std::collections::HashSet;

use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{char_def, key::Key};

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

/// [chars]からランダムに一文字取得する。ただし、50/50の確率で取得しない場合がある
///
/// # Arguments
/// * `chars` - 文字のリスト
/// * `rng` - 乱数生成器
///
/// # Returns
/// ランダムに選択された文字。取得しない場合はNone
fn pick_char_optional(chars: &mut Vec<char>, rng: &mut StdRng) -> Option<char> {
    if !rng.gen::<bool>() || chars.is_empty() {
        None
    } else {
        Some(pick_char(chars, rng))
    }
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
const EXCLUDE_MAP: [(usize, usize); 4] = [(0, 0), (0, 4), (0, 5), (0, 9)];
// 左手シフトキーのindex
const LEFT_SHIFT_INDEX: (usize, usize) = (1, 2);
// 右手シフトキーのindex
const RIGHT_SHIFT_INDEX: (usize, usize) = (1, 7);
// 左手濁音シフトキーのindex
const LEFT_TURBID_INDEX: (usize, usize) = (1, 3);
// 右手濁音シフトキーのindex
const RIGHT_TURBID_INDEX: (usize, usize) = (1, 6);
// 左手半濁音シフトキーのindex
const LEFT_SEMITURBID_INDEX: (usize, usize) = (2, 3);
// 右手半濁音シフトキーのindex
const RIGHT_SEMITURBID_INDEX: (usize, usize) = (2, 6);

impl Keymap {
    /// 指定されたseedを元にしてキーマップを生成する
    ///
    /// 生成されたkeymapは、あくまでランダムなキーマップであり、実際に利用するためには、[Keymap::meet_requirements]がtrueを返すことを前提としなければ
    /// ならない。
    fn generate(rng: &mut StdRng) -> Keymap {
        let mut layout = vec![vec![Key::empty(); 10]; 3];
        let mut assignable_chars = char_def::assignable_chars();
        // 対応するインデックスは最初になんとかしておく
        let mut indices: HashSet<(usize, usize)> = (0..3)
            .map(|r| (0..10).map(move |c| (r as usize, c as usize)))
            .flatten()
            .collect();
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

        // 残りの場所に追加していく

        for (r, c) in indices {
            let char = pick_char(&mut assignable_chars, rng);

            let mut key: Option<Key> = None;
            while key.is_none() {
                key = Key::new_normal(char, pick_char_optional(&mut assignable_chars, rng));
            }

            layout[r][c] = key.expect("should be key");
        }

        if !assignable_chars.is_empty() {
            panic!("Leave some chars: {:?}", assignable_chars)
        }

        Keymap {
            generation: 0,
            layout,
        }
    }

    /// keymapに対して操作を実行して、実行した結果のkeymapを返す
    pub fn advance_generation(&self, rng: &mut StdRng) -> Keymap {
        let mut keymap = self.clone();

        // いくつか定義されている処理をランダムに実行する
        let operation: i32 = (rng.gen_range(0..3));

        match operation {
            0 => keymap.swap_unshifted_between_keys(rng),
            1 => keymap.swap_shifted_between_keys(rng),
            2 => keymap.flip_key(rng),
            _ => panic!("invalid case"),
        }

        keymap
    }

    /// 任意のkeyにおけるunshiftedを交換する。ただし、交換後のkeyで
    ///
    ///
    fn swap_unshifted_between_keys(&mut self, rng: &mut StdRng) {}

    /// 任意のkeyにおけるshiftedを交換する。ただし、交換後のkeyで
    ///
    ///
    fn swap_shifted_between_keys(&mut self, rng: &mut StdRng) {}

    /// 任意のkeyにおける無シフト面とシフト面を交換する。
    ///
    /// # Arguments
    /// * `rng` - 乱数生成機
    ///
    /// # Return
    /// 交換されたkeymap
    fn flip_key(&mut self, rng: &mut StdRng) {}
}
