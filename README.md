[![codecov](https://codecov.io/github/eur-lex-rs/eur-lex-lib/graph/badge.svg?token=6TH7gBGvLu)](https://codecov.io/github/eur-lex-rs/eur-lex-lib)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Feur-lex-rs%2Feur-lex-lib.svg?type=shield&issueType=license)](https://app.fossa.com/projects/git%2Bgithub.com%2Feur-lex-rs%2Feur-lex-lib?ref=badge_shield&issueType=license)

# eur-lex-rs

A Rust workspace for working with EU legislative acts published in
[Formex 4](https://op.europa.eu/en/web/eu-vocabularies/formex) XML format.

## Crates

| Crate | Description |
|---|---|
| [`eur-lex-lib`](eur-lex-lib/) | Library — parses Formex XML into typed Rust structs |
| [`eur-lex-utils`](eur-lex-utils/) | CLI tools — fetch and convert acts from EUR-Lex |

The library extracts the full document structure: bibliographic metadata (CELEX
number, document date, legal value, Official Journal reference, authors),
title, preamble (legal bases and recitals), enacting terms (chapters, sections,
articles, and nested lists), tables, annexes, and a flat definitions map when
the act contains a Definitions article. Both original and consolidated acts are
supported.

---

## European legislative acts

The European Union produces two main types of binding secondary legislation.

**Regulations** are directly applicable across all member states from the
moment they enter into force. No national transposition is needed; a regulation
has the same legal force as national law in every member state the day it is
published in the Official Journal.

**Directives** are binding as to the result to be achieved but leave each
member state free to choose the form and methods. They must be transposed into
national law within a deadline set by the directive itself. The national
transposition laws differ from country to country, but the outcome must meet
the directive's requirements.

**Consolidated versions** are unofficial editorial compilations produced by the
Publications Office. They integrate all subsequent amendments into the original
text so that readers see the current wording in a single document, without
having to cross-reference a chain of amending acts. Consolidated versions have
no independent legal force — only the original act and its amending acts are
legally binding — but they are the most convenient starting point for reading
the current state of a piece of legislation.

---

## EUR-Lex, Cellar, and CELEX numbers

[EUR-Lex](https://eur-lex.europa.eu) is the official portal for EU law,
providing free access to the Official Journal and to the full text of all EU
legislative acts.

The [Cellar](https://op.europa.eu/en/web/cellar) content repository, maintained
by the Publications Office of the European Union, is the underlying store from
which EUR-Lex serves its content. Formex XML files are available directly from
Cellar without authentication.

### CELEX numbers

Every EU legal act has a unique CELEX identifier. The format is:

```
3 YYYY T NNNN
│  │   │  └─ sequential number within the year
│  │   └─ document type: R = Regulation, L = Directive
│  └─ year of publication
└─ sector: 3 = secondary legislation
```

Examples — all eight acts included as test fixtures in this repository:

| Act | CELEX | Format |
|---|---|---|
| EU AI Act (2024) | `32024R1689` | Original regulation |
| Digital Services Act (2022) | `32022R2065` | Original regulation |
| EU Trade Mark Regulation (2017) | `32017R1001` | Original regulation |
| Copyright in the Digital Single Market Directive (2019) | `32019L0790` | Original directive |
| Anti-Dumping Regulation (2016) | `32016R1036` | Original regulation |
| Anti-Dumping Regulation — consolidated (2018) | `02016R1036-20180608` | Consolidated regulation |
| REACH Regulation (2006) | `32006R1907` | Consolidated regulation |
| Consumer Rights Directive (2011) | `32011L0083` | Consolidated directive |

The CELEX number appears in every EUR-Lex URL, e.g.:
`https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R1689`

---

## Building

```bash
cargo build --release
```

Two binaries are produced under `target/release/`: `eur_lex_fetch` and
`eur_lex_loader`. See [`eur-lex-utils/`](eur-lex-utils/) for usage.

---

## Running the tests

```bash
cargo test
```

Unit tests live alongside their source modules in `eur-lex-lib/`. Integration
tests in `eur-lex-lib/tests/` validate the full parse of eight EU legislative
acts against known structural counts:

| File | Act | Format | Articles | Recitals | Definitions | Tables |
|---|---|---|---|---|---|---|
| `eu_ai_act.rs` | EU AI Act (`32024R1689`) | Original | 113 | 180 | 68 | — |
| `dsa.rs` | Digital Services Act (`32022R2065`) | Original | 93 | 156 | 27 | — |
| `dsma.rs` | Copyright in the Digital Single Market (`32019L0790`) | Original | 32 | 86 | 6 | — |
| `trademark_act.rs` | EU Trade Mark Regulation (`32017R1001`) | Original | 212 | 48 | — | — |
| `anti_dumping.rs` | Anti-Dumping Regulation (`32016R1036`) | Original | 25 | 32 | — | — |
| `anti_dumping_consolidated.rs` | Anti-Dumping Regulation (`02016R1036-20180608`) | Consolidated | — | — | — | — |
| `reach.rs` | REACH Regulation (`32006R1907`) | Consolidated | 141 | — | — | ✓ |
| `consumer_directive.rs` | Consumer Rights Directive (`32011L0083`) | Consolidated | 36 | — | — | ✓ |

The table tests (✓) verify that `Subparagraph::Table` values are produced for
annex tables in both Formex table encodings:

- **REACH** (ANNEX IV) — a bare `<TBL>` element sitting directly inside a
  `<CONTENTS>` block, with no wrapping `<GR.TBL>`.
- **Consumer Rights Directive** (ANNEX II) — a correlation table wrapped in a
  `<GR.TBL>` element, which carries an optional title above the table.

Test fixtures are in the `data/` directory at the workspace root.

---

## Limitations

- Only the English Formex 4 format is tested.
- Footnote bodies (`<NOTE>`) are dropped during text extraction; only the
  surrounding sentence is preserved.
- Formex elements not covered by the model (e.g. images, mathematical formulae)
  are silently reduced to their plain-text content where possible; structure is
  lost.


## License
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Feur-lex-rs%2Feur-lex-lib.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2Feur-lex-rs%2Feur-lex-lib?ref=badge_large)