use std::collections::HashSet;

use grid::Grid;

use crate::tetra::{static_tetros_iter, PlacedTetro, PlacedTetroInBoundaries, Tetro};
use crate::util::{Pos, PosInGrid, Size, SizeOf};

pub type Placement = Vec<PlacedTetroInBoundaries>;

pub struct Configuration {
    pub size: Size,
    pub unavailable: HashSet<Pos>,
}

impl Configuration {
    pub fn new(size: Size, unavailable: HashSet<Pos>) -> Self {
        Self { size, unavailable }
    }

    pub fn brute_force<S>(&self, stats: &'_ mut S) -> Vec<PlacementResult>
    where
        S: CollectStats,
    {
        RecursionState::brute_force(self, stats)
    }
}

#[derive(Clone, Copy, derive_more::Display, Debug)]
pub enum Cell {
    #[display(fmt = "-")]
    Empty,
    #[display(fmt = "x")]
    Unavailable,
    #[display(fmt = "+")]
    Occupied,
}

struct RecursionState<'a, S>
where
    S: CollectStats,
{
    grid: Grid<Cell>,
    how_many_free: usize,
    min_free_cells: usize,
    stack: Vec<PlacedTetroInBoundaries>,
    results: Vec<PlacementResult>,
    positions_for_lookup: Vec<Pos>,
    stats: &'a mut S,
}

const CACHE_EACH_TETROS: usize = 4;

impl<'a, S> RecursionState<'a, S>
where
    S: CollectStats,
{
    fn brute_force(
        Configuration { size, unavailable }: &Configuration,
        stats: &'a mut S,
    ) -> Vec<PlacementResult> {
        let mut state = RecursionState::new(*size, unavailable, stats);
        state.recursion();
        state.results
    }

    fn new(Size { rows, cols }: Size, unavailable: &'_ HashSet<Pos>, stats: &'a mut S) -> Self {
        let mut grid = Grid::init(rows, cols, Cell::Empty);
        let mut how_many_free = rows * cols;
        let mut initially_unavailable = 0;
        for Pos { row, col } in unavailable.iter() {
            grid[*row][*col] = Cell::Unavailable;
            initially_unavailable += 1;
            how_many_free -= 1;
        }
        let min_free_cells = (cols * rows - initially_unavailable) % 4;

        let stack = Vec::with_capacity(cols * rows);
        let results = Vec::new();

        // initially - all positions except unavailable
        let mut iter_positions = Vec::with_capacity(cols * rows);
        for row in 0..rows {
            for col in 0..cols {
                if let Cell::Empty = grid[row][col] {
                    iter_positions.push(Pos { row, col })
                }
            }
        }

        Self {
            grid,
            how_many_free,
            min_free_cells,

            stack,
            results,
            stats,

            positions_for_lookup: iter_positions,
        }
    }

    fn recursion(&mut self) {
        self.stats.recursions_inc();

        if self.how_many_free == self.min_free_cells {
            // self.results.push(PlacementResult {
            //     placement: self.stack.clone(),
            //     free: 0,
            // });
            self.stats.results_inc();
        }

        for tetro in static_tetros_iter() {
            if let Some(tetro_in_boundaries) = self.find_any_fit_for(tetro) {
                self.fill_and_push(tetro_in_boundaries);
                self.recursion();
                self.pop_and_clear();
            }
        }
    }

    fn fill_and_push(&mut self, tetro: PlacedTetroInBoundaries) {
        for i in tetro.iter_relative_to_place() {
            self.grid[i.row][i.col] = Cell::Occupied;
            self.how_many_free -= 1;
        }
        self.stack.push(tetro);
        self.update_lookup_cache();
    }

    fn pop_and_clear(&mut self) {
        let placed_tetro = self.stack.pop().unwrap();
        for i in placed_tetro.iter_relative_to_place() {
            self.grid[i.row][i.col] = Cell::Empty;
            self.how_many_free += 1;
        }
        self.update_lookup_cache();
    }

    fn find_any_fit_for(&self, tetro: &'static Tetro) -> Option<PlacedTetroInBoundaries> {
        self.positions_for_lookup
            .iter()
            .map(|pos| {
                PlacedTetroInBoundaries::in_boundaries(
                    PlacedTetro::new(tetro, *pos),
                    self.grid.size_of(),
                )
            })
            .find(|in_boundaries| {
                if let Some(in_boundaries) = in_boundaries {
                    let all_empty = in_boundaries
                        .iter_relative_to_place()
                        .all(|pos| matches!(self.grid.pos(&pos), Cell::Empty));
                    return all_empty;
                }
                false
            })
            .flatten()
    }

    /// should be called after pushing and popping a tetro
    fn update_lookup_cache(&mut self) {
        let stack_size = self.stack.len();

        let how_many_exclude = {
            if stack_size % CACHE_EACH_TETROS == 0 {
                Some(stack_size)
            } else if (stack_size + 1) % CACHE_EACH_TETROS == 0 {
                Some(stack_size - (stack_size % CACHE_EACH_TETROS))
            } else {
                None
            }
        };

        if let Some(mut excluded) = how_many_exclude {
            self.positions_for_lookup.clear();

            for row in 0..self.grid.rows() {
                for col in 0..self.grid.cols() {
                    if let Cell::Empty = self.grid[row][col] {
                        if excluded > 0 {
                            excluded -= 1
                        } else {
                            self.positions_for_lookup.push(Pos::new(row, col));
                        }
                    }
                }
            }
        }
    }
}

pub struct PlacementResult {
    pub placement: Placement,
    pub free: usize,
}

pub trait CollectStats {
    fn recursions_inc(&mut self);

    fn results_inc(&mut self);
}
