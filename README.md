# DoNow

**DoNow** is a command and script abbreviator written in Rust. Define named blocks in `~/.donow/DoNow` and run them with a single command:

```
$ donow build
$ donow deploy @env=prod --verbose
```

DoNow is its own DSL with variables, control flow, arrays, templates, functions, and shell integration — no YAML, no TOML, no bash required.

---

## Installation

```bash
git clone https://github.com/yourname/donow.git
cd donow
cargo build --release
cp target/release/donow ~/.local/bin/   # or any $PATH dir
```

## Quickstart

```bash
donow --init
donow --list
donow build
```

`--init` creates `~/.donow/DoNow` with example blocks. Edit the file to define your own:

```
# ~/.donow/DoNow
build:
    echo "compiling..."
    cargo build

deploy:
    echo "deploying to @env"
    git push origin main
```

## Usage

```
donow <block> [@param=value ...] [--flag ...]
donow --list
donow --init
donow --help
```

| Argument | Description |
|---|---|
| `<block>` | Name of the block to execute |
| `@param=value` | Set a CLI parameter (accessible as `@param`) |
| `--flag` | Shorthand for `@flag=true` |
| `--key=value` | Shorthand for `@key=value` |
| `--list` | List all available blocks |
| `--init` | Create `~/.donow/` with example DoNow |
| `--help` | Show this help |

## Examples

```bash
donow build                    # run the 'build' block
donow deploy @env=prod         # @env becomes "prod"
donow test --verbose           # @verbose becomes "true"
donow --list                   # list all blocks
```

## Documentation

| File | Description |
|---|---|
| `MANUAL.md` | Complete language reference |
| `TEMPLATES.md` | Template system (`T<name>`) |
| `FUNCTIONS.md` | Function system and builtins (`F<name>`) |
| `CLASSES.md` | Class system (`C<name>`) |

---

## License

MIT — see [LICENSE.md](LICENSE.md)
