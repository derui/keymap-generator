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

/// 直線的なレイアウトを表す。ここでのレイアウトは、あくまでも通常のキー配置との対応関係のみを管理しており、
/// 割当などは対応外である。
pub mod linear {
    use std::collections::HashMap;

    use super::Point;

    const LINEAR_LAYOUT: [Point; 26] = [
        // Point { row: 0, col: 0 },
        Point { row: 0, col: 1 },
        Point { row: 0, col: 2 },
        Point { row: 0, col: 3 },
        // Point { row: 0, col: 4 },
        // Point { row: 0, col: 5 },
        Point { row: 0, col: 6 },
        Point { row: 0, col: 7 },
        Point { row: 0, col: 8 },
        // Point { row: 0, col: 9 },
        Point { row: 1, col: 0 },
        Point { row: 1, col: 1 },
        Point { row: 1, col: 2 },
        Point { row: 1, col: 3 },
        Point { row: 1, col: 4 },
        Point { row: 1, col: 5 },
        Point { row: 1, col: 6 },
        Point { row: 1, col: 7 },
        Point { row: 1, col: 8 },
        Point { row: 1, col: 9 },
        Point { row: 2, col: 0 },
        Point { row: 2, col: 1 },
        Point { row: 2, col: 2 },
        Point { row: 2, col: 3 },
        Point { row: 2, col: 4 },
        Point { row: 2, col: 5 },
        Point { row: 2, col: 6 },
        Point { row: 2, col: 7 },
        Point { row: 2, col: 8 },
        Point { row: 2, col: 9 },
    ];

    const LINEAR_MAPPING: [(char, Point); 26] = [
        // Point { row: 0, col: 0 },
        ('w', Point { row: 0, col: 1 }),
        ('e', Point { row: 0, col: 2 }),
        ('r', Point { row: 0, col: 3 }),
        // Point { row: 0, col: 4 },
        // Point { row: 0, col: 5 },
        ('u', Point { row: 0, col: 6 }),
        ('i', Point { row: 0, col: 7 }),
        ('o', Point { row: 0, col: 8 }),
        // Point { row: 0, col: 9 },
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
        ('b', Point { row: 2, col: 4 }),
        ('n', Point { row: 2, col: 5 }),
        ('m', Point { row: 2, col: 6 }),
        (',', Point { row: 2, col: 7 }),
        ('.', Point { row: 2, col: 8 }),
        ('/', Point { row: 2, col: 9 }),
    ];

    /// 各特殊キーの位置
    pub const LINEAR_L_SHIFT_INDEX: usize = 8;
    pub const LINEAR_R_SHIFT_INDEX: usize = 13;
    pub const LINEAR_L_TURBID_INDEX: usize = 9;
    pub const LINEAR_R_TURBID_INDEX: usize = 12;
    pub const LINEAR_L_SEMITURBID_INDEX: usize = 19;
    pub const LINEAR_R_SEMITURBID_INDEX: usize = 22;

    /// 直線的になるレイアウトを返す
    pub fn linear_layout() -> &'static [Point] {
        &LINEAR_LAYOUT
    }

    /// 直線的になるレイアウトと、QWERTYにおいて対応する文字のmappingを返す
    pub fn linear_mapping() -> HashMap<char, Point> {
        LINEAR_MAPPING.iter().cloned().collect()
    }
}
