# eur-lex-utils

Command-line tools for fetching and converting EU legislative acts from
[EUR-Lex](https://eur-lex.europa.eu) via the Cellar repository.

Part of the [`eur-lex-rs`](../) workspace. Parsing is handled by
[`eur-lex-lib`](../eur-lex-lib/).

## Tools

| Binary | Description |
|---|---|
| `eur_lex_fetch` | Download a Formex publication by CELEX number |
| `eur_lex_loader` | Parse a local Formex directory and output JSON |

## Building

```bash
cargo build --release -p eur-lex-utils
```

Binaries are produced under `target/release/`.

---

## `eur_lex_fetch`

Downloads a Formex ZIP archive from the EUR-Lex Cellar API by CELEX number,
extracts it into a local directory, then prints the act title to stdout so you
can confirm the correct act was retrieved. Progress messages go to stderr.

```
eur_lex_fetch [OPTIONS] <CELEX> <DIR>

Arguments:
  <CELEX>  CELEX number of the act to fetch (e.g. 32024R1689)
  <DIR>    Directory where the Formex files will be extracted

Options:
  -l, --lang <LANG>  Language code (ISO 639-2/B, e.g. eng, fra, deu) [default: eng]
  -h, --help         Print help
  -V, --version      Print version
```

```bash
# Fetch the EU AI Act in English
eur_lex_fetch 32024R1689 data/32024R1689
# → Fetching 32024R1689 (eng)...
# → Extracted to data/32024R1689
# → Regulation (EU) 2024/1689 of the European Parliament …

# Fetch the REACH Regulation in French
eur_lex_fetch 32006R1907 data/32006R1907_fr --lang fra
```

The extracted directory will contain several `.fmx.xml` files:

| Filename pattern | Content |
|---|---|
| `*.000101.fmx.xml` | Main act (title, preamble, enacting terms) |
| `*.012401.fmx.xml` and above | Annexes, one file each (original acts) |
| `*.doc.fmx.xml` | Registry listing all files in order |
| `*.toc.fmx.xml` | Table of contents (not used by this tool) |

Consolidated acts embed their annexes inline in the main file; no separate
annex files are produced.

> **Rate limiting**: keep concurrent requests below 5 per IP address.

---

## `eur_lex_loader`

Parses a local Formex directory (previously fetched with `eur_lex_fetch` or
downloaded manually) and writes the act as JSON to stdout or a file. Can also
fetch directly from Cellar without saving the Formex files locally.

```
eur_lex_loader [OPTIONS] [DIR]

Arguments:
  [DIR]  Path to a local Formex act directory

Options:
  -c, --celex <CELEX>  Fetch from EUR-Lex Cellar by CELEX number (e.g. 32022R2065)
  -o, --output <FILE>  Write JSON output to FILE instead of stdout
      --compact        Output compact JSON (default: pretty-printed)
  -h, --help           Print help
  -V, --version        Print version
```

`DIR` and `--celex` are mutually exclusive. Running with no arguments prints help.

```bash
# Fetch the DSA directly from EUR-Lex and pretty-print to stdout
eur_lex_loader -c 32022R2065

# Fetch the EU AI Act and write compact JSON to a file
eur_lex_loader -c 32024R1689 --compact --output ai_act.json

# Parse a previously downloaded act
eur_lex_loader data/32024R1689

# Write compact JSON to a file
eur_lex_loader data/32024R1689 --compact --output ai_act.json

# Pipe pretty-printed JSON into jq
eur_lex_loader data/32024R1689 | jq '.preamble.recitals | length'
```

### Output format

The tool outputs a single JSON object. The shape depends on whether the act is
an original or a consolidated version.

**Original acts** include a full preamble:

```jsonc
{
  "metadata": {
    "celex": "32024R1689",
    "document_date": "20240613",
    "legal_value": "REG",
    "language": "EN",
    "authors": ["PE", "CS"],
    "eea_relevant": true,
    "official_journal": { "collection": "L", "number": "1689", "date": "20240712", "language": "EN" },
    "page_first": 1,
    "page_last": 144,
    "page_total": 144
  },
  "title": "Regulation (EU) 2024/1689 …",
  "preamble": {
    "init": "THE EUROPEAN PARLIAMENT AND THE COUNCIL …",
    "visas": ["Having regard to …", "…"],
    "recitals": [
      { "number": "(1)", "text": "The purpose of this Regulation …" }
    ],
    "enacting_formula": "HAVE ADOPTED THIS REGULATION:"
  },
  "enacting_terms": { "…": "…" },
  "annexes": [ "…" ],
  "definitions": { "…": "…" }
}
```

**Consolidated acts** have a slim preamble with no visas or recitals:

```jsonc
{
  "metadata": { "celex": "32006R1907", "legal_value": "REG", "…": "…" },
  "title": "Regulation (EC) No 1907/2006 …",
  "preamble": {
    "init": "THE EUROPEAN PARLIAMENT AND THE COUNCIL …",
    "enacting_formula": "HAVE ADOPTED THIS REGULATION:"
  },
  "enacting_terms": { "…": "…" },
  "annexes": [ "…" ]
}
```

**Full output shape:**

```jsonc
{
  "metadata": {
    "celex": "32024R1689",          // CELEX identifier
    "document_date": "20240613",    // signing/adoption date, YYYYMMDD
    "legal_value": "REG",           // "REG" | "DIR" | "DEC" | …
    "language": "EN",               // document language code
    "authors": ["PE", "CS"],        // institutional authors
    "eea_relevant": true,           // EEA relevance flag
    "official_journal": {
      "collection": "L",            // OJ series ("L" or "C")
      "number": "1689",             // OJ issue number
      "date": "20240712",           // publication date, YYYYMMDD
      "language": "EN"              // language edition
    },
    "page_first": 1,
    "page_last": 144,
    "page_total": 144,
    "prod_id": "20240610001",       // internal production ID (absent in older acts)
    "fin_id": "789012"              // internal final ID (absent in older acts)
  },

  "title": "Regulation (EU) 2024/1689 …",

  "preamble": {
    "init": "THE EUROPEAN PARLIAMENT AND THE COUNCIL …",
    "visas": ["Having regard to …", "…"],
    "recitals": [
      { "number": "(1)", "text": "The purpose of this Regulation …" }
    ],
    "enacting_formula": "HAVE ADOPTED THIS REGULATION:"
  },

  // enacting_terms.content is one of two variants:
  // - "Chapters" for acts divided into chapters (most acts)
  // - "Articles" for flat acts whose articles sit directly in <ENACTING.TERMS>
  "enacting_terms": {
    "content": {
      "Chapters": [
        {
          "title": "CHAPTER I",
          "subtitle": "General provisions",
          // A chapter contains either sections or articles directly:
          "contents": {
            "Articles": [
              {
                "number": "Article 1",
                "title": "Subject matter",
                // Article content is one of three variants:
                // 1. Paragraphs — numbered <PARAG> wrappers (most common)
                "content": { "Paragraphs": [
                  {
                    "number": "1.",
                    "alineas": [
                      // A plain text block:
                      { "Plain": "The purpose of this Regulation …" },
                      // A <P> intro + <LIST> collapsed into a single List block:
                      { "List": {
                          "list_type": "alpha",
                          "intro": "The following practices shall be prohibited:",
                          "items": [
                            { "number": 1, "content": { "Text": "…" } },
                            // An item with a nested list:
                            { "number": 2, "content": { "List": {
                                "list_type": "roman",
                                "intro": "…",
                                "items": [
                                  { "number": 1, "content": { "Text": "…" } }
                                ]
                            } } }
                          ]
                      } },
                      // A table parsed from <GR.TBL> or a bare <TBL> element:
                      { "Table": {
                          "col_count": 3,
                          "title": "Correlation table",   // omitted when absent
                          "row_count": 2,
                          "rows": [
                            { "is_header": true, "cell_count": 3,
                              "cells": [
                                { "text": "Old directive", "is_header": true },
                                { "text": "New directive", "is_header": true },
                                { "text": "Remarks",       "is_header": true }
                              ] },
                            { "cell_count": 3,
                              "cells": [
                                { "text": "Article 1" },
                                { "text": "Article 3" },
                                { "text": "" }
                              ] }
                          ]
                      } }
                    ]
                  }
                ] }
                // 2. Alineas — bare <ALINEA> children, no <PARAG> wrapper:
                // "content": { "Alineas": [ { "content": [ { "Plain": "…" } ] } ] }
                // 3. Subdivisions — <SUBDIV> thematic groups, each with a title:
                // "content": { "Subdivisions": [
                //   { "title": "NORMAL VALUE",
                //     "content": { "Paragraphs": [ { "number": "1.", "alineas": [ … ] } ] } }
                // ] }
              }
            ]
          }
        }
      ]
      // For a flat act (no chapters):
      // "content": { "Articles": [ { "number": "Article 1", "…": "…" } ] }
    }
  },

  "annexes": [
    {
      "number": "ANNEX I",
      "subtitle": "List of harmonised standards …",
      // Annexes with titled sub-divisions use Sections:
      "content": {
        "Sections": [
          {
            "title": "Part A",
            "alineas": [
              { "Plain": "…" },
              { "List": { "list_type": "alpha", "intro": "…",
                          "items": [ { "number": 1, "content": { "Text": "…" } } ] } },
              { "Table": { "col_count": 2, "row_count": 1,
                           "rows": [ { "cell_count": 2,
                                       "cells": [ { "text": "…" }, { "text": "…" } ] } ] } }
            ]
          }
        ]
      }
      // Annexes with flat numbered items use Paragraphs:
      // "content": {
      //   "Paragraphs": [
      //     { "Plain": "Introductory text …" },
      //     { "Numbered": { "number": "1.", "alineas": [ { "Plain": "…" } ] } },
      //     { "Table": { "col_count": 3, "row_count": 5, "rows": [ "…" ] } }
      //   ]
      // }
    }
  ],

  // Present only when the act contains a Definitions article.
  // Key: defined term. Value: full definition text as it appears in the act.
  "definitions": {
    "AI system": "\u201CAI system\u201D means a machine-based system …",
    "high-risk AI system": "\u201Chigh-risk AI system\u201D means …"
  }
}
```

`list_type` is omitted from `List` when the `<LIST>` element carries no `TYPE`
attribute. `title` is omitted from `Table` when the `<TBL>` element has no
`<TITLE>`. `is_header` is omitted from `Row` and `Cell` when `false`.
`definitions` is omitted when the act has no Definitions article.

In `metadata`, all fields except `eea_relevant` are optional and omitted from
the JSON when absent. `prod_id` and `fin_id` are absent in older Formex files.
`authors` is omitted when empty.
