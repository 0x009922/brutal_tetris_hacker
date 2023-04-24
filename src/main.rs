mod algorithm;
mod app_terminal;
mod parse_field;
mod tetra;
mod util;

use std::io::stdout;
use std::num::NonZeroUsize;

use clap::Parser;
use crossterm::style::Print;
use crossterm::{cursor, terminal, ExecutableCommand};

use algorithm::CollectStats;

#[derive(Parser)]
struct Args {
    /// The limit of the generated results.
    #[arg(long)]
    results_limit: Option<NonZeroUsize>,
    /// Read the field from STDIN.
    ///
    /// Use `--stdin-char-empty` and `--stdin-char-busy` to configure characters recognition. Any
    /// other characters are not allowed. The length of each line should be fixed.
    #[arg(long)]
    stdin: bool,
    /// In case of reading the field from STDIN, which character treat as an empty cell
    #[arg(long, default_value_t = '-')]
    stdin_char_empty: char,
    /// In case of reading the field from STDIN, which character treat as an unavailable cell
    #[arg(long, default_value_t = 'x')]
    stdin_char_busy: char,
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

fn main() -> Result<(), color_eyre::Report> {
    let args = Args::parse();

    let conf = {
        let mut conf = if args.stdin {
            use std::io::{self, Read};

            let mut input = String::new();
            io::stdin().read_to_string(&mut input).unwrap();

            parse_field::Parser::new(args.stdin_char_empty, args.stdin_char_busy)
                .parse(input)
                .map(|parse_field::ParsedField { size, unavailable }| {
                    algorithm::Configuration::new(size, unavailable)
                })
                .map_err(|err| color_eyre::eyre::eyre!("Failed to parse field from STDIN: {err}"))?
        } else {
            app_terminal::live_configuration::State::new(4, 4)
                .live()?
                .into_configuration()
        };

        if let Some(limit) = args.results_limit {
            conf = conf.with_results_limit(limit);
        }
        conf
    };

    conf.print_field()?;

    let mut stats = Stats::new();
    let placements = conf.run(&mut stats);
    let elapsed = stats.start.elapsed();

    for item in &placements {
        app_terminal::report_placement(item, &conf)?;
        stdout().execute(Print("\n"))?;
    }

    stdout().execute(Print(format!(
        "\n  Found placements: {} (time: {:.2?})\n",
        placements.len(),
        elapsed
    )))?;

    Ok(())
}
