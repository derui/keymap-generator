/// layoutにおける位置を表す
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point(usize, usize);

impl Point {
    pub fn new(row: usize, col: usize) -> Self {
        Point(row, col)
    }

    #[inline]
    pub fn row(&self) -> usize {
        self.0
    }

    #[inline]
    pub fn col(&self) -> usize {
        self.1
    }
}

impl From<Point> for (usize, usize) {
    fn from(value: Point) -> Self {
        (value.0, value.1)
    }
}

impl From<&Point> for (usize, usize) {
    fn from(value: &Point) -> Self {
        (value.0, value.1)
    }
}

impl From<(usize, usize)> for Point {
    fn from(value: (usize, usize)) -> Self {
        Point(value.0, value.1)
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
        ('w', Point(0, 1)),
        ('e', Point(0, 2)),
        ('r', Point(0, 3)),
        // ('t', Point { row: 0, col: 4 }),
        // ('y', Point { row: 0, col: 5 }),
        ('u', Point(0, 6)),
        ('i', Point(0, 7)),
        ('o', Point(0, 8)),
        // ('p', Point { row: 0, col: 8 }),
        ('a', Point(1, 0)),
        ('s', Point(1, 1)),
        ('d', Point(1, 2)),
        ('f', Point(1, 3)),
        ('g', Point(1, 4)),
        ('h', Point(1, 5)),
        ('j', Point(1, 6)),
        ('k', Point(1, 7)),
        ('l', Point(1, 8)),
        (';', Point(1, 9)),
        ('z', Point(2, 0)),
        ('x', Point(2, 1)),
        ('c', Point(2, 2)),
        ('v', Point(2, 3)),
        ('b', Point(2, 4)),
        ('n', Point(2, 5)),
        ('m', Point(2, 6)),
        (',', Point(2, 7)),
        ('.', Point(2, 8)),
        ('/', Point(2, 9)),
    ];

    /// 各特殊キーの位置
    pub const LINEAR_L_SHIFT_INDEX: usize = 8;
    pub const LINEAR_R_SHIFT_INDEX: usize = 13;
    pub const LINEAR_L_TURBID_INDEX: usize = 9;
    pub const LINEAR_R_TURBID_INDEX: usize = 12;
    pub const LINEAR_L_SEMITURBID_INDEX: usize = 19;
    pub const LINEAR_R_SEMITURBID_INDEX: usize = 22;

    /// 読点の位置
    pub fn reading_point_points() -> Vec<Point> {
        vec![Point(1, 6), Point(1, 7)]
    }

    /// 句点の位置
    pub fn punctuation_mark_points() -> Vec<Point> {
        vec![Point(1, 2), Point(1, 3)]
    }

    /// ゔの位置
    pub fn turbid_u_point() -> Point {
        Point(0, 0)
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
        if point.col() <= 4 {
            Hand::Left
        } else {
            Hand::Right
        }
    }

    pub fn get_left_small_shifter() -> Point {
        Point(0, 0)
    }

    pub fn get_right_small_shifter() -> Point {
        Point(0, 9)
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
        let ret = get_hand_of_point(&Point(1, 3));

        // assert
        assert_eq!(ret, Hand::Left);
    }

    #[test]
    fn get_right_hand() {
        // arrange

        // act
        let ret = get_hand_of_point(&Point(1, 5));

        // assert
        assert_eq!(ret, Hand::Right);
    }

    #[test]
    fn char_of_point() {
        // arrange

        // act
        let ret = get_char_of_point(&Point(1, 5));

        // assert
        assert_eq!(ret, 'h');
    }
}
