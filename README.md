# php_formatter

Command-line PHP formatter written in Rust. Used as the formatting engine for the [PHP Formatter](../vscode-extension) VS Code extension.

## Build

Requires Rust stable 1.70+.

```bash
cargo build --release
# binary: target/release/php_formatter  (or .exe on Windows)
```

No C compiler or external toolchain needed — pure Rust, single dependency (`json` for the JSON protocol).

## Usage

### Legacy mode (stdin → stdout)

Pipe PHP source in, get formatted source out:

```bash
php_formatter < input.php > output.php

# or pass a file path directly
php_formatter src/Controller.php
```

Config is auto-discovered: the binary walks up from the file path looking for `.phpfmt.toml`.

### JSON mode (`--json`)

Send a JSON request on stdin, receive a JSON response on stdout. Used by the VS Code extension.

```bash
echo '{"command":"format","source":"<?php $a=1;\n$foo=2;\n","file_path":"/project/src/Foo.php"}' \
  | php_formatter --json
```

#### Request

```json
{
  "command": "format",
  "source": "<?php ...",
  "file_path": "/absolute/path/to/file.php"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `command` | `"format"` \| `"check"` | yes | `format` returns formatted source; `check` returns diagnostics without modifying |
| `source` | string | yes | Full PHP source to process |
| `file_path` | string | no | Used to discover `.phpfmt.toml` by walking up the directory tree |

#### Response

```json
{
  "ok": true,
  "command": "format",
  "formatted": "<?php ...",
  "changed": true,
  "timing_ms": 4,
  "diagnostics": []
}
```

| Field | Type | Description |
|-------|------|-------------|
| `ok` | bool | `false` only on a hard error (binary will also set `error`) |
| `formatted` | string \| null | Present on `format`; absent on `check` |
| `changed` | bool | Whether source differs from formatted output |
| `timing_ms` | number | Wall time for the format pass |
| `diagnostics` | array | Non-empty on `check`; each item has `line`, `col`, `end_line`, `end_col`, `message`, `code`, `severity`, optional `fix` |
| `error` | string \| null | Human-readable error message when `ok` is false |

## Configuration

Place `.phpfmt.toml` in the project root (or any parent directory):

```toml
[style]
indent_size = 4       # spaces per level

[align]
assignments     = true   # align = in consecutive $var blocks
fat_arrows      = true   # align => in array / match blocks
inline_comments = true   # align // comments on consecutive lines

[on_save]
enabled = false
```

Defaults are used for any key not present.

## Format rules

| Rule | What it does |
|------|-------------|
| **Indent** | Converts tabs to spaces; normalises indent depth |
| **Spacing** | Ensures single space around binary operators and after commas |
| **Align `=`** | Groups consecutive `$var = …` lines and aligns `=` to the same column |
| **Align `=>`** | Groups consecutive `'key' => value` lines and aligns `=>` |
| **Align comments** | Groups lines with trailing `//` comments and aligns the `//` |
| **Frozen regions** | Lines between `@fmt-off` / `@fmt-on` (or `@formatter:off` / `@formatter:on`) are passed through unchanged |

## Project structure

```
src/
├── main.rs             Entry point — argument parsing, legacy vs. JSON dispatch
├── formatter.rs        Line struct, format pipeline, per-line detection helpers
├── config.rs           Config struct, .phpfmt.toml discovery and parsing
├── protocol.rs         BinaryRequest / BinaryResponse (manual JSON, no proc-macros)
└── rules/
    ├── mod.rs
    ├── align.rs            = alignment pass
    ├── align_arrows.rs     => alignment pass
    ├── align_comments.rs   Inline comment alignment pass
    ├── frozen.rs           @fmt-off / @fmt-on region splitting
    ├── indent.rs           Indentation normalisation
    └── spacing.rs          Operator and comma spacing
```

The format pipeline for each non-frozen region:

1. Split source into lines
2. Per line: normalise indent → normalise spacing → split off inline comment → detect `=` / `=>` positions
3. Alignment pass: `=` → `=>` → inline comments (in that order; comment pass uses rendered lengths from the previous two)
4. Render all lines and join

## License

MIT
