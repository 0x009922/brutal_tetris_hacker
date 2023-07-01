# brutal_tetris_hacker

Brut-forces a NxM field with tetras in order to find optimal placements.

## Build & Run

```bash
cargo build --release
./target/release/brutal-tetris-hacker --help
```

## Usage

**Printed help message:**

```
Usage: brutal-tetris-hacker [OPTIONS]

Options:
      --results-limit <RESULTS_LIMIT>
          The limit of the generated results

      --stdin
          Read the field from STDIN.
          
          Use `--stdin-char-empty` and `--stdin-char-busy` to configure characters recognition.
          Any other characters are not allowed. The length of each line should be fixed.

      --stdin-char-empty <STDIN_CHAR_EMPTY>
          In case of reading the field from STDIN, which character treat as an empty cell
          
          [default: -]

      --stdin-char-busy <STDIN_CHAR_BUSY>
          In case of reading the field from STDIN, which character treat as an unavailable cell
          
          [default: x]

      --output-format <OUTPUT_FORMAT>
          [default: default]
          [possible values: default, json]

  -h, --help
          Print help (see a summary with '-h')
```
