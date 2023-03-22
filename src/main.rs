use clap::Parser;
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
    pub struct Tetro {
        positions: [Pos; 4],
        size: (usize, usize),
    }

    // impl std::fmt::Debug for Tetro {
    //     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    //         std::fmt::Display::fmt(&self, f)
    //     }
    // }

    const TETRO_COUNT: usize = 19;

    const fn const_tetro(positions: [(usize, usize); 4]) -> Tetro {
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
            size,
        }
    }

    macro_rules! tetro {
        ($a:expr, $b:expr, $c:expr, $d:expr) => {
            const_tetro([$a, $b, $c, $d])
        };
    }

    pub const TETROS: [Tetro; TETRO_COUNT] = [
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
        pub fn size(&self) -> &(usize, usize) {
            &self.size
        }

        pub fn iter(&self) -> impl Iterator<Item = &Pos> {
            self.positions.iter()
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
    pub struct PositionedTetro {
        pub tetro: &'static Tetro,
        pub pos: Pos,
    }

    impl PositionedTetro {
        pub fn new(tetro: &'static Tetro, pos: Pos) -> Self {
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

        #[derive(Clone, derive_more::Display, Debug)]
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
            how_many_free: usize,
            min_free_cells: usize,
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
                let min_free_cells = (grid.cols() * grid.rows() - initially_unavailable) % 4;

                Self {
                    grid,
                    how_many_free,
                    min_free_cells,
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

                if tetro_size.0 > self.grid.rows() || tetro_size.1 > self.grid.cols() {
                    return None;
                }

                for row in 0..(self.grid.rows() - (tetro_size.0) + 1) {
                    for col in 0..(self.grid.cols() - (tetro_size.1) + 1) {
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

            pub fn min_free_cells(&self) -> &usize {
                &self.min_free_cells
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
            {
                let how_many_free = cells.how_many_free();
                let min_free_cells = cells.min_free_cells();

                if how_many_free == min_free_cells {
                    results.push(stack.clone());
                    // if results.len() == RESULTS_COUNT {
                    //     return RecursiveReturn::Stop;
                    // }
                }
            }

            perf.iters += 1;
            if perf.iters % 100_000 == 0 {
                println!(
                    "iters: {}, time: {:.2?}, results: {}",
                    perf.iters,
                    perf.ts.elapsed(),
                    results.len()
                );
            }

            for tetro in TetroIter::new() {
                if let Some(pos) = cells.try_put(tetro) {
                    stack.push(PositionedTetro::new(tetro, pos));
                    match recursive(cells, stack, results, perf) {
                        RecursiveReturn::Cont => {
                            let item = stack.pop().unwrap();
                            cells.unwind(&item);
                        }
                        ret @ RecursiveReturn::Stop => return ret,
                    }
                }
            }

            RecursiveReturn::Cont
        }

        let mut stack = Vec::with_capacity((size.0 * size.1) / 4);
        let mut grid = Cells::new(size, unavailable);

        let mut results = Vec::with_capacity(stack.len());
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

    use crossterm::style::{
        Attribute, Color, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    };
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
            color_fore: Option<Color>,
            color_back: Option<Color>,
            attrs: Vec<Option<Attribute>>,
        }

        impl View {
            fn new(char: char) -> Self {
                Self {
                    char,
                    color_fore: None,
                    color_back: None,
                    attrs: vec![],
                }
            }

            fn foreground(mut self, color: Color) -> Self {
                self.color_fore = Some(color);
                self
            }

            fn background(mut self, color: Color) -> Self {
                self.color_back = Some(color);
                self
            }

            fn attr(mut self, attr: Option<Attribute>) -> Self {
                self.attrs.push(attr);
                self
            }
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
                    View::new(sym)
                        .foreground(self.colors[idx_in_colors])
                        .attr(self.attrs[idx_in_attrs]),
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
            View::new(CHAR_EMPTY).attr(Some(Attribute::Dim)),
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
            grid[*row][*col] = View::new(CHAR_UNAVAILABLE)
                .foreground(Color::DarkRed)
                .background(Color::DarkRed)
        }

        for row in 0..grid.rows() {
            stdout().execute(Print("  "))?;
            for x in grid.iter_row(row) {
                if let Some(col) = x.color_fore {
                    stdout().execute(SetForegroundColor(col))?;
                }
                if let Some(col) = x.color_back {
                    stdout().execute(SetBackgroundColor(col))?;
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

const RESULTS_COUNT: usize = 3;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    no_print_placements: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let conf = live_terminal_conf::State::new(4, 4)
        .configure()?
        .into_configuration();

    let start = std::time::Instant::now();
    let placements = conf.find_placements();
    let elapsed = start.elapsed();

    if !args.no_print_placements {
        for item in placements.iter() {
            reporter::report(item, &conf)?;
            stdout().execute(Print("\n"))?;
        }
    }

    stdout().execute(Print(format!(
        "\n  Found placements: {} (time: {:.2?})\n",
        placements.len(),
        elapsed
    )))?;

    Ok(())
}
