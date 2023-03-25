use std::io::stdout;

use clap::Parser;
use crossterm::style::Print;
use crossterm::{terminal, ExecutableCommand, Result};

use crate::brute_force::CollectStats;

mod maths {
    use std::fmt::{Debug, Formatter};
    use std::ops::Add;

    use derive_more::Display;

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
}

mod brute_force {
    use std::collections::HashSet;
    use std::ops::Add;

    use grid::Grid;

    use crate::maths::Pos;
    use crate::tetro::{static_tetros_iter, PlacedTetro, Tetro};

    pub type Placement = Vec<PlacedTetro>;

    pub struct Configuration {
        pub size: (usize, usize),
        pub unavailable: HashSet<(usize, usize)>,
    }

    impl Configuration {
        pub fn new(size: (usize, usize), unavailable: HashSet<(usize, usize)>) -> Self {
            Self { size, unavailable }
        }

        pub fn brute_force<S>(&self, stats: &'_ mut S) -> Vec<PlacementResult>
        where
            S: CollectStats,
        {
            RecursionState::brute_force(&self, stats)
        }
    }

    #[derive(Clone, derive_more::Display, Debug)]
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

        stack: Vec<PlacedTetro>,
        results: Vec<PlacementResult>,

        stats: &'a mut S,
    }

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

        fn new(
            (rows, cols): (usize, usize),
            unavailable: &'_ HashSet<(usize, usize)>,
            stats: &'a mut S,
        ) -> Self {
            let mut grid = Grid::init(rows, cols, Cell::Empty);
            let mut how_many_free = rows * cols;
            let mut initially_unavailable = 0;
            for (x, y) in unavailable.iter() {
                grid[*x][*y] = Cell::Unavailable;
                initially_unavailable += 1;
                how_many_free -= 1;
            }
            let min_free_cells = (cols * rows - initially_unavailable) % 4;

            let stack = Vec::with_capacity(cols * rows);
            let results = Vec::new();

            Self {
                grid,
                how_many_free,
                min_free_cells,

                stack,
                results,
                stats,
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
                if let Some(pos) = self.find_any_fit_for(tetro) {
                    self.fill_and_push(PlacedTetro::new(tetro, pos));
                    self.recursion();
                    self.pop_and_clear();
                }
            }
        }

        fn fill_and_push(&mut self, positioned: PlacedTetro) {
            for i in positioned.tetro.iter() {
                let Pos { row, col } = positioned.position.add(i);
                self.grid[row][col] = Cell::Occupied;
                self.how_many_free -= 1;
            }
            self.stack.push(positioned);
        }

        fn pop_and_clear(&mut self) {
            let PlacedTetro {
                tetro,
                position: pos,
            } = self.stack.pop().unwrap();
            for i in tetro.iter() {
                let Pos { row, col } = pos.add(i);
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
    }

    pub struct PlacementResult {
        pub placement: Placement,
        pub free: usize,
    }

    pub trait CollectStats {
        fn recursions_inc(&mut self);

        fn results_inc(&mut self);
    }
}

mod app_terminal {
    use std::collections::{HashMap, HashSet};
    use std::fmt::Write;
    use std::io::stdout;
    use std::ops::Add;

    use crossterm::style::{
        Attribute, Color, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    };
    use crossterm::{
        cursor, event,
        event::Event,
        execute,
        style::Print,
        terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
        Command, ExecutableCommand, Result,
    };
    use grid::Grid;

    use crate::brute_force::{Configuration, PlacementResult};
    use crate::maths::Pos;
    use crate::tetro::PlacedTetro;

    pub const CHAR_EMPTY: char = '·';
    pub const CHAR_UNAVAILABLE: char = '×';

    pub mod live_configuration {
        use super::*;

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

        pub struct State {
            rows: Bounded<1, { usize::MAX }>,
            cols: Bounded<1, { usize::MAX }>,
            cursor: (Bounded<0, { usize::MAX }>, Bounded<0, { usize::MAX }>),
            unavailable: HashSet<(usize, usize)>,
        }

        impl State {
            pub fn new(rows: usize, cols: usize) -> Self {
                Self {
                    rows: Bounded(rows),
                    cols: Bounded(cols),
                    cursor: (Bounded(0), Bounded(0)),
                    unavailable: HashSet::new(),
                }
            }

            pub fn live(mut self) -> Result<Self> {
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
                Configuration::new((self.rows.0, self.cols.0), self.unavailable)
            }

