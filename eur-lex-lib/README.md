# eur-lex-lib

A Rust library for parsing EU legislative acts published in
[Formex 4](https://op.europa.eu/en/web/eu-vocabularies/formex) XML format into
typed Rust structs.

Part of the [`eur-lex-rs`](../) workspace.

## Usage

```toml
[dependencies]
eur-lex-lib = { git = "https://github.com/eur-lex-rs/eur-lex-lib" }
```

## Quick start

```rust
use std::path::Path;
use eur_lex_lib::{load_act, Act};
use eur_lex_lib::model::EnactingTermsContent;

let act = load_act(Path::new("/path/to/formex/dir"))?;

println!("{}", act.title());

if let Some(celex) = &act.metadata().celex {
    println!("CELEX: {celex}");
}

if let EnactingTermsContent::Chapters(chapters) = &act.enacting_terms().content {
    for chapter in chapters {
        println!("{}", chapter.title);
    }
}

if let Some(def) = act.definitions().get("AI system") {
    println!("{def}");
}

// Access variant-specific fields (e.g. preamble visas):
if let Act::Regular(reg) = &act {
    println!("{} visas", reg.preamble.visas.len());
}
```

## What it parses

`load_act` takes the path to a Formex publication directory — the kind produced
by [`eur_lex_fetch`](../eur-lex-utils/) or downloaded manually from EUR-Lex —
and returns an `Act`. Both original and consolidated acts are supported.

The parsed output includes:

- **Metadata** — CELEX number, document date, legal value, language,
  institutional authors, Official Journal reference, page numbers.
- **Title** — full title text.
- **Preamble** — for original acts: legal bases (visas) and recitals, each with
  citations to other acts. For consolidated acts: init text and enacting formula
  only.
- **Enacting terms** — the operative body of the act, structured as chapters →
  sections → articles, or a flat list of articles. Article content takes one of
  three forms: numbered paragraphs (`<PARAG>`), bare alineas (`<ALINEA>`), or
  titled subdivisions (`<SUBDIV>`). Each level can contain plain text, lists
  (with optional nesting), and tables.
- **Annexes** — either section-based (with titled `<GR.SEQ>` sub-divisions) or
  paragraph-based (flat numbered items, plain text, and tables).
- **Definitions** — a flat `HashMap<String, String>` extracted from any article
  titled "Definitions".

## Data model

The top-level type is `Act`, an untagged enum:

```rust
pub enum Act {
    Regular(RegularAct),
    Consolidated(ConsolidatedAct),
}
```

Convenience methods on `Act` work for both variants:

| Method | Returns |
|---|---|
| `act.title()` | `&str` |
| `act.metadata()` | `&Metadata` |
| `act.enacting_terms()` | `&EnactingTerms` |
| `act.annexes()` | `&[Annex]` |
| `act.definitions()` | `&HashMap<String, String>` |

The full model is in `eur_lex_lib::model`. See `cargo doc --open` for the
complete API.

## Errors

All errors are returned as `eur_lex_lib::error::Error`:

```rust
pub enum Error {
    Xml(roxmltree::Error),
    Io { path: String, source: std::io::Error },
    MissingElement(&'static str),
}
```

## Limitations

- Only the English Formex 4 format is tested.
- Footnote bodies (`<NOTE>`) are dropped during text extraction; only the
  surrounding sentence is preserved.
- Formex elements not covered by the model (e.g. images, mathematical formulae)
  are silently reduced to their plain-text content where possible; structure is
  lost.
