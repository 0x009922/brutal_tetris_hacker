use std::io::stdout;

use crossterm::style::Print;
use crossterm::{ExecutableCommand, Result};

mod maths {
    use derive_more::Display;
    use std::fmt::{Debug, Formatter};
    use std::ops::Add;

    #[derive(Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Display)]
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
}

mod tetro {
    use super::maths::Pos;

    #[derive(Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
    pub struct Tetro([Pos; 4]);

    // impl std::fmt::Debug for Tetro {
    //     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    //         std::fmt::Display::fmt(&self, f)
    //     }
    // }

    const TETRO_COUNT: usize = 19;

    #[inline]
    const fn transform((row, col): (usize, usize)) -> Pos {
        Pos { row, col }
    }

    macro_rules! tetro {
        ($a:expr, $b:expr, $c:expr, $d:expr) => {
            Tetro([
                // ..
                transform($a),
                transform($b),
                transform($c),
                transform($d),
            ])
        };
    }

    const TETROS: [Tetro; TETRO_COUNT] = [
        tetro!((0, 0), (0, 1), (1, 0), (1, 1)),
        tetro!((0, 0), (0, 1), (0, 2), (0, 3)),
        tetro!((0, 0), (1, 0), (2, 0), (3, 0)),
        tetro!((0, 0), (0, 1), (0, 2), (1, 1)),
        tetro!((0, 0), (1, 0), (1, 1), (2, 0)),
        tetro!((0, 1), (1, 0), (1, 1), (1, 2)),
        tetro!((0, 1), (1, 0), (1, 1), (2, 1)),
        tetro!((0, 0), (0, 1), (0, 2), (1, 0)),
        tetro!((0, 0), (1, 0), (2, 0), (2, 1)),
        tetro!((0, 2), (1, 0), (1, 1), (1, 2)),
        tetro!((0, 0), (0, 1), (1, 1), (2, 1)),
        tetro!((0, 0), (0, 1), (0, 2), (1, 2)),
        tetro!((0, 1), (1, 1), (2, 0), (2, 1)),
        tetro!((0, 0), (1, 0), (1, 1), (1, 2)),
        tetro!((0, 0), (0, 1), (1, 0), (2, 0)),
        tetro!((0, 1), (0, 2), (1, 0), (1, 1)),
        tetro!((0, 0), (1, 0), (1, 1), (2, 1)),
        tetro!((0, 0), (0, 1), (1, 1), (1, 2)),
        tetro!((0, 1), (1, 0), (1, 1), (2, 0)),
    ];

    impl Tetro {
        pub fn size(&self) -> (usize, usize) {
            let mut rows = 1;
            let mut cols = 1;

            for Pos { row, col } in self.iter() {
                rows = rows.max(row + 1);
                cols = cols.max(col + 1);
            }

            (rows, cols)
        }

        pub fn iter(&self) -> impl Iterator<Item = &Pos> {
            self.0.iter()
        }
    }

    impl IntoIterator for Tetro {
        type Item = Pos;
        type IntoIter = core::array::IntoIter<Self::Item, 4>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    #[derive(Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
    pub struct PositionedTetro {
        pub tetro: Tetro,
        pub pos: Pos,
    }

    impl PositionedTetro {
        pub fn new(tetro: Tetro, pos: Pos) -> Self {
            Self { tetro, pos }
        }
    }

    pub struct TetroIter {
        i: usize,
    }

    impl TetroIter {
        pub fn new() -> Self {
            Self { i: 0 }
        }
    }

    impl Iterator for TetroIter {
        type Item = &'static Tetro;

        fn next(&mut self) -> Option<Self::Item> {
            if self.i < TETRO_COUNT - 1 {
                self.i += 1;
                Some(&TETROS[self.i - 1])
            } else {
                None
            }
        }
    }
}

mod grid_checker {
    use grid::Grid;
    use std::fmt::{Debug, Formatter};

    use crate::maths::Pos;
    use crate::tetro::{PositionedTetro, Tetro, TetroIter};

    #[derive(Clone, derive_more::Display)]
    pub enum Cell {
        #[display(fmt = "-")]
        Empty,
        #[display(fmt = "x")]
        Unavailable,
        #[display(fmt = "+")]
        Occupied,
    }

    impl Debug for Cell {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            std::fmt::Display::fmt(&self, f)
        }
    }

    trait Placer {
        fn try_put(&mut self, tetro: &Tetro) -> Option<Pos>;

        fn unwind(&mut self, tetro: &PositionedTetro);

        fn how_many_free(&self) -> usize;
    }