            fn print(&self) -> Result<()> {
                stdout()
                    .execute(cursor::MoveTo(0, 0))?
                    .execute(Print(
                        "Controls: wasd - resize; arrows - move; space - click; esc - stop"
                            .to_string(),
                    ))?
                    .execute(cursor::MoveToNextLine(2))?
                    .execute(Print(format!("N x M: {} x {}", self.rows.0, self.cols.0)))?
                    .execute(cursor::MoveToNextLine(2))?;

                for row in 0..self.rows.0 {
                    stdout().execute(cursor::MoveRight(2))?;

                    for col in 0..self.cols.0 {
                        let cursor = (row, col) == (self.cursor.0 .0, self.cursor.1 .0);

                        if self.unavailable.contains(&(row, col)) {
                            execute!(
                                stdout(),
                                SetBackgroundColor(if cursor {
                                    Color::DarkRed
                                } else {
                                    Color::White
                                }),
                                SetAttribute(Attribute::Bold),
                                SetForegroundColor(Color::DarkRed),
                                Print(CHAR_UNAVAILABLE),
                                ResetColor
                            )?;
                        } else {
                            execute!(
                                stdout(),
                                SetBackgroundColor(if cursor {
                                    Color::DarkGrey
                                } else {
                                    Color::White
                                }),
                                Print(CHAR_EMPTY),
                                ResetColor
                            )?;
                        };
                    }

                    stdout().execute(cursor::MoveToNextLine(1))?;
                }

                Ok(())
            }

            fn align_cursor(&mut self) {
                self.cursor.0 .0 = self.cursor.0 .0.min(self.rows.0 - 1);
                self.cursor.1 .0 = self.cursor.1 .0.min(self.cols.0 - 1);
            }

            fn toggle_under_cursor(&mut self) {
                let entry = (self.cursor.0 .0, self.cursor.1 .0);
                if self.unavailable.contains(&entry) {
                    self.unavailable.remove(&entry);
                } else {
                    self.unavailable.insert(entry);
                };
            }
        }
    }

    pub fn report_placement(result: &PlacementResult, conf: &Configuration) -> Result<()> {
        #[derive(Clone)]
        enum View {
            Tetro {
                char: char,
                color: Color,
                attr: OptionAttribute,
            },
            Empty,
            Unavailable,
        }

        #[derive(Clone)]
        struct OptionAttribute(Option<Attribute>);

        impl Command for OptionAttribute {
            fn write_ansi(&self, f: &mut impl Write) -> std::fmt::Result {
                match self.0 {
                    Some(attr) => SetAttribute(attr).write_ansi(f),
                    None => Ok(()),
                }
            }
        }

        struct TetroViewsComposer {
            views: HashMap<PlacedTetro, View>,
            colors: Vec<Color>,
            attrs: Vec<Option<Attribute>>,
        }

        impl TetroViewsComposer {
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

            fn update(mut self, item: PlacedTetro) -> Self {
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
                    View::Tetro {
                        char: sym,
                        color: self.colors[idx_in_colors],
                        attr: OptionAttribute(self.attrs[idx_in_attrs]),
                    },
                );
                self
            }
        }

        let TetroViewsComposer { views, .. } = result
            .placement
            .iter()
            .fold(TetroViewsComposer::new(), |acc, item| {
                acc.update(item.clone())
            });

        let mut grid = Grid::init(conf.size.0, conf.size.1, View::Empty);

        for item in result.placement.iter() {
            let view = views.get(item).unwrap();
            let PlacedTetro {
                position: pos,
                tetro,
            } = item;
            for i in tetro.iter() {
                let Pos { row, col } = pos.add(i);
                grid[row][col] = (*view).clone();
            }
        }

        for (row, col) in conf.unavailable.iter() {
            grid[*row][*col] = View::Unavailable
        }

        for row in 0..grid.rows() {
            stdout().execute(Print("  "))?;
            for view in grid.iter_row(row) {
                match view {
                    View::Empty => execute!(
                        stdout(),
                        SetForegroundColor(Color::Grey),
                        SetAttribute(Attribute::Dim),
                        Print(CHAR_EMPTY),
                        ResetColor,
                    )?,
                    View::Unavailable => execute!(
                        stdout(),
                        SetForegroundColor(Color::DarkRed),
                        SetBackgroundColor(Color::DarkRed),
                        Print(CHAR_UNAVAILABLE),
                        ResetColor
                    )?,
                    View::Tetro { char, color, attr } => {
                        execute!(
                            stdout(),
                            SetForegroundColor(*color),
                            attr,
                            Print(char),
                            ResetColor
                        )?;
                    }
                }
            }
            stdout().execute(Print("\n"))?;
        }

        Ok(())
    }
}

#[derive(Parser)]
struct Args {
    #[arg(long)]
    no_print_placements: bool,
}

struct Stats {
    start: std::time::Instant,
    recursions: usize,
    results: usize,
}

impl Stats {
    fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
            recursions: 0,
            results: 0,
        }
    }
}

impl CollectStats for Stats {
    fn recursions_inc(&mut self) {
        self.recursions += 1;

        if self.recursions % 10_000 == 0 {
            println!(
                "recursions: {}, time: {:.2?}, results: {}",
                self.recursions,
                self.start.elapsed(),
                self.results
            );
        }
    }

    fn results_inc(&mut self) {
        self.results += 1;
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let conf = app_terminal::live_configuration::State::new(4, 4)
        .live()?
        .into_configuration();

    let mut stats = Stats::new();
    let placements = conf.brute_force(&mut stats);
    let elapsed = stats.start.elapsed();

    if !args.no_print_placements {
        for item in placements.iter() {
            app_terminal::report_placement(item, &conf)?;
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
