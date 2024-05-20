/// layoutにおける位置を表す
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    // 行
    row: usize,

    // 列
    col: usize,
}

impl From<Point> for (usize, usize) {
    fn from(value: Point) -> Self {
        (value.row, value.col)
    }
}

impl From<&Point> for (usize, usize) {
    fn from(value: &Point) -> Self {
        (value.row, value.col)
    }
}

impl From<(usize, usize)> for Point {
    fn from(value: (usize, usize)) -> Self {
        Point {
            row: value.0,
            col: value.1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    Right,
    Left,
}

/// 直線的なレイアウトを表す。ここでのレイアウトは、あくまでも通常のキー配置との対応関係のみを管理しており、
/// 割当などは対応外である。
pub mod linear {
    use std::collections::HashMap;

    use super::{Hand, Point};

    const LINEAR_MAPPING: [(char, Point); 26] = [
        // ('q', Point { row: 0, col: 0 }),
        ('w', Point { row: 0, col: 1 }),
        ('e', Point { row: 0, col: 2 }),
        ('r', Point { row: 0, col: 3 }),
        ('t', Point { row: 0, col: 4 }),
        ('y', Point { row: 0, col: 5 }),
        ('u', Point { row: 0, col: 6 }),
        ('i', Point { row: 0, col: 7 }),
        ('o', Point { row: 0, col: 8 }),
        // ('p', Point { row: 0, col: 8 }),
        ('a', Point { row: 1, col: 0 }),
        ('s', Point { row: 1, col: 1 }),
        ('d', Point { row: 1, col: 2 }),
        ('f', Point { row: 1, col: 3 }),
        ('g', Point { row: 1, col: 4 }),
        ('h', Point { row: 1, col: 5 }),
        ('j', Point { row: 1, col: 6 }),
        ('k', Point { row: 1, col: 7 }),
        ('l', Point { row: 1, col: 8 }),
        (';', Point { row: 1, col: 9 }),
        ('z', Point { row: 2, col: 0 }),
        ('x', Point { row: 2, col: 1 }),
        ('c', Point { row: 2, col: 2 }),
        ('v', Point { row: 2, col: 3 }),
        // ('b', Point { row: 2, col: 4 }),
        // ('n', Point { row: 2, col: 5 }),
        ('m', Point { row: 2, col: 6 }),
        (',', Point { row: 2, col: 7 }),
        ('.', Point { row: 2, col: 8 }),
        ('/', Point { row: 2, col: 9 }),
    ];

    /// 各特殊キーの位置
    pub const LINEAR_L_SHIFT_INDEX: usize = 10;
    pub const LINEAR_R_SHIFT_INDEX: usize = 15;
    pub const LINEAR_L_TURBID_INDEX: usize = 11;
    pub const LINEAR_R_TURBID_INDEX: usize = 14;
    pub const LINEAR_L_SEMITURBID_INDEX: usize = 21;
    pub const LINEAR_R_SEMITURBID_INDEX: usize = 22;

    /// 読点の位置
    pub fn reading_point_points() -> Vec<Point> {
        vec![Point { row: 1, col: 6 }, Point { row: 1, col: 7 }]
    }

    /// 句点の位置
    pub fn punctuation_mark_points() -> Vec<Point> {
        vec![Point { row: 1, col: 2 }, Point { row: 1, col: 3 }]
    }

    /// 直線的になるレイアウトを返す
    pub fn linear_layout() -> Vec<Point> {
        LINEAR_MAPPING
            .iter()
            .cloned()
            .map(|v| v.1)
            .collect::<Vec<_>>()
    }

    /// 直線的になるレイアウトと、QWERTYにおいて対応する文字のmappingを返す
    pub fn linear_mapping() -> HashMap<char, Point> {
        LINEAR_MAPPING.iter().cloned().collect()
    }

    /// layoutにおいて担当する手を返す
    pub fn get_hand_of_point(point: &Point) -> Hand {
        if point.col <= 4 {
            Hand::Left
        } else {
            Hand::Right
        }
    }

    pub fn get_left_small_shifter() -> Point {
        Point { row: 0, col: 0 }
    }

    pub fn get_right_small_shifter() -> Point {
        Point { row: 0, col: 9 }
    }

    /// layoutにおいて、指定された位置に対応する文字を返す
    pub fn get_char_of_point(point: &Point) -> char {
        if *point == get_left_small_shifter() {
            return 'q';
        }

        if *point == get_right_small_shifter() {
            return 'p';
        }

        LINEAR_MAPPING
            .iter()
            .find(|(_, p)| *p == *point)
            .map(|(c, _)| *c)
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use tests::linear::get_char_of_point;

    use self::linear::get_hand_of_point;

    use super::*;

    #[test]
    fn get_left_hand() {
        // arrange

        // act
        let ret = get_hand_of_point(&Point { row: 1, col: 3 });

        // assert
        assert_eq!(ret, Hand::Left);
    }

    #[test]
    fn get_right_hand() {
        // arrange

        // act
        let ret = get_hand_of_point(&Point { row: 1, col: 5 });

        // assert
        assert_eq!(ret, Hand::Right);
    }

    #[test]
    fn char_of_point() {
        // arrange

        // act
        let ret = get_char_of_point(&Point { row: 1, col: 5 });

        // assert
        assert_eq!(ret, 'h');
    }
}
