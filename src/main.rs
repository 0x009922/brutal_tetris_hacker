mod algorithm;
mod app_terminal;
mod tetra;
mod util;

use std::io::stdout;

use clap::Parser;
use crossterm::style::Print;
use crossterm::{cursor, terminal, ExecutableCommand, Result};

use algorithm::CollectStats;

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
