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

use crate::algorithm::{Configuration, PlacementResult};
use crate::tetra::PlacedBoundariesChecked;
use crate::util::{Pos, Size};

pub const CHAR_EMPTY: char = '·';
pub const CHAR_UNAVAILABLE: char = '×';

pub mod live_configuration {
    use super::{
        cursor, event, print_field_setup, stdout, terminal, Clear, ClearType, Configuration,
        EnterAlternateScreen, Event, ExecutableCommand, HashSet, LeaveAlternateScreen, Pos, Print,
        RawMode, Result, Size,
    };
    use crossterm::style::{Attribute, Color, SetAttribute, SetForegroundColor};

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
            enum LoopResult {
                Terminate,
                Proceed,
            }

            stdout().execute(EnterAlternateScreen)?;
            terminal::enable_raw_mode()?;

            self.print()?;

            let loop_result = loop {
                if let Event::Key(event::KeyEvent { code, .. }) = event::read()? {
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
                .execute(Print("Controls:"))?
                .execute(cursor::MoveToNextLine(2))?
                .execute(cursor::MoveRight(2))?;

            fn print_simple_controls(controls: &str) -> Result<()> {
                for (i, symbol) in controls.chars().enumerate() {
                    if i > 0 {
                        stdout()
                            .execute(SetForegroundColor(Color::Grey))?
                            .execute(SetAttribute(Attribute::Dim))?
                            .execute(Print(" / "))?
                            .execute(SetAttribute(Attribute::Reset))?;
                    }

                    stdout()
                        .execute(SetForegroundColor(Color::Blue))?
                        .execute(Print(symbol))?;
                }

                Ok(())
            }

            print_simple_controls("WASD")?;

            stdout()
                .execute(SetForegroundColor(Color::Reset))?
                .execute(Print(" - resize the field"))?
                .execute(cursor::MoveToNextLine(1))?
                .execute(cursor::MoveRight(2))?;

            print_simple_controls("↑←↓→")?;

            stdout()
                .execute(SetForegroundColor(Color::Reset))?
                .execute(Print(" - move the cursor"))?
                .execute(cursor::MoveToNextLine(1))?
                .execute(cursor::MoveRight(2))?
                .execute(SetForegroundColor(Color::Blue))?
                .execute(Print("Space"))?
                .execute(SetForegroundColor(Color::Reset))?
                .execute(Print(" - toggle the cell"))?
                .execute(cursor::MoveToNextLine(1))?
                .execute(cursor::MoveRight(2))?
                .execute(SetForegroundColor(Color::Yellow))?
                .execute(Print("Esc"))?
                .execute(SetForegroundColor(Color::Reset))?
                .execute(Print(" - ../"))?;

            stdout().execute(cursor::MoveToNextLine(2))?;

            // execute!(
            //     stdout(),
            //     cursor::MoveTo(0, 0),
            //     Print("Controls:"),
            //     cursor::MoveTo(2, 1),
            //     SetForegroundColor(Color::Blue),
            //     Print("W")
            // )?;
            // stdout()
            //     .execute(cursor::MoveTo(0, 0))?
            //     .execute(Print(
            //         "Controls: wasd - resize; arrows - move; space - click; esc - stop".to_string(),
            //     ))?
            //     .execute(cursor::MoveToNextLine(2))?
            //     .execute(Print(format!("N x M: {} x {}", self.rows.0, self.cols.0)))?
            //     .execute(cursor::MoveToNextLine(2))?;

            print_field_setup(
                self.as_size(),
                &self.unavailable,
                Some(self.cursor_as_pos()),
                &RawMode::Enabled,
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
        print_field_setup(self.size, &self.unavailable, None, &RawMode::Disabled)?;
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
    raw_mode: &RawMode,
) -> Result<()> {
    for row in 0..size.rows {
        match raw_mode {
            RawMode::Enabled => execute!(stdout(), cursor::MoveRight(2))?,
            RawMode::Disabled => execute!(stdout(), Print("  "))?,
        }

        for col in 0..size.cols {
            let under_cursor = cursor.map_or(false, |pos| (row, col) == (pos.row, pos.col));

            if unavailable.contains(&Pos::new(row, col)) {
                execute!(
                    stdout(),
                    SetBackgroundColor(if under_cursor {
                        Color::DarkRed
                    } else {
                        Color::Reset
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
                        Color::Reset
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

#[derive(Clone)]
enum CellView {
    Tetra(TetraView),
    Empty,
    Unavailable,
}

#[derive(Clone)]
struct TetraView {
    char: char,
    color: Color,
    attr: OptionAttribute,
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

fn compose_tetra_views(result: &PlacementResult) -> HashMap<&PlacedBoundariesChecked, TetraView> {
    const COLORS: [Color; 5] = [
        Color::Green,
        Color::Cyan,
        Color::Blue,
        Color::Magenta,
        Color::Yellow,
    ];

    const ATTRIBUTES: [Option<Attribute>; 3] =
        [None, Some(Attribute::Bold), Some(Attribute::Italic)];

    let map: HashMap<_, _> = result
        .placement
        .iter()
        .enumerate()
        .map(|(idx, tetra)| {
            let sym = char::from_u32(('A' as usize + idx) as u32).unwrap();

            let relative = idx % (COLORS.len() * ATTRIBUTES.len());
            let idx_color = relative % COLORS.len();
            let idx_attr = relative / COLORS.len();

            let view = TetraView {
                char: sym,
                color: COLORS[idx_color],
                attr: OptionAttribute(ATTRIBUTES[idx_attr]),
            };

            (tetra, view)
        })
        .collect();

    map
}

fn grid_view(result: &PlacementResult, conf: &Configuration) -> Grid<CellView> {
    let mut grid = Grid::init(conf.size.rows, conf.size.cols, CellView::Empty);

    for Pos { row, col } in &conf.unavailable {
        grid[*row][*col] = CellView::Unavailable;
    }

    for (tetra, view) in compose_tetra_views(result) {
        for Pos { row, col } in tetra.iter_relative_to_place() {
            grid[row][col] = CellView::Tetra(view.clone());
        }
    }

    grid
}

pub fn report_placement(result: &PlacementResult, conf: &Configuration) -> Result<()> {
    let grid = grid_view(result, conf);

    for row in 0..grid.rows() {
        stdout().execute(Print("  "))?;
        for view in grid.iter_row(row) {
            match view {
                CellView::Empty => execute!(
                    stdout(),
                    SetForegroundColor(Color::Grey),
                    SetAttribute(Attribute::Dim),
                    Print(CHAR_EMPTY),
                    ResetColor,
                )?,
                CellView::Unavailable => execute!(
                    stdout(),
                    SetForegroundColor(Color::DarkRed),
                    // SetBackgroundColor(Color::DarkRed),
                    Print(CHAR_UNAVAILABLE),
                    ResetColor
                )?,
                CellView::Tetra(TetraView { char, color, attr }) => {
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
