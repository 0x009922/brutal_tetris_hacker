use std::io::stdout;

use clap::Parser;
use crossterm::style::Print;
use crossterm::{cursor, terminal, ExecutableCommand, Result};

use crate::brute_force::CollectStats;

mod util {
    use std::fmt::{Debug, Formatter};
    use std::ops::Add;

    use derive_more::Display;
    use grid::Grid;

    #[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Display)]
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
}

mod tetro {
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
}

mod brute_force {
    use std::collections::HashSet;

    use grid::Grid;

    use crate::tetro::{static_tetros_iter, PlacedTetro, PlacedTetroInBoundaries, Tetro};
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

            // initially - all positions
            let mut iter_positions = Vec::with_capacity(cols * rows);
            for row in 0..rows {
                for col in 0..cols {
                    iter_positions.push(Pos { row, col })
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
        }

        fn pop_and_clear(&mut self) {
            let placed_tetro = self.stack.pop().unwrap();
            for i in placed_tetro.iter_relative_to_place() {
                self.grid[i.row][i.col] = Cell::Empty;
                self.how_many_free += 1;
            }
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
    use crate::tetro::PlacedTetroInBoundaries;
    use crate::util::{Pos, Size};

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
            unavailable: HashSet<Pos>,
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
                Configuration::new((self.rows.0, self.cols.0).into(), self.unavailable)
            }

            fn cursor_as_pos(&self) -> Pos {
                (self.cursor.0 .0, self.cursor.1 .0).into()
            }

            fn as_size(&self) -> Size {
                (self.rows.0, self.cols.0).into()
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

                print_field_setup(
                    self.as_size(),
                    &self.unavailable,
                    Some(self.cursor_as_pos()),
                    RawMode::Enabled,
                )?;

                Ok(())
            }

            fn align_cursor(&mut self) {
                self.cursor.0 .0 = self.cursor.0 .0.min(self.rows.0 - 1);
                self.cursor.1 .0 = self.cursor.1 .0.min(self.cols.0 - 1);
            }

            fn toggle_under_cursor(&mut self) {
                let entry = self.cursor_as_pos();
                if self.unavailable.contains(&entry) {
                    self.unavailable.remove(&entry);
                } else {
                    self.unavailable.insert(entry);
                };
            }
        }
    }

    impl Configuration {
        pub fn print_field(&self) -> Result<()> {
            stdout().execute(Print("Field:\n\n"))?;
            print_field_setup(self.size, &self.unavailable, None, RawMode::Disabled)?;
            stdout().execute(Print("\n"))?;
            Ok(())
        }
    }

    enum RawMode {
        Enabled,
        Disabled,
    }

    fn print_field_setup(
        size: Size,
        unavailable: &HashSet<Pos>,
        cursor: Option<Pos>,
        raw_mode: RawMode,
    ) -> Result<()> {
        for row in 0..size.rows {
            match raw_mode {
                RawMode::Enabled => execute!(stdout(), cursor::MoveRight(2))?,
                RawMode::Disabled => execute!(stdout(), Print("  "))?,
            }

            for col in 0..size.cols {
                let under_cursor = cursor
                    .map(|pos| (row, col) == (pos.row, pos.col))
                    .unwrap_or(false);

                if unavailable.contains(&Pos::new(row, col)) {
                    execute!(
                        stdout(),
                        SetBackgroundColor(if under_cursor {
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
                        SetBackgroundColor(if under_cursor {
                            Color::DarkGrey
                        } else {
                            Color::White
                        }),
                        Print(CHAR_EMPTY),
                        ResetColor
                    )?;
                };
            }

            match raw_mode {
                RawMode::Enabled => execute!(stdout(), cursor::MoveToNextLine(1))?,
                RawMode::Disabled => execute!(stdout(), Print("\n"))?,
            }
        }

        Ok(())
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
            views: HashMap<PlacedTetroInBoundaries, View>,
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

            fn update(mut self, item: PlacedTetroInBoundaries) -> Self {
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

        let mut grid = Grid::init(conf.size.rows, conf.size.cols, View::Empty);

        for item in result.placement.iter() {
            let view = views.get(item).unwrap();
            for i in item.iter_relative_to_place() {
                grid[i.row][i.col] = (*view).clone();
            }
        }

        for Pos { row, col } in conf.unavailable.iter() {
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

        if self.recursions % 100_000 == 0 {
            stdout()
                .execute(terminal::Clear(terminal::ClearType::CurrentLine))
                .unwrap()
                .execute(cursor::MoveLeft(100))
                .unwrap()
                .execute(Print(format!(
                    "recursions: {}, time: {:.2?}, results: {}",
                    self.recursions,
                    self.start.elapsed(),
                    self.results
                )))
                .unwrap();
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

    conf.print_field()?;

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
