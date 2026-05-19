//! Output formatters for [`crate::model::Act`] and its sub-types, enabled by the `fmt` Cargo feature.
//!
//! Three formats are provided:
//!
//! - [`markdown`] — GitHub-flavoured Markdown
//! - [`txt`] — Plain text with no markup syntax
//! - [`html`] — Self-contained HTML5 document
//!
//! # Usage
//!
//! ```no_run
//! use std::path::Path;
//! use eur_lex_lib::{load_act, fmt::{Format, Render}};
//!
//! let act = load_act(Path::new("/path/to/dir")).unwrap();
//! println!("{}", act.render(Format::Markdown));
//!
//! // Sub-types are renderable too:
//! if let eur_lex_lib::Act::Regular(reg) = &act {
//!     for recital in &reg.preamble.recitals {
//!         print!("{}", recital.render(Format::Txt));
//!     }
//! }
//! ```

pub mod html;
pub mod markdown;
pub mod txt;

use crate::model::{
    Act, Annex, AnnexSection, Article, Chapter, EnactingTerms, Preamble, Recital, Section,
};

/// The three supported output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// GitHub-flavoured Markdown.
    Markdown,
    /// Plain text with no markup syntax.
    Txt,
    /// Self-contained HTML5 document.
    Html,
}

/// Render a value to a human-readable string in the given [`Format`].
pub trait Render {
    fn render(&self, format: Format) -> String;
}

impl Render for Act {
    fn render(&self, format: Format) -> String {
        match format {
            Format::Markdown => markdown::render_act(self),
            Format::Txt      => txt::render_act(self),
            Format::Html     => html::render_act(self),
        }
    }
}

impl Render for Preamble {
    fn render(&self, format: Format) -> String {
        match format {
            Format::Markdown => markdown::render_preamble(self),
            Format::Txt      => txt::render_preamble(self),
            Format::Html     => html::render_preamble_section(self),
        }
    }
}

impl Render for Recital {
    fn render(&self, format: Format) -> String {
        match format {
            Format::Markdown => markdown::render_recital(self),
            Format::Txt      => txt::render_recital(self),
            Format::Html     => html::render_recital(self),
        }
    }
}

impl Render for EnactingTerms {
    fn render(&self, format: Format) -> String {
        match format {
            Format::Markdown => markdown::render_enacting_terms(self),
            Format::Txt      => txt::render_enacting_terms(self),
            Format::Html     => html::render_enacting_terms(self),
        }
    }
}

impl Render for Chapter {
    fn render(&self, format: Format) -> String {
        match format {
            Format::Markdown => markdown::render_chapter(self),
            Format::Txt      => txt::render_chapter(self),
            Format::Html     => html::render_chapter(self),
        }
    }
}

impl Render for Section {
    fn render(&self, format: Format) -> String {
        match format {
            Format::Markdown => markdown::render_section(self),
            Format::Txt      => txt::render_section(self),
            Format::Html     => html::render_section(self),
        }
    }
}

impl Render for Article {
    fn render(&self, format: Format) -> String {
        match format {
            Format::Markdown => markdown::render_article(self),
            Format::Txt      => txt::render_article(self),
            Format::Html     => html::render_article(self),
        }
    }
}

impl Render for Annex {
    fn render(&self, format: Format) -> String {
        match format {
            Format::Markdown => markdown::render_annex(self),
            Format::Txt      => txt::render_annex(self),
            Format::Html     => html::render_annex(self),
        }
    }
}

impl Render for AnnexSection {
    fn render(&self, format: Format) -> String {
        match format {
            Format::Markdown => markdown::render_annex_section(self),
            Format::Txt      => txt::render_annex_section(self),
            Format::Html     => html::render_annex_section(self),
        }
    }
}
