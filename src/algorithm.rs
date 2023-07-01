use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::ops::ControlFlow;

use grid::Grid;

use crate::tetra::{Placed, PlacedBoundariesChecked, Shuffler, Tetra};
use crate::util::{Pos, PosInGrid, Size, SizeOf};

pub type Placement = Vec<PlacedBoundariesChecked>;

pub struct Configuration {
    /// Size of the grid
    pub size: Size,
    /// What cells are unavailable to put Tetras into
    pub unavailable: HashSet<Pos>,
    /// How many results to generate
    pub results_limit: Option<NonZeroUsize>,
}

impl Configuration {
    pub fn new(size: Size, unavailable: HashSet<Pos>) -> Self {
        Self {
            size,
            unavailable,
            results_limit: None,
        }
    }

    pub fn with_results_limit(mut self, value: NonZeroUsize) -> Self {
        self.results_limit = Some(value);
        self
    }

    pub fn run<S>(&self, stats: &'_ mut S) -> Vec<PlacementResult>
    where
        S: CollectStats,
    {
        RecursionState::find_placements(self, stats)
    }
}

#[derive(Clone, Copy, derive_more::DebugCustom)]
pub enum Cell {
    #[debug(fmt = "-")]
    Empty,
    #[debug(fmt = "#")]
    Unavailable,
    #[debug(fmt = "+")]
    Occupied,
}

struct DebugGrid<'a>(&'a Grid<Cell>);

