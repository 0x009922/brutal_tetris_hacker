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
    use crate::grid_checker::cells::Cells;
    use grid::Grid;

    use crate::maths::Pos;
    use crate::tetro::{PositionedTetro, Tetro, TetroIter};

    mod cells {
        use super::{Grid, Pos, PositionedTetro, Tetro};

        #[derive(Clone, derive_more::Display)]
        pub enum Cell {
            #[display(fmt = "-")]
            Empty,
            #[display(fmt = "x")]
            Unavailable,
            #[display(fmt = "+")]
            Occupied,
        }

        pub struct Cells {
            grid: Grid<Cell>,
            initially_unavailable: usize,
            how_many_free: usize,
        }

        impl Cells {
            pub fn new<'a>(
                (rows, cols): (usize, usize),
                unavailable: impl Iterator<Item = &'a (usize, usize)>,
            ) -> Self {
                let mut grid = Grid::init(rows, cols, Cell::Empty);
                let mut how_many_free = grid.cols() * grid.cols();
                let mut initially_unavailable = 0;
                for (x, y) in unavailable {
                    grid[*x][*y] = Cell::Unavailable;
                    initially_unavailable += 1;
                    how_many_free -= 1;
                }

                Self {
                    grid,
                    how_many_free,
                    initially_unavailable,
                }
            }

            pub fn try_put(&mut self, tetro: &Tetro) -> Option<Pos> {
                match self.find_any_fit_for(tetro) {
                    Some(pos) => {
                        for tetro_pos in tetro.iter() {
                            let Pos { row, col } = &pos + tetro_pos;
                            self.grid[row][col] = Cell::Occupied;
                            self.how_many_free -= 1;
                        }

                        Some(pos)
                    }
                    None => None,
                }
            }

            pub fn unwind(&mut self, PositionedTetro { tetro, pos }: &PositionedTetro) {
                for tetro_pos in tetro.iter() {
                    let Pos { row, col } = pos + tetro_pos;
                    self.grid[row][col] = Cell::Empty;
                    self.how_many_free += 1;
                }
            }

            fn find_any_fit_for(&self, tetro: &Tetro) -> Option<Pos> {
                let tetro_size = tetro.size();

                for row in 0..(self.grid.rows() - tetro_size.0 + 1) {
                    for col in 0..(self.grid.cols() - tetro_size.1 + 1) {
                        let pos = Pos { row, col };

                        let all_empty = tetro.iter().all(|tetro_pos| {
                            let Pos { row, col } = &pos + tetro_pos;
                            matches!(self.grid[row][col], Cell::Empty)
                        });

                        if all_empty {
                            return Some(pos);
                        }
                    }
                }

                None
            }

            pub fn how_many_free(&self) -> &usize {
                &self.how_many_free
            }

            pub fn min_free_cells(&self) -> usize {
                let available = self.grid.cols() * self.grid.rows() - self.initially_unavailable;
                available % 4
            }
        }
    }

    pub type Placement = Vec<PositionedTetro>;

    pub fn find_placements<'a>(
        size: (usize, usize),
        unavailable: impl Iterator<Item = &'a (usize, usize)>,
    ) -> Vec<Placement> {
        enum RecursiveReturn {
            Cont,
            Stop,
        }

        // struct FoundPlacements {
        //     items: Vec<()>,
        // }

        struct Perf {
            iters: u128,
            ts: std::time::Instant,
        }

        fn recursive(
            cells: &mut Cells,
            stack: &mut Vec<PositionedTetro>,
            results: &mut Vec<Placement>,
            perf: &mut Perf,
        ) -> RecursiveReturn {
            perf.iters += 1;
            if perf.iters % 10_000 == 0 {
                println!("iters: {}, time: {:.2?}", perf.iters, perf.ts.elapsed());
            }
            for tetro in TetroIter::new() {
                if let Some(pos) = cells.try_put(tetro) {
                    stack.push(PositionedTetro::new(tetro.clone(), pos));
                    let ret = recursive(cells, stack, results, perf);
                    let item = stack.pop().unwrap();
                    cells.unwind(&item);
                    match ret {
                        RecursiveReturn::Cont => continue,
                        RecursiveReturn::Stop => return ret,
                    }
                } else if *cells.how_many_free() == cells.min_free_cells() {
                    results.push(stack.clone());
                    if results.len() == 1 {
                        return RecursiveReturn::Stop;
                    }
                }
            }

            RecursiveReturn::Cont
        }

        let mut stack = Vec::with_capacity((size.0 * size.1) / 4);
        let mut grid = Cells::new(size, unavailable);

        let mut results = Vec::new();
        recursive(
            &mut grid,
            &mut stack,
            &mut results,
            &mut Perf {
                iters: 0,
                ts: std::time::Instant::now(),
            },
        );

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
    use crate::reporter::{CHAR_EMPTY, CHAR_UNAVAILABLE};

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
        pub unavailable: HashSet<(usize, usize)>,
    }

    impl Configuration {
        pub fn find_placements(&self) -> Vec<Placement> {
            super::grid_checker::find_placements(self.size, self.unavailable.iter())
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
                        event::KeyCode::Left => self.cursor.1.dec(),
                        event::KeyCode::Right => self.cursor.1.inc(),
                        event::KeyCode::Up => self.cursor.0.dec(),
                        event::KeyCode::Down => self.cursor.0.inc(),
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
                unavailable: self.occupied,
            }
        }

        fn print(&self) -> Result<()> {
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
                    let ch = if self.occupied.contains(&(row, col)) {
                        CHAR_UNAVAILABLE
                    } else {
                        CHAR_EMPTY
                    };
                    stdout().execute(Print(ch))?;
                }
                stdout().execute(cursor::MoveToNextLine(1))?;
            }

            stdout().execute(cursor::MoveTo(
                (cursor_base.0 + self.cursor.1 .0) as u16,
                (cursor_base.1 + self.cursor.0 .0) as u16,
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

    use crossterm::style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor};
    use crossterm::{style::Print, ExecutableCommand, Result};
    use grid::Grid;

    use crate::grid_checker::Placement;
    use crate::live_terminal_conf::Configuration;
    use crate::maths::Pos;
    use crate::tetro::PositionedTetro;

    pub const CHAR_EMPTY: char = '·';
    pub const CHAR_UNAVAILABLE: char = '×';

    pub fn report(placement: &Placement, conf: &Configuration) -> Result<()> {
        #[derive(Clone)]
        struct View {
            char: char,
            color: Option<Color>,
            attrs: Vec<Option<Attribute>>,
        }

        struct ViewsComposer {
            views: HashMap<PositionedTetro, View>,
            colors: Vec<Color>,
            attrs: Vec<Option<Attribute>>,
        }

        impl ViewsComposer {
            fn new() -> Self {
                Self {
                    views: HashMap::new(),
                    colors: vec![
                        Color::Green,
                        Color::Cyan,
                        Color::Blue,
                        Color::Magenta,
                        Color::Yellow,
                    ],
                    attrs: vec![None, Some(Attribute::Bold), Some(Attribute::Italic)],
                }
            }

            fn update(mut self, item: PositionedTetro) -> Self {
                let idx = self.views.len();

                let sym = char::from_u32(
                    // ASCII 'a'
                    (97 + idx) as u32,
                )
                .unwrap();

                let idx_in_colors_total = idx % (self.colors.len() * self.attrs.len());
                let idx_in_colors = idx_in_colors_total % self.colors.len();
                let idx_in_attrs = idx_in_colors_total / self.colors.len();

                self.views.insert(
                    item,
                    View {
                        char: sym,
                        color: Some(self.colors[idx_in_colors]),
                        attrs: vec![self.attrs[idx_in_attrs]],
                    },
                );
                self
            }
        }

        let ViewsComposer { views, .. } = placement
            .iter()
            .fold(ViewsComposer::new(), |acc, item| acc.update(item.clone()));

        let mut grid = Grid::init(
            conf.size.0,
            conf.size.1,
            View {
                char: CHAR_EMPTY,
                color: None,
                attrs: vec![Some(Attribute::Dim)],
            },
        );

        for item in placement {
            let PositionedTetro { pos, tetro } = item;
            let view = views.get(item).unwrap();
            for i_pos in tetro.iter() {
                let Pos { row, col } = pos + i_pos;
                grid[row][col] = (*view).clone();
            }
        }

        for (row, col) in conf.unavailable.iter() {
            grid[*row][*col] = View {
                char: CHAR_UNAVAILABLE,
                color: Some(Color::Red),
                attrs: Vec::new(),
            }
        }

        for row in 0..grid.rows() {
            stdout().execute(Print("  "))?;
            for x in grid.iter_row(row) {
                if let Some(col) = x.color {
                    stdout().execute(SetForegroundColor(col))?;
                }
                stdout().execute(Print(x.char))?;
                for attr in x.attrs.iter().flatten() {
                    stdout().execute(SetAttribute(*attr))?;
                }
                stdout().execute(ResetColor)?;
            }
            stdout().execute(Print("\n"))?;
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let conf = live_terminal_conf::State::new(6, 6)
        .configure()?
        .into_configuration();
    let placements = conf.find_placements();

    stdout().execute(Print(format!("Found placements: {}\n\n", placements.len())))?;

    for item in placements {
        reporter::report(&item, &conf)?;
        stdout().execute(Print("\n"))?;
    }

    Ok(())
}
