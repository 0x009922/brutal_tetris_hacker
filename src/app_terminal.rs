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
use crate::tetra::PlacedTetraInBoundaries;
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
                    "Controls: wasd - resize; arrows - move; space - click; esc - stop".to_string(),
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
        views: HashMap<PlacedTetraInBoundaries, View>,
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

        fn update(mut self, item: PlacedTetraInBoundaries) -> Self {
            let idx = self.views.len();

            let sym = char::from_u32(
                // ASCII 'A'
                (65 + idx) as u32,
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