    impl Placer for Grid<Cell> {
        fn try_put(&mut self, tetro: &Tetro) -> Option<Pos> {
            fn find(grid: &Grid<Cell>, tetro: &Tetro) -> Option<Pos> {
                let tetro_size = tetro.size();

                for row in 0..(grid.rows() - tetro_size.0 + 1) {
                    for col in 0..grid.cols() - tetro_size.1 + 1 {
                        let pos = Pos { row, col };

                        let all_empty = tetro.iter().all(|tetro_pos| {
                            let Pos { row, col } = &pos + tetro_pos;
                            matches!(grid[row][col], Cell::Empty)
                        });

                        if all_empty {
                            return Some(pos);
                        }
                    }
                }

                None
            }

            match find(self, tetro) {
                Some(pos) => {
                    for tetro_pos in tetro.iter() {
                        let Pos { row, col } = &pos + tetro_pos;
                        self[row][col] = Cell::Occupied;
                    }

                    Some(pos)
                }
                None => None,
            }
        }

        fn unwind(&mut self, PositionedTetro { tetro, pos }: &PositionedTetro) {
            for tetro_pos in tetro.iter() {
                let Pos { row, col } = pos + tetro_pos;
                self[row][col] = Cell::Empty;
            }
        }

        fn how_many_free(&self) -> usize {
            self.iter().fold(0usize, |acc, cell| match cell {
                Cell::Empty => acc + 1,
                _ => acc,
            })
        }
    }

    pub type Placement = Vec<PositionedTetro>;

    pub fn find_placements(
        (rows, cols): (usize, usize),
        unavailable: impl Iterator<Item = (usize, usize)>,
    ) -> Vec<Placement> {
        enum RecursiveReturn {
            Cont,
            Stop,
        }

        fn recursive(
            grid: &mut Grid<Cell>,
            stack: &mut Vec<PositionedTetro>,
            results: &mut Vec<Placement>,
        ) -> RecursiveReturn {
            // dbg!(&grid);
            for tetro in TetroIter::new() {
                if let Some(pos) = grid.try_put(tetro) {
                    stack.push(PositionedTetro::new(tetro.clone(), pos));
                    let ret = recursive(grid, stack, results);
                    let item = stack.pop().unwrap();
                    grid.unwind(&item);
                    match ret {
                        RecursiveReturn::Cont => continue,
                        RecursiveReturn::Stop => return ret,
                    }
                } else if grid.how_many_free() == 0 {
                    results.push(stack.clone());
                    if results.len() == 1 {
                        return RecursiveReturn::Stop;
                    }
                }
            }

            RecursiveReturn::Cont
        }

        let mut grid = Grid::init(rows, cols, Cell::Empty);
        for (x, y) in unavailable {
            grid[x][y] = Cell::Unavailable;
        }

        let mut stack = Vec::with_capacity((grid.rows() * grid.cols()) / 4);
        let mut results = Vec::new();
        recursive(&mut grid, &mut stack, &mut results);

        results
    }
}

mod live_terminal_conf {
    use std::collections::HashSet;
    use std::io::stdout;

    use crossterm::event::Event;
    use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
    use crossterm::{cursor, event, style::Print, terminal, ExecutableCommand, Result};

    use crate::grid_checker::Placement;

    struct Bounded<const N: usize, const M: usize>(usize);

    impl<const N: usize, const M: usize> Bounded<N, M> {
        fn inc(&mut self) {
            if self.0 < M {
                self.0 += 1;
            }
        }

        fn dec(&mut self) {
            if self.0 > N {
                self.0 -= 1;
            }
        }
    }

    pub struct Configuration {
        pub size: (usize, usize),
        pub occupied: HashSet<(usize, usize)>,
    }

    impl Configuration {
        pub fn find_placements(self) -> Vec<Placement> {
            super::grid_checker::find_placements(self.size, self.occupied.into_iter())
        }
    }

    pub struct State {
        rows: Bounded<1, { usize::MAX }>,
        cols: Bounded<1, { usize::MAX }>,
        cursor: (Bounded<0, { usize::MAX }>, Bounded<0, { usize::MAX }>),
        occupied: HashSet<(usize, usize)>,
    }

    impl State {
        pub fn new(rows: usize, cols: usize) -> Self {
            Self {
                rows: Bounded(rows),
                cols: Bounded(cols),
                cursor: (Bounded(0), Bounded(0)),
                occupied: HashSet::new(),
            }
        }

