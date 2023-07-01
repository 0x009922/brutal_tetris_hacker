use std::fmt::{Debug, Formatter};
use std::ops::Add;

use derive_more::Display;
use grid::Grid;
use serde::Serialize;

#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Display, Serialize)]
#[display(fmt = "({row}, {col})")]
pub struct Pos {
    pub row: usize,
    pub col: usize,
}

impl Debug for Pos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}

impl From<(usize, usize)> for Pos {
    fn from(val: (usize, usize)) -> Self {
        Pos {
            row: val.0,
            col: val.1,
        }
    }
}

impl Add for &Pos {
    type Output = Pos;

    fn add(self, rhs: Self) -> Self::Output {
        Pos {
            row: self.row + rhs.row,
            col: self.col + rhs.col,
        }
    }
}

impl Pos {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

pub trait PosInGrid<T> {
    fn pos<'a>(&'a self, pos: &Pos) -> &'a T;
}

impl<T> PosInGrid<T> for Grid<T> {
    fn pos<'a>(&'a self, pos: &Pos) -> &'a T {
        &self[pos.row][pos.col]
    }
}

#[derive(Clone, Copy, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Size {
    pub rows: usize,
    pub cols: usize,
}

impl Size {
    pub const fn new(rows: usize, cols: usize) -> Self {
        Self { rows, cols }
    }
}

pub trait SizeOf {
    fn size_of(&self) -> Size;
}

impl<T> SizeOf for Grid<T> {
    fn size_of(&self) -> Size {
        Size::new(self.rows(), self.cols())
    }
}

impl From<(usize, usize)> for Size {
    fn from((rows, cols): (usize, usize)) -> Self {
        Self::new(rows, cols)
    }
}