impl std::fmt::Debug for DebugGrid<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..self.0.rows() {
            for col in 0..self.0.cols() {
                self.0[row][col].fmt(f)?;
            }
            f.write_str("\n")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct RecursionState<'a, S>
where
    S: CollectStats,
{
    grid: Grid<Cell>,
    how_many_free: usize,
    stack: Vec<PlacedBoundariesChecked>,
    results: Vec<PlacementResult>,
    positions_for_lookup: Vec<Pos>,
    stats: &'a mut S,

    results_limit: Option<NonZeroUsize>,

    acceptance_threshold: usize,
    random_tetras: Shuffler,
}

impl<'a, S> RecursionState<'a, S>
where
    S: CollectStats,
{
    fn find_placements(cfg: &Configuration, stats: &'a mut S) -> Vec<PlacementResult> {
        let mut recursion = RecursionState::with_configuration(cfg, stats);
        recursion.run();
        recursion.results
    }

    fn with_configuration(
        Configuration {
            size,
            unavailable,
            results_limit,
        }: &Configuration,
        stats: &'a mut S,
    ) -> Self {
        let (rows, cols) = (size.rows, size.cols);

        let mut grid = Grid::init(rows, cols, Cell::Empty);
        let mut how_many_free = rows * cols;
        let mut initially_unavailable = 0;
        for Pos { row, col } in unavailable.iter() {
            grid[*row][*col] = Cell::Unavailable;
            initially_unavailable += 1;
            how_many_free -= 1;
        }
        let min_free_cells = (cols * rows - initially_unavailable) % 4;
        let acceptance_threshold =
            ((how_many_free - min_free_cells) as f64).powf(0.5).floor() as usize;

        let stack = Vec::with_capacity(cols * rows);
        let results = Vec::new();

        // all positions except unavailable
        let mut iter_positions = Vec::with_capacity(cols * rows);
        for row in 0..rows {
            for col in 0..cols {
                if let Cell::Empty = grid[row][col] {
                    iter_positions.push(Pos { row, col });
                }
            }
        }

        Self {
            grid,
            how_many_free,
            acceptance_threshold,

            stack,
            results,
            stats,

            positions_for_lookup: iter_positions,

            results_limit: *results_limit,
            random_tetras: Shuffler::new(),
        }
    }

    fn run(&mut self) -> ControlFlow<()> {
        self.stats.recursions_inc();

        let mut was_any_fit = false;

        for tetra in self.random_tetras.finite_iter() {
            if let Some(tetra_in_boundaries) = self.find_any_fit_for(tetra) {
                was_any_fit = true;
                self.fill_and_push(tetra_in_boundaries);
                if let ControlFlow::Break(_) = self.run() {
                    return ControlFlow::Break(());
                }
                self.pop_and_clear();
            }
        }

        if !was_any_fit && self.how_many_free < self.acceptance_threshold {
            // TODO: check if same set of free cells is already stored

            self.results.push(PlacementResult {
                placement: self.stack.clone(),
                free: 0,
            });
            self.stats.results_inc();

            if let Some(limit) = self.results_limit {
                if self.results.len() == limit.get() {
                    return ControlFlow::Break(());
                }
            }
        }

        ControlFlow::Continue(())
    }

    fn fill_and_push(&mut self, tetra: PlacedBoundariesChecked) {
        for i in tetra.iter_relative_to_place() {
            self.grid[i.row][i.col] = Cell::Occupied;
            self.how_many_free -= 1;
        }
        self.stack.push(tetra);
    }

    fn pop_and_clear(&mut self) {
        let placed_tetra = self.stack.pop().unwrap();
        for i in placed_tetra.iter_relative_to_place() {
            self.grid[i.row][i.col] = Cell::Empty;
            self.how_many_free += 1;
        }
    }

    fn find_any_fit_for(&self, tetra: &'static Tetra) -> Option<PlacedBoundariesChecked> {
        self.positions_for_lookup
            .iter()
            .map(|pos| {
                PlacedBoundariesChecked::in_boundaries(
                    Placed::new(tetra, *pos),
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
}

#[derive(Debug)]
pub struct PlacementResult {
    pub placement: Placement,
    pub free: usize,
}

pub trait CollectStats {
    fn recursions_inc(&mut self);

    fn results_inc(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct StatsDummy;

    impl CollectStats for StatsDummy {
        fn recursions_inc(&mut self) {}

        fn results_inc(&mut self) {}
    }

    // #[test]
    mod caching {
        use super::*;
        use crate::tetra::I_HORIZONTAL;

        fn config_factory() -> Configuration {
            Configuration::new(Size::new(8, 8), HashSet::new())
        }

        impl<'a, S> RecursionState<'a, S>
        where
            S: CollectStats,
        {
            fn force_fill(&mut self, tetra: &'static Tetra) {
                self.fill_and_push(self.find_any_fit_for(tetra).unwrap());
            }
        }

        #[test]
        fn all_positions_initially() {
            let mut stats = StatsDummy;
            let rec = RecursionState::with_configuration(&config_factory(), &mut stats);

            assert_eq!(rec.positions_for_lookup.len(), 8 * 8);
        }

        #[test]
        fn cache_behaviour() {
            let mut stats = StatsDummy;
            let mut rec = RecursionState::with_configuration(&config_factory(), &mut stats);

            rec.force_fill(I_HORIZONTAL);
            rec.force_fill(I_HORIZONTAL);
            rec.force_fill(I_HORIZONTAL);

            assert_eq!(rec.positions_for_lookup.len(), 8 * 8);

            rec.force_fill(I_HORIZONTAL);

            assert_eq!(rec.positions_for_lookup.len(), 8 * 8 - 4 * 4);

            rec.force_fill(I_HORIZONTAL);

            // still
            assert_eq!(rec.positions_for_lookup.len(), 8 * 8 - 4 * 4);

            rec.pop_and_clear();
            rec.pop_and_clear();

            assert_eq!(rec.positions_for_lookup.len(), 8 * 8);
        }
    }

    #[test]
    fn results_count_for_empty_4x4() {
        let cfg = Configuration::new(Size::new(4, 4), HashSet::new());

        let results = cfg.run(&mut StatsDummy);

        assert_eq!(results.len(), 267);
    }

    #[test]
    fn results_count_for_non_empty_6x6() {
        let unavailable = {
            let mut set = HashSet::new();
            for (row, col) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                set.insert(Pos::new(row, col));
            }
            set
        };
        let cfg = Configuration::new(Size::new(6, 6), unavailable);

        let results = cfg.run(&mut StatsDummy);

        assert_eq!(results.len(), 44);
    }
}
