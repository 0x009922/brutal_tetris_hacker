use super::util::Pos;
use crate::util::Size;
use std::ops::Add;

#[derive(Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Tetro {
    positions: [Pos; 4],
    size: Size,
    col_shift: usize,
}

const TETRO_COUNT: usize = 19;

const fn const_tetro(positions: [(usize, usize); 4], col_shift: usize) -> Tetro {
    const fn transform_pos((row, col): (usize, usize)) -> Pos {
        Pos { row, col }
    }

    const fn max_const(a: usize, b: usize) -> usize {
        if a > b {
            a
        } else {
            b
        }
    }

    const fn max_const_tuple(size: (usize, usize), pos: (usize, usize)) -> (usize, usize) {
        (max_const(size.0, pos.0 + 1), max_const(size.1, pos.1 + 1))
    }

    let size = {
        let mut size = (1, 1);

        size = max_const_tuple(size, positions[0]);
        size = max_const_tuple(size, positions[1]);
        size = max_const_tuple(size, positions[2]);
        size = max_const_tuple(size, positions[3]);

        size
    };

    Tetro {
        positions: [
            transform_pos(positions[0]),
            transform_pos(positions[1]),
            transform_pos(positions[2]),
            transform_pos(positions[3]),
        ],
        size: Size::new(size.0, size.1),
        col_shift,
    }
}

macro_rules! tetro {
    ($a:expr, $b:expr, $c:expr, $d:expr, $shift:expr) => {
        const_tetro([$a, $b, $c, $d], $shift)
    };
}

const TETROS: [Tetro; TETRO_COUNT] = [
    tetro!((0, 0), (0, 1), (1, 0), (1, 1), 0),
    tetro!((0, 0), (0, 1), (0, 2), (0, 3), 0),
    tetro!((0, 0), (1, 0), (2, 0), (3, 0), 0),
    tetro!((0, 0), (0, 1), (0, 2), (1, 1), 0),
    tetro!((0, 0), (1, 0), (1, 1), (2, 0), 0),
    tetro!((0, 1), (1, 0), (1, 1), (1, 2), 1),
    tetro!((0, 1), (1, 0), (1, 1), (2, 1), 1),
    tetro!((0, 0), (0, 1), (0, 2), (1, 0), 0),
    tetro!((0, 0), (1, 0), (2, 0), (2, 1), 0),
    tetro!((0, 2), (1, 0), (1, 1), (1, 2), 2),
    tetro!((0, 0), (0, 1), (1, 1), (2, 1), 0),
    tetro!((0, 0), (0, 1), (0, 2), (1, 2), 0),
    tetro!((0, 1), (1, 1), (2, 0), (2, 1), 1),
    tetro!((0, 0), (1, 0), (1, 1), (1, 2), 0),
    tetro!((0, 0), (0, 1), (1, 0), (2, 0), 0),
    tetro!((0, 1), (0, 2), (1, 0), (1, 1), 1),
    tetro!((0, 0), (1, 0), (1, 1), (2, 1), 0),
    tetro!((0, 0), (0, 1), (1, 1), (1, 2), 0),
    tetro!((0, 1), (1, 0), (1, 1), (2, 0), 1),
];

#[cfg(test)]
pub const I_HORIZONTAL: &Tetro = &TETROS[1];
#[cfg(test)]
pub const T_LOOK_LEFT: &Tetro = &TETROS[6];

impl Tetro {
    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn iter(&self) -> impl Iterator<Item = &Pos> {
        self.positions.iter()
    }

    pub fn col_shift(&self) -> &usize {
        &self.col_shift
    }
}

impl IntoIterator for Tetro {
    type Item = Pos;
    type IntoIter = core::array::IntoIter<Self::Item, 4>;

    fn into_iter(self) -> Self::IntoIter {
        self.positions.into_iter()
    }
}

#[derive(Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct PlacedTetro {
    pub tetro: &'static Tetro,
    pub position: Pos,
}

impl PlacedTetro {
    pub fn new(tetro: &'static Tetro, position: Pos) -> Self {
        Self { tetro, position }
    }
}

pub fn static_tetros_iter() -> impl Iterator<Item = &'static Tetro> {
    TETROS.iter()
}

#[derive(Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct PlacedTetroInBoundaries(PlacedTetro);

impl PlacedTetroInBoundaries {
    pub fn in_boundaries(placed: PlacedTetro, boundaries: Size) -> Option<Self> {
        let PlacedTetro { tetro, position } = placed;

        let tetro_size = tetro.size();
        let tetro_col_shift = *tetro.col_shift();

        if tetro_col_shift > position.col
            || position.col + tetro_size.cols - tetro_col_shift > boundaries.cols
            || position.row + tetro_size.rows > boundaries.rows
        {
            return None;
        }

        Some(Self(placed))
    }

    pub fn iter_relative_to_place<'a>(&'a self) -> impl Iterator<Item = Pos> + 'a {
        let Self(PlacedTetro {
            tetro,
            position: relative,
        }) = self;

        tetro.iter().map(|pos| {
            let mut pos = pos.add(relative);
            pos.col -= tetro.col_shift;
            pos
        })
    }
}

impl From<PlacedTetroInBoundaries> for PlacedTetro {
    fn from(value: PlacedTetroInBoundaries) -> Self {
        value.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_for_3x3() {
        assert!(matches!(
            PlacedTetroInBoundaries::in_boundaries(
                PlacedTetro::new(I_HORIZONTAL, Pos::new(0, 0)),
                Size::new(3, 3)
            ),
            None
        ));
    }

    #[test]
    fn check_for_horizontal_i_in_4x4() {
        assert!(matches!(
            PlacedTetroInBoundaries::in_boundaries(
                PlacedTetro::new(I_HORIZONTAL, Pos::new(0, 0)),
                Size::new(4, 4)
            ),
            Some(_)
        ));
    }

    #[test]
    fn check_horizontal_i_in_4x4_at_col_1() {
        assert!(matches!(
            PlacedTetroInBoundaries::in_boundaries(
                PlacedTetro::new(I_HORIZONTAL, Pos::new(0, 1)),
                Size::new(4, 4)
            ),
            None
        ));
    }

    #[test]
    fn checj_t_at_right_border() {
        assert!(matches!(
            PlacedTetroInBoundaries::in_boundaries(
                PlacedTetro::new(T_LOOK_LEFT, Pos::new(0, 2)),
                Size::new(3, 3)
            ),
            Some(_)
        ));
    }
}
