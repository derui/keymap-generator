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

    /// linear layoutにおける、特殊キーのindex
    pub fn indices_of_special_keys() -> Vec<usize> {
        vec![
            LINEAR_L_SHIFT_INDEX,
            LINEAR_R_SHIFT_INDEX,
            LINEAR_L_TURBID_INDEX,
            LINEAR_R_TURBID_INDEX,
            LINEAR_L_SEMITURBID_INDEX,
            LINEAR_R_SEMITURBID_INDEX,
        ]
    }

    /// linear layoutにおける、濁音と半濁音に関連するキーのindex
    pub fn indices_of_turbid_related_keys() -> Vec<usize> {
        vec![
            LINEAR_L_TURBID_INDEX,
            LINEAR_R_TURBID_INDEX,
            LINEAR_L_SEMITURBID_INDEX,
            LINEAR_R_SEMITURBID_INDEX,
        ]
    }
}
