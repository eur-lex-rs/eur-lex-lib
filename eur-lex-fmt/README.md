# eur-lex-fmt

Print EU legislative acts in human-readable formats — Markdown or plain text.

Part of the [`eur-lex-rs`](../) workspace. Parsing is handled by
[`eur-lex-lib`](../eur-lex-lib/).

## Building

```bash
cargo build --release -p eur-lex-fmt
```

The `eur_lex_print` binary is produced under `target/release/`.

---

## `eur_lex_print`

Renders a Formex act as Markdown (default) or plain text. Pass a local
directory path, or use `--celex` to fetch directly from EUR-Lex Cellar.

```
eur_lex_print [OPTIONS] [DIR]

Arguments:
  [DIR]  Path to a local Formex act directory

Options:
  -c, --celex <CELEX>    Fetch from EUR-Lex Cellar by CELEX number (e.g. 32024R1689)
  -f, --format <FORMAT>  Output format [default: md] [possible values: md, txt]
  -h, --help             Print help
  -V, --version          Print version
```

`DIR` and `--celex` are mutually exclusive. Running with no arguments prints help.

---

### Markdown output (default)

```bash
# Render a previously downloaded act
eur_lex_print data/32024R1689

# Fetch and render directly from EUR-Lex
eur_lex_print --celex 32024R1689

# Pipe to a pager
eur_lex_print --celex 32022R2065 | less
```

Produces GitHub-flavoured Markdown with `#` headings, `>` blockquotes for
visas, bold recital numbers, GFM pipe tables, and indented nested lists.
Suitable for display in Markdown viewers or conversion to HTML.

---

### Plain text output

```bash
# Render as plain text
eur_lex_print --format txt data/32024R1689

# Fetch and render as plain text
eur_lex_print --format txt --celex 32022R2065 | less
```

Produces plain text with no markup syntax — no `#`, `**`, `>`, or `|`
characters. Tables are column-aligned; major section separators use lines of
U+2500 (`─`) characters. Suitable for terminal output, text-only pipelines, or
tools that do not understand Markdown.

---

> **Rate limiting**: the EUR-Lex Cellar API asks that concurrent requests be
> kept below 5 per IP address.
