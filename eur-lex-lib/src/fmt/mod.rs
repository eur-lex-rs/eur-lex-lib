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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::loader::load_act;
    use crate::model::{
        Alinea, Annex, AnnexContent, AnnexSection, Article, ArticleContent, Chapter,
        ChapterContents, EnactingTerms, EnactingTermsContent, Preamble, Recital, Section,
        Subparagraph,
    };

    use super::{Format, Render};

    // ── helpers ───────────────────────────────────────────────────────────────

    fn plain_alinea(text: &str) -> Alinea {
        Alinea { content: vec![Subparagraph::Plain(text.to_string())], citations: vec![] }
    }

    fn simple_article() -> Article {
        Article {
            number: "Article 1".to_string(),
            title: Some("Scope".to_string()),
            content: ArticleContent::Alineas(vec![plain_alinea("This regulation applies.")]),
        }
    }

    // ── Act ───────────────────────────────────────────────────────────────────

    #[test]
    /// Act::render dispatches to the correct module: output equals the module's render_act.
    fn act_render_dispatches_correctly() {
        let act = load_act(Path::new("../data/32022R2065")).expect("failed to load DSA");
        assert_eq!(act.render(Format::Markdown), super::markdown::render_act(&act));
        assert_eq!(act.render(Format::Txt),      super::txt::render_act(&act));
        assert_eq!(act.render(Format::Html),     super::html::render_act(&act));
    }

    // ── Recital ───────────────────────────────────────────────────────────────

    #[test]
    /// Recital::render produces the correct format-specific output for all three formats.
    fn recital_render_all_formats() {
        let r = Recital {
            number: "(1)".to_string(),
            text: "Whereas something.".to_string(),
            citations: vec![],
        };
        assert_eq!(r.render(Format::Markdown), "**(1)** Whereas something.\n\n");
        assert_eq!(r.render(Format::Txt),      "(1) Whereas something.\n\n");
        assert_eq!(r.render(Format::Html),     "<p><strong>(1)</strong> Whereas something.</p>\n");
    }

    // ── Preamble ──────────────────────────────────────────────────────────────

    #[test]
    /// Preamble::render produces format-specific visa and recital markup.
    fn preamble_render_all_formats() {
        let p = Preamble {
            init: "THE PARLIAMENT".to_string(),
            visas: vec!["Having regard to the Treaty.".to_string()],
            recitals: vec![Recital {
                number: "(1)".to_string(),
                text: "Whereas something.".to_string(),
                citations: vec![],
            }],
            enacting_formula: "HAS ADOPTED:".to_string(),
        };
        let md = p.render(Format::Markdown);
        assert!(md.contains("> Having regard"), "markdown visas should use blockquote");
        assert!(md.contains("**(1)**"), "markdown recitals should use bold numbers");

        let txt = p.render(Format::Txt);
        assert!(txt.contains("Having regard"));
        assert!(!txt.contains("> "), "txt should not use blockquote syntax");
        assert!(!txt.contains("**"), "txt should not use bold syntax");

        let html = p.render(Format::Html);
        assert!(html.contains("<blockquote>"), "html visas should use blockquote");
        assert!(html.contains("<strong>(1)</strong>"), "html recitals should bold number");
    }

    // ── EnactingTerms ─────────────────────────────────────────────────────────

    #[test]
    /// EnactingTerms::render produces format-specific structural markup.
    fn enacting_terms_render_all_formats() {
        let et = EnactingTerms {
            content: EnactingTermsContent::Articles(vec![simple_article()]),
        };
        assert!(et.render(Format::Markdown).contains("####"));
        assert!(!et.render(Format::Txt).contains('#'));
        assert!(et.render(Format::Html).contains(r#"class="enacting-terms""#));
    }

    // ── Chapter ───────────────────────────────────────────────────────────────

    #[test]
    /// Chapter::render produces format-specific heading markup.
    fn chapter_render_all_formats() {
        let ch = Chapter {
            title: "CHAPTER I".to_string(),
            subtitle: None,
            contents: ChapterContents::Articles(vec![simple_article()]),
        };
        assert!(ch.render(Format::Markdown).contains("## CHAPTER I"));
        assert!(!ch.render(Format::Txt).contains('#'));
        assert!(ch.render(Format::Html).contains(r#"class="chapter""#));
    }

    // ── Section ───────────────────────────────────────────────────────────────

    #[test]
    /// Section::render produces format-specific heading markup.
    fn section_render_all_formats() {
        let sec = Section {
            title: "SECTION 1".to_string(),
            subtitle: None,
            articles: vec![simple_article()],
        };
        assert!(sec.render(Format::Markdown).contains("### SECTION 1"));
        assert!(!sec.render(Format::Txt).contains('#'));
        assert!(sec.render(Format::Html).contains(r#"class="section""#));
    }

    // ── Article ───────────────────────────────────────────────────────────────

    #[test]
    /// Article::render produces format-specific heading markup.
    fn article_render_all_formats() {
        let art = simple_article();
        assert!(art.render(Format::Markdown).contains("#### Article 1 — Scope"));
        assert!(!art.render(Format::Txt).contains('#'));
        assert!(art.render(Format::Html).contains(r#"class="article""#));
    }

    // ── Annex ─────────────────────────────────────────────────────────────────

    #[test]
    /// Annex::render produces format-specific heading markup.
    fn annex_render_all_formats() {
        let annex = Annex {
            number: "ANNEX I".to_string(),
            subtitle: None,
            content: AnnexContent::Paragraphs(vec![Subparagraph::Plain("text".to_string())]),
        };
        assert!(annex.render(Format::Markdown).contains("## ANNEX I"));
        assert!(!annex.render(Format::Txt).contains('#'));
        assert!(annex.render(Format::Html).contains(r#"class="annex""#));
    }

    // ── AnnexSection ──────────────────────────────────────────────────────────

    #[test]
    /// AnnexSection::render produces format-specific heading markup.
    fn annex_section_render_all_formats() {
        let sec = AnnexSection {
            title: "Part 1".to_string(),
            alineas: vec![Subparagraph::Plain("text".to_string())],
            citations: vec![],
        };
        assert!(sec.render(Format::Markdown).contains("### Part 1"));
        assert!(!sec.render(Format::Txt).contains('#'));
        assert!(sec.render(Format::Html).contains(r#"class="annex-section""#));
    }
}
