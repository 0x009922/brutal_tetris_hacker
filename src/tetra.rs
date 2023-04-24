use super::util::Pos;
use crate::util::Size;

use std::ops::Add;

#[derive(Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Tetra {
    positions: [Pos; 4],
    size: Size,
    col_shift: usize,
}

const TETRAS_COUNT: usize = 19;

const fn const_tetra(positions: [(usize, usize); 4], col_shift: usize) -> Tetra {
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

    Tetra {
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

macro_rules! tetra {
    ($a:expr, $b:expr, $c:expr, $d:expr, $shift:expr) => {
        const_tetra([$a, $b, $c, $d], $shift)
    };
}

const TETRAS: [Tetra; TETRAS_COUNT] = [
    tetra!((0, 0), (0, 1), (1, 0), (1, 1), 0),
    tetra!((0, 0), (0, 1), (0, 2), (0, 3), 0),
    tetra!((0, 0), (1, 0), (2, 0), (3, 0), 0),
    tetra!((0, 0), (0, 1), (0, 2), (1, 1), 0),
    tetra!((0, 0), (1, 0), (1, 1), (2, 0), 0),
    tetra!((0, 1), (1, 0), (1, 1), (1, 2), 1),
    tetra!((0, 1), (1, 0), (1, 1), (2, 1), 1),
    tetra!((0, 0), (0, 1), (0, 2), (1, 0), 0),
    tetra!((0, 0), (1, 0), (2, 0), (2, 1), 0),
    tetra!((0, 2), (1, 0), (1, 1), (1, 2), 2),
    tetra!((0, 0), (0, 1), (1, 1), (2, 1), 0),
    tetra!((0, 0), (0, 1), (0, 2), (1, 2), 0),
    tetra!((0, 1), (1, 1), (2, 0), (2, 1), 1),
    tetra!((0, 0), (1, 0), (1, 1), (1, 2), 0),
    tetra!((0, 0), (0, 1), (1, 0), (2, 0), 0),
    tetra!((0, 1), (0, 2), (1, 0), (1, 1), 1),
    tetra!((0, 0), (1, 0), (1, 1), (2, 1), 0),
    tetra!((0, 0), (0, 1), (1, 1), (1, 2), 0),
    tetra!((0, 1), (1, 0), (1, 1), (2, 0), 1),
];

#[cfg(test)]
pub const I_HORIZONTAL: &Tetra = &TETRAS[1];
#[cfg(test)]
pub const T_LOOK_LEFT: &Tetra = &TETRAS[6];

impl Tetra {
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

impl IntoIterator for Tetra {
    type Item = Pos;
    type IntoIter = core::array::IntoIter<Self::Item, 4>;

    fn into_iter(self) -> Self::IntoIter {
        self.positions.into_iter()
    }
}

#[derive(Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Placed {
    pub tetra: &'static Tetra,
    pub position: Pos,
}

impl Placed {
    pub fn new(tetra: &'static Tetra, position: Pos) -> Self {
        Self { tetra, position }
    }
}

#[derive(Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct PlacedBoundariesChecked(Placed);

impl PlacedBoundariesChecked {
    pub fn in_boundaries(placed: Placed, boundaries: Size) -> Option<Self> {
        let Placed { tetra, position } = placed;

        let tetra_size = tetra.size();
        let tetra_col_shift = *tetra.col_shift();

        if tetra_col_shift > position.col
            || position.col + tetra_size.cols - tetra_col_shift > boundaries.cols
            || position.row + tetra_size.rows > boundaries.rows
        {
            return None;
        }

        Some(Self(placed))
    }

    pub fn iter_relative_to_place(&self) -> impl Iterator<Item = Pos> + '_ {
        let Self(Placed {
            tetra,
            position: relative,
        }) = self;

        tetra.iter().map(|pos| {
            let mut pos = pos.add(relative);
            pos.col -= tetra.col_shift;
            pos
        })
    }
}

impl From<PlacedBoundariesChecked> for Placed {
    fn from(value: PlacedBoundariesChecked) -> Self {
        value.0
    }
}

/// Yields finite shuffled tetra iterators.
///
/// ```
/// let mut generator = RandomTetras::new();
/// let mut tetras = generator.finite_iter();
/// assert!(matches!(tetras.next(), Some(Tetra { .. })));
/// ```
#[derive(Debug)]
pub struct Shuffler {
    rng: rand::rngs::ThreadRng,
}

impl Shuffler {
    pub fn new() -> Self {
        let rng = rand::thread_rng();
        Self { rng }
    }

    pub fn finite_iter(&mut self) -> impl Iterator<Item = &'static Tetra> {
        use rand::Rng;

        let tetras = array_macro::array![_ => self.rng.gen_range(0..TETRAS_COUNT); TETRAS_COUNT];
        tetras.into_iter().map(|idx| &TETRAS[idx])
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_for_3x3() {
        assert!(matches!(
            PlacedBoundariesChecked::in_boundaries(
                Placed::new(I_HORIZONTAL, Pos::new(0, 0)),
                Size::new(3, 3)
            ),
            None
        ));
    }

    #[test]
    fn check_for_horizontal_i_in_4x4() {
        assert!(matches!(
            PlacedBoundariesChecked::in_boundaries(
                Placed::new(I_HORIZONTAL, Pos::new(0, 0)),
                Size::new(4, 4)
            ),
            Some(_)
        ));
    }

    #[test]
    fn check_horizontal_i_in_4x4_at_col_1() {
        assert!(matches!(
            PlacedBoundariesChecked::in_boundaries(
                Placed::new(I_HORIZONTAL, Pos::new(0, 1)),
                Size::new(4, 4)
            ),
            None
        ));
    }

    #[test]
    fn checj_t_at_right_border() {
        assert!(matches!(
            PlacedBoundariesChecked::in_boundaries(
                Placed::new(T_LOOK_LEFT, Pos::new(0, 2)),
                Size::new(3, 3)
            ),
            Some(_)
        ));
    }
}