        pub fn configure(mut self) -> Result<Self> {
            stdout().execute(EnterAlternateScreen)?;
            terminal::enable_raw_mode().unwrap();

            self.print()?;

            enum LoopResult {
                Terminate,
                Proceed,
            }

            let loop_result = loop {
                if let Event::Key(event::KeyEvent { code, .. }) = event::read().unwrap() {
                    match code {
                        event::KeyCode::Esc => break LoopResult::Terminate,
                        event::KeyCode::Enter => break LoopResult::Proceed,
                        event::KeyCode::Char('w') => self.rows.dec(),
                        event::KeyCode::Char('s') => self.rows.inc(),
                        event::KeyCode::Char('a') => self.cols.dec(),
                        event::KeyCode::Char('d') => self.cols.inc(),
                        event::KeyCode::Left => self.cursor.0.dec(),
                        event::KeyCode::Right => self.cursor.0.inc(),
                        event::KeyCode::Up => self.cursor.1.dec(),
                        event::KeyCode::Down => self.cursor.1.inc(),
                        event::KeyCode::Char(' ') => self.toggle_under_cursor(),
                        _ => {}
                    }
                }

                stdout().execute(Clear(ClearType::All))?;

                self.align_cursor();
                self.print()?;
            };

            terminal::disable_raw_mode()?;
            stdout().execute(LeaveAlternateScreen)?;

            match loop_result {
                LoopResult::Terminate => {
                    std::process::exit(1);
                }
                LoopResult::Proceed => Ok(self),
            }
        }

        pub fn into_configuration(self) -> Configuration {
            Configuration {
                size: (self.rows.0, self.cols.0),
                occupied: self.occupied,
            }
        }

        fn print(&self) -> Result<()> {
            const FIELD_CHAR_EMPTY: char = '-';
            const FIELD_CHAR_SET: char = '+';

            stdout()
                .execute(cursor::MoveTo(0, 0))?
                .execute(Print(
                    "Controls: wasd - resize; arrows - move; space - click; esc - stop".to_string(),
                ))?
                .execute(cursor::MoveToNextLine(2))?
                .execute(Print(format!("N x M: {} x {}", self.rows.0, self.cols.0)))?
                .execute(cursor::MoveToNextLine(2))?;

            let cursor_base = (2, 4);

            for row in 0..self.rows.0 {
                stdout().execute(Print("  "))?;
                for col in 0..self.cols.0 {
                    let ch = if self.occupied.contains(&(col, row)) {
                        FIELD_CHAR_SET
                    } else {
                        FIELD_CHAR_EMPTY
                    };
                    stdout().execute(Print(ch))?;
                }
                stdout().execute(cursor::MoveToNextLine(1))?;
            }

            stdout().execute(cursor::MoveTo(
                (cursor_base.0 + self.cursor.0 .0) as u16,
                (cursor_base.1 + self.cursor.1 .0) as u16,
            ))?;

            Ok(())
        }

        fn align_cursor(&mut self) {
            self.cursor.0 .0 = self.cursor.0 .0.min(self.cols.0 - 1);
            self.cursor.1 .0 = self.cursor.1 .0.min(self.rows.0 - 1);
        }

        fn toggle_under_cursor(&mut self) {
            let entry = (self.cursor.0 .0, self.cursor.1 .0);
            if self.occupied.contains(&entry) {
                self.occupied.remove(&entry);
            } else {
                self.occupied.insert(entry);
            };
        }
    }
}

mod reporter {
    use std::collections::HashMap;

    use std::io::stdout;

    use crossterm::{style::Print, ExecutableCommand, Result};
    use grid::Grid;

    use crate::grid_checker::Placement;
    use crate::maths::Pos;
    use crate::tetro::PositionedTetro;

    pub trait Report {
        fn report(&self, grid_size: &(usize, usize)) -> Result<()>;
    }

    impl Report for Placement {
        fn report(&self, (rows, cols): &(usize, usize)) -> Result<()> {
            #[derive(Clone)]
            struct View {
                char: char,
            }

            let views = self.iter().fold(HashMap::new(), |mut acc, item| {
                acc.insert(
                    item.clone(),
                    View {
                        char: char::from_u32(
                            // ASCII 'a'
                            (97 + acc.len()) as u32,
                        )
                        .unwrap(),
                    },
                );
                acc
            });

            let mut grid = Grid::init(*rows, *cols, View { char: '-' });

            for item in self {
                let PositionedTetro { pos, tetro } = item;
                let view = views.get(item).unwrap();
                for i_pos in tetro.iter() {
                    let Pos { row, col } = pos + i_pos;
                    grid[row][col] = (*view).clone();
                }
            }

            for row in 0..grid.rows() {
                stdout().execute(Print("  "))?;
                for x in grid.iter_row(row) {
                    stdout().execute(Print(x.char))?;
                }
                stdout().execute(Print("\n"))?;
            }

            Ok(())
        }
    }
}

fn main() -> Result<()> {
    let conf = live_terminal_conf::State::new(4, 4)
        .configure()?
        .into_configuration();
    let size = conf.size;
    let placements = conf.find_placements();

    stdout().execute(Print(format!("Found placements: {}\n\n", placements.len())))?;

    for item in placements {
        reporter::Report::report(&item, &size)?;
        stdout().execute(Print("\n"))?;
    }

    Ok(())
}
