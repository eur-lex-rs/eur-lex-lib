//! Parser for Formex act XML files (`<ACT>` and `<CONS.ACT>` roots).
//!
//! The two public functions, [`parse_regular_act`] and [`parse_consolidated_act`],
//! each parse one `.fmx.xml` file and return `(title, preamble, enacting_terms)`.
//! The caller ([`crate::loader`]) assembles those parts with annex data into a
//! complete [`crate::model::Act`].

use roxmltree::{Document, Node};

use super::text::extract_text;
use super::{child, extract_citations, parse_block_children};
use crate::error::Error;
use crate::model::*;

/// Parses a regular Formex act XML string (`<ACT>` root) into its three parts.
///
/// Returns `(title, preamble, enacting_terms)`. The caller assembles these with
/// parsed annex files to build a [`crate::model::RegularAct`].
///
/// # Errors
///
/// Returns [`crate::error::Error::Xml`] for malformed XML and
/// [`crate::error::Error::MissingElement`] if `<TITLE>`, `<PREAMBLE>`, or
/// `<ENACTING.TERMS>` are absent from the document root.
pub fn parse_regular_act(xml: &str) -> Result<(String, Preamble, EnactingTerms), Error> {
    let doc = Document::parse(xml)?;
    let root = doc.root_element();
    let title = parse_title(child(root, "TITLE")?)?;
    let preamble = parse_preamble(child(root, "PREAMBLE")?)?;
    let enacting_terms = parse_enacting_terms(child(root, "ENACTING.TERMS")?)?;
    Ok((title, preamble, enacting_terms))
}

/// Parses a consolidated Formex act XML string (`<CONS.ACT>` root) into its three parts.
///
/// Returns `(title, preamble, enacting_terms)`. The caller assembles these with
/// inline `<CONS.ANNEX>` elements to build a [`crate::model::ConsolidatedAct`].
///
/// # Errors
///
/// Returns [`crate::error::Error::Xml`] for malformed XML and
/// [`crate::error::Error::MissingElement`] if `<CONS.DOC>`, `<TITLE>`,
/// `<PREAMBLE>`, or `<ENACTING.TERMS>` are absent.
pub fn parse_consolidated_act(
    xml: &str,
) -> Result<(String, ConsolidatedPreamble, EnactingTerms), Error> {
    let doc = Document::parse(xml)?;
    let root = doc.root_element();
    let content = child(root, "CONS.DOC")?;
    let title = parse_title(child(content, "TITLE")?)?;
    let preamble = parse_consolidated_preamble(child(content, "PREAMBLE")?)?;
    let enacting_terms = parse_enacting_terms(child(content, "ENACTING.TERMS")?)?;
    Ok((title, preamble, enacting_terms))
}

/// Joins all `<P>` children of `<TITLE><TI>` into a single space-separated string.
fn parse_title(node: Node) -> Result<String, Error> {
    let ti = child(node, "TI")?;
    let parts: Vec<String> = ti
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "P")
        .map(extract_text)
        .collect();
    Ok(parts.join(" "))
}

/// Extracts all four structural parts of a `<PREAMBLE>` element.
fn parse_preamble(node: Node) -> Result<Preamble, Error> {
    let init = node
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "PREAMBLE.INIT")
        .map(extract_text)
        .unwrap_or_default();

    let visas = node
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "GR.VISA")
        .map(|gr| {
            gr.children()
                .filter(|n| n.is_element() && n.tag_name().name() == "VISA")
                .map(extract_text)
                .collect()
        })
        .unwrap_or_default();

    let recitals = node
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "GR.CONSID")
        .map(|gr| {
            gr.children()
                .filter(|n| n.is_element() && n.tag_name().name() == "CONSID")
                .map(parse_recital)
                .collect()
        })
        .unwrap_or_default();

    let enacting_formula = node
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "PREAMBLE.FINAL")
        .map(extract_text)
        .unwrap_or_default();

    Ok(Preamble {
        init,
        visas,
        recitals,
        enacting_formula,
    })
}

/// Extracts the two fields of a consolidated preamble (no visas or recitals).
fn parse_consolidated_preamble(node: Node) -> Result<ConsolidatedPreamble, Error> {
    let init = node
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "PREAMBLE.INIT")
        .map(extract_text)
        .unwrap_or_default();
    let enacting_formula = node
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "PREAMBLE.FINAL")
        .map(extract_text)
        .unwrap_or_default();
    Ok(ConsolidatedPreamble {
        init,
        enacting_formula,
    })
}

/// Extracts the number and text from a single `<CONSID>` recital element.
///
/// Standard recitals use an `<NP>` wrapper with `<NO.P>` and `<TXT>` children.
/// If no `<NP>` is found the entire element is rendered as plain text with an
/// empty number.
fn parse_recital(node: Node) -> Recital {
    let np = node
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "NP");

    let (number, text) = if let Some(np) = np {
        let number = np
            .children()
            .find(|n| n.is_element() && n.tag_name().name() == "NO.P")
            .map(extract_text)
            .unwrap_or_default();
        let text = np
            .children()
            .find(|n| n.is_element() && n.tag_name().name() == "TXT")
            .map(extract_text)
            .unwrap_or_default();
        (number, text)
    } else {
        (String::new(), extract_text(node))
    };

    let citations = extract_citations(node);
    Recital {
        number,
        text,
        citations,
    }
}

/// Parses `<ENACTING.TERMS>` into an [`EnactingTerms`].
///
/// If `<DIVISION>` children are present they become chapters; otherwise
/// `<ARTICLE>` children are collected directly (flat acts with no chapter structure).
fn parse_enacting_terms(node: Node) -> Result<EnactingTerms, Error> {
    let division_nodes: Vec<_> = node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "DIVISION")
        .collect();

    let content = if !division_nodes.is_empty() {
        let chapters = division_nodes
            .into_iter()
            .map(parse_chapter)
            .collect::<Result<Vec<_>, _>>()?;
        EnactingTermsContent::Chapters(chapters)
    } else {
        let articles = node
            .children()
            .filter(|n| n.is_element() && n.tag_name().name() == "ARTICLE")
            .map(parse_article)
            .collect::<Result<Vec<_>, _>>()?;
        EnactingTermsContent::Articles(articles)
    };

    Ok(EnactingTerms { content })
}

/// Parses a top-level `<DIVISION>` as a chapter.
///
/// If the division contains child `<DIVISION>` elements those are parsed as
/// sections; otherwise its `<ARTICLE>` children are parsed directly.
fn parse_chapter(node: Node) -> Result<Chapter, Error> {
    let title_node = child(node, "TITLE")?;
    let title = extract_text(child(title_node, "TI")?);
    let subtitle = title_node
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "STI")
        .map(extract_text);

    let sub_divisions: Vec<_> = node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "DIVISION")
        .collect();

    let contents = if !sub_divisions.is_empty() {
        let sections = sub_divisions
            .into_iter()
            .map(parse_section)
            .collect::<Result<Vec<_>, _>>()?;
        ChapterContents::Sections(sections)
    } else {
        let articles = node
            .children()
            .filter(|n| n.is_element() && n.tag_name().name() == "ARTICLE")
            .map(parse_article)
            .collect::<Result<Vec<_>, _>>()?;
        ChapterContents::Articles(articles)
    };

    Ok(Chapter {
        title,
        subtitle,
        contents,
    })
}

/// Parses a nested `<DIVISION>` as a section (articles only, no further nesting).
fn parse_section(node: Node) -> Result<Section, Error> {
    let title_node = child(node, "TITLE")?;
    let title = extract_text(child(title_node, "TI")?);
    let subtitle = title_node
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "STI")
        .map(extract_text);

    let articles = node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "ARTICLE")
        .map(parse_article)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Section {
        title,
        subtitle,
        articles,
    })
}

/// Parses an `<ARTICLE>` element.
///
/// When `<PARAG>` wrappers are present each becomes a [`LegalParagraph`].
/// Some articles (e.g. Article 113 of the EU AI Act) contain bare `<ALINEA>`
/// elements with no `<PARAG>` wrapper; each becomes an [`Alinea`] directly.
fn parse_article(node: Node) -> Result<Article, Error> {
    let number = node
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "TI.ART")
        .map(extract_text)
        .unwrap_or_default();

    let title = node
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "STI.ART")
        .map(extract_text);

    let subdiv_nodes: Vec<_> = node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "SUBDIV")
        .collect();

    let parag_nodes: Vec<_> = node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "PARAG")
        .collect();

    let content = if !subdiv_nodes.is_empty() {
        let subdivisions = subdiv_nodes
            .into_iter()
            .map(parse_subdivision)
            .collect::<Result<Vec<_>, _>>()?;
        ArticleContent::Subdivisions(subdivisions)
    } else if !parag_nodes.is_empty() {
        let paragraphs = parag_nodes
            .into_iter()
            .map(parse_legal_paragraph)
            .collect::<Result<Vec<_>, _>>()?;
        ArticleContent::Paragraphs(paragraphs)
    } else {
        // Some articles have bare <ALINEA> children with no <PARAG> wrapper.
        // Real example: Article 3 of the DSA.
        let alineas = node
            .children()
            .filter(|n| n.is_element() && n.tag_name().name() == "ALINEA")
            .map(parse_alinea)
            .collect();
        ArticleContent::Alineas(alineas)
    };

    Ok(Article {
        number,
        title,
        content,
    })
}

/// Parses a `<PARAG>` element into a [`LegalParagraph`].
fn parse_legal_paragraph(node: Node) -> Result<LegalParagraph, Error> {
    let number = node
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "NO.PARAG")
        .map(extract_text)
        .unwrap_or_default();

    let alineas = node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "ALINEA")
        .map(parse_alinea)
        .collect();

    let citations = extract_citations(node);
    Ok(LegalParagraph {
        number,
        alineas,
        citations,
    })
}

/// Parses a single `<ALINEA>` element into an [`Alinea`].
fn parse_alinea(node: Node) -> Alinea {
    Alinea {
        content: parse_block_children(node),
        citations: extract_citations(node),
    }
}

/// Parses a `<SUBDIV>` element into a [`Subdivision`].
fn parse_subdivision(node: Node) -> Result<Subdivision, Error> {
    let title = node
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "TITLE")
        .map(extract_text)
        .unwrap_or_default();

    let subdiv_nodes: Vec<_> = node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "SUBDIV")
        .collect();

    let parag_nodes: Vec<_> = node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "PARAG")
        .collect();

    let content = if !subdiv_nodes.is_empty() {
        let subdivisions = subdiv_nodes
            .into_iter()
            .map(parse_subdivision)
            .collect::<Result<Vec<_>, _>>()?;
        SubdivisionContent::Subdivisions(subdivisions)
    } else if !parag_nodes.is_empty() {
        let paragraphs = parag_nodes
            .into_iter()
            .map(parse_legal_paragraph)
            .collect::<Result<Vec<_>, _>>()?;
        SubdivisionContent::Paragraphs(paragraphs)
    } else {
        let alineas = node
            .children()
            .filter(|n| n.is_element() && n.tag_name().name() == "ALINEA")
            .map(parse_alinea)
            .collect();
        SubdivisionContent::Alineas(alineas)
    };

    Ok(Subdivision { title, content })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parses a raw XML string into a `roxmltree::Document`, panicking on error.
    fn doc(xml: &str) -> roxmltree::Document<'_> {
        roxmltree::Document::parse(xml).unwrap()
    }

    // ── parse_act errors ──────────────────────────────────────────────────────

    #[test]
    /// An `<ACT>` without a `<TITLE>` element returns a `MissingElement` error.
    fn parse_act_missing_title() {
        let result = parse_regular_act("<ACT><PREAMBLE/><ENACTING.TERMS/></ACT>");
        assert!(matches!(result, Err(Error::MissingElement(_))));
    }

    // ── consolidated act (CONS.ACT) ───────────────────────────────────────────

    #[test]
    /// A minimal `<CONS.ACT>` with one `<DIVISION>` is parsed into title, preamble, and enacting terms.
    fn parse_cons_act_basic() {
        let xml = r#"<CONS.ACT>
            <INFO.CONSLEG/>
            <INFO.PROD/>
            <CONS.DOC>
                <BIB.INSTANCE/>
                <FAM.COMP/>
                <TITLE><TI><P>Test Consolidated Regulation</P></TI></TITLE>
                <PREAMBLE>
                    <PREAMBLE.INIT><P>THE COUNCIL,</P></PREAMBLE.INIT>
                    <PREAMBLE.FINAL><P>HAVE ADOPTED:</P></PREAMBLE.FINAL>
                </PREAMBLE>
                <ENACTING.TERMS>
                    <DIVISION>
                        <TITLE><TI><P>TITLE I</P></TI></TITLE>
                        <ARTICLE><TI.ART>Article 1</TI.ART><ALINEA>Text.</ALINEA></ARTICLE>
                    </DIVISION>
                </ENACTING.TERMS>
            </CONS.DOC>
        </CONS.ACT>"#;
        let (title, preamble, enacting_terms) = parse_consolidated_act(xml).unwrap();
        assert_eq!(title, "Test Consolidated Regulation");
        assert_eq!(preamble.init, "THE COUNCIL,");
        assert_eq!(preamble.enacting_formula, "HAVE ADOPTED:");
        let EnactingTermsContent::Chapters(ref chapters) = enacting_terms.content else {
            panic!("expected Chapters content");
        };
        assert_eq!(chapters.len(), 1);
    }

    #[test]
    /// A `<TOC>` element inside `<ENACTING.TERMS>` is ignored; only `<DIVISION>` elements become chapters.
    fn parse_cons_act_toc_is_skipped() {
        // The <TOC> element inside <ENACTING.TERMS> of a consolidated act must
        // not be counted as a chapter — only <DIVISION> elements are chapters.
        let xml = r#"<CONS.ACT>
            <INFO.CONSLEG/>
            <CONS.DOC>
                <TITLE><TI><P>Act</P></TI></TITLE>
                <PREAMBLE>
                    <PREAMBLE.INIT><P>Init.</P></PREAMBLE.INIT>
                    <PREAMBLE.FINAL><P>Final.</P></PREAMBLE.FINAL>
                </PREAMBLE>
                <ENACTING.TERMS>
                    <TOC><TITLE><TI><P>Table of Contents</P></TI></TITLE></TOC>
                    <DIVISION>
                        <TITLE><TI><P>TITLE I</P></TI></TITLE>
                        <ARTICLE><TI.ART>Article 1</TI.ART><ALINEA>Text.</ALINEA></ARTICLE>
                    </DIVISION>
                    <DIVISION>
                        <TITLE><TI><P>TITLE II</P></TI></TITLE>
                        <ARTICLE><TI.ART>Article 2</TI.ART><ALINEA>Text.</ALINEA></ARTICLE>
                    </DIVISION>
                </ENACTING.TERMS>
            </CONS.DOC>
        </CONS.ACT>"#;
        let (_, _, enacting_terms) = parse_consolidated_act(xml).unwrap();
        let EnactingTermsContent::Chapters(ref chapters) = enacting_terms.content else {
            panic!("expected Chapters content");
        };
        assert_eq!(chapters.len(), 2, "TOC must not be counted as a chapter");
    }

    #[test]
    /// An `<ACT>` without a `<PREAMBLE>` element returns a `MissingElement` error.
    fn parse_act_missing_preamble() {
        let result =
            parse_regular_act("<ACT><TITLE><TI><P>Title</P></TI></TITLE><ENACTING.TERMS/></ACT>");
        assert!(matches!(result, Err(Error::MissingElement(_))));
    }

    #[test]
    /// An `<ACT>` without an `<ENACTING.TERMS>` element returns a `MissingElement` error.
    fn parse_act_missing_enacting_terms() {
        let result =
            parse_regular_act("<ACT><TITLE><TI><P>Title</P></TI></TITLE><PREAMBLE/></ACT>");
        assert!(matches!(result, Err(Error::MissingElement(_))));
    }

    // ── title ─────────────────────────────────────────────────────────────────

    #[test]
    /// Multiple `<P>` children inside `<TI>` are joined with a space into a single title string.
    fn title_joins_p_elements() {
        let xml = "<TITLE><TI><P>Act</P><P>of 1 January</P></TI></TITLE>";
        let d = doc(xml);
        let result = parse_title(d.root_element()).unwrap();
        assert_eq!(result, "Act of 1 January");
    }

    // ── preamble ──────────────────────────────────────────────────────────────

    #[test]
    /// Preamble with two `<VISA>` and three `<CONSID>` elements produces the correct counts and texts.
    fn preamble_counts_visas_and_recitals() {
        let xml = r#"<PREAMBLE>
            <PREAMBLE.INIT><P>THE COUNCIL,</P></PREAMBLE.INIT>
            <GR.VISA>
                <VISA><P>Visa one</P></VISA>
                <VISA><P>Visa two</P></VISA>
            </GR.VISA>
            <GR.CONSID>
                <CONSID><NP><NO.P>(1)</NO.P><TXT>First recital.</TXT></NP></CONSID>
                <CONSID><NP><NO.P>(2)</NO.P><TXT>Second recital.</TXT></NP></CONSID>
                <CONSID><NP><NO.P>(3)</NO.P><TXT>Third recital.</TXT></NP></CONSID>
            </GR.CONSID>
            <PREAMBLE.FINAL><P>HAVE ADOPTED:</P></PREAMBLE.FINAL>
        </PREAMBLE>"#;
        let d = doc(xml);
        let p = parse_preamble(d.root_element()).unwrap();
        assert_eq!(p.visas.len(), 2);
        assert_eq!(p.recitals.len(), 3);
        assert_eq!(p.init, "THE COUNCIL,");
        assert_eq!(p.enacting_formula, "HAVE ADOPTED:");
    }

    #[test]
    /// A `<CONSID>` with `<NO.P>` and `<TXT>` produces the correct recital number and text.
    fn recital_number_and_text() {
        let xml = "<CONSID><NP><NO.P>(42)</NO.P><TXT>Some text.</TXT></NP></CONSID>";
        let d = doc(xml);
        let r = parse_recital(d.root_element());
        assert_eq!(r.number, "(42)");
        assert_eq!(r.text, "Some text.");
    }

    #[test]
    /// A `<CONSID>` with no `<NP>` wrapper falls back to rendering the whole element
    /// as plain text with an empty number string.
    fn recital_without_np_falls_back_to_full_text() {
        let xml = "<CONSID><P>Unnumbered recital.</P></CONSID>";
        let d = doc(xml);
        let r = parse_recital(d.root_element());
        assert_eq!(r.number, "");
        assert_eq!(r.text, "Unnumbered recital.");
    }

    // ── chapters and sections ─────────────────────────────────────────────────

    #[test]
    /// A `<DIVISION>` with only `<ARTICLE>` children (no nested `<DIVISION>`) produces
    /// `ChapterContents::Articles`.
    fn chapter_with_direct_articles() {
        let xml = r#"<DIVISION>
            <TITLE><TI><P>CHAPTER I</P></TI></TITLE>
            <ARTICLE><TI.ART>Article 1</TI.ART><ALINEA>Text.</ALINEA></ARTICLE>
            <ARTICLE><TI.ART>Article 2</TI.ART><ALINEA>Text.</ALINEA></ARTICLE>
        </DIVISION>"#;
        let d = doc(xml);
        let ch = parse_chapter(d.root_element()).unwrap();
        assert_eq!(ch.title, "CHAPTER I");
        match ch.contents {
            ChapterContents::Articles(arts) => assert_eq!(arts.len(), 2),
            ChapterContents::Sections(_) => panic!("expected Articles"),
        }
    }

    #[test]
    /// A `<DIVISION>` whose children are themselves `<DIVISION>` elements produces
    /// `ChapterContents::Sections`, each section carrying its own articles.
    fn chapter_with_sections() {
        let xml = r#"<DIVISION>
            <TITLE><TI><P>CHAPTER III</P></TI></TITLE>
            <DIVISION>
                <TITLE><TI><P>SECTION 1</P></TI></TITLE>
                <ARTICLE><TI.ART>Article 5</TI.ART><ALINEA>Text.</ALINEA></ARTICLE>
            </DIVISION>
            <DIVISION>
                <TITLE><TI><P>SECTION 2</P></TI></TITLE>
                <ARTICLE><TI.ART>Article 6</TI.ART><ALINEA>Text.</ALINEA></ARTICLE>
            </DIVISION>
        </DIVISION>"#;
        let d = doc(xml);
        let ch = parse_chapter(d.root_element()).unwrap();
        match ch.contents {
            ChapterContents::Sections(secs) => {
                assert_eq!(secs.len(), 2);
                assert_eq!(secs[0].title, "SECTION 1");
                assert_eq!(secs[1].articles.len(), 1);
            }
            ChapterContents::Articles(_) => panic!("expected Sections"),
        }
    }

    // ── articles ──────────────────────────────────────────────────────────────

    #[test]
    /// An `<ARTICLE>` with `<PARAG>` wrappers, a `<TI.ART>` number, and a
    /// `<STI.ART>` subtitle is parsed into the correct counts and field values.
    fn article_with_paragraphs() {
        let xml = r#"<ARTICLE>
            <TI.ART>Article 6</TI.ART>
            <STI.ART><P>Classification rules</P></STI.ART>
            <PARAG><NO.PARAG>1.</NO.PARAG><ALINEA>First paragraph.</ALINEA></PARAG>
            <PARAG><NO.PARAG>2.</NO.PARAG><ALINEA>Second paragraph.</ALINEA></PARAG>
        </ARTICLE>"#;
        let d = doc(xml);
        let art = parse_article(d.root_element()).unwrap();
        assert_eq!(art.number, "Article 6");
        assert_eq!(art.title.as_deref(), Some("Classification rules"));
        let ArticleContent::Paragraphs(ref paras) = art.content else {
            panic!("expected Paragraphs");
        };
        assert_eq!(paras.len(), 2);
        assert_eq!(paras[0].number, "1.");
        assert!(matches!(&paras[1].alineas[0].content[0],
            Subparagraph::Text(t) if t == "Second paragraph."));
    }

    #[test]
    fn article_bare_alineas_become_alineas_variant() {
        // Some articles have no <PARAG> wrapper — alineas sit directly under <ARTICLE>.
        let xml = r#"<ARTICLE>
            <TI.ART>Article 113</TI.ART>
            <ALINEA>Only text.</ALINEA>
        </ARTICLE>"#;
        let d = doc(xml);
        let art = parse_article(d.root_element()).unwrap();
        let ArticleContent::Alineas(ref alineas) = art.content else {
            panic!("expected Alineas");
        };
        assert_eq!(alineas.len(), 1);
        assert!(matches!(&alineas[0].content[0], Subparagraph::Text(t) if t == "Only text."));
    }

    #[test]
    /// A `<LIST>` inside an `<ALINEA>` must produce a `Subparagraph::List` (not flat
    /// text), including correct nesting for sub-lists — matching Article 5's
    /// prohibited-practices list structure.
    fn alinea_list_items_are_content_blocks() {
        let xml = r#"<ARTICLE>
            <TI.ART>Article 5</TI.ART>
            <PARAG>
                <NO.PARAG>1.</NO.PARAG>
                <ALINEA>
                    <P>The following shall be prohibited:</P>
                    <LIST TYPE="alpha">
                        <ITEM><NP><NO.P>(a)</NO.P><TXT>Practice A.</TXT></NP></ITEM>
                        <ITEM><NP>
                            <NO.P>(b)</NO.P>
                            <TXT>Practice B:</TXT>
                            <P><LIST TYPE="roman">
                                <ITEM><NP><NO.P>(i)</NO.P><TXT>Sub-practice i.</TXT></NP></ITEM>
                                <ITEM><NP><NO.P>(ii)</NO.P><TXT>Sub-practice ii.</TXT></NP></ITEM>
                            </LIST></P>
                        </NP></ITEM>
                    </LIST>
                </ALINEA>
                <ALINEA>Point (b) is without prejudice to existing rules.</ALINEA>
            </PARAG>
        </ARTICLE>"#;
        let d = doc(xml);
        let art = parse_article(d.root_element()).unwrap();
        let ArticleContent::Paragraphs(ref paras) = art.content else {
            panic!("expected Paragraphs");
        };
        assert_eq!(paras.len(), 1);
        let alineas = &paras[0].alineas;
        // Two <ALINEA> elements → two Alinea structs.
        assert_eq!(alineas.len(), 2);
        // First alinea: intro+list grouped into one List block.
        assert_eq!(alineas[0].content.len(), 1);
        match &alineas[0].content[0] {
            Subparagraph::List(lb) => {
                assert!(lb.intro.contains("prohibited"));
                assert_eq!(lb.items.len(), 2);
                assert!(matches!(
                    &lb.items[0],
                    Item {
                        number: 1,
                        content: ItemContent::Text(_)
                    }
                ));
                match &lb.items[1] {
                    Item {
                        number: 2,
                        content: ItemContent::List(inner),
                    } => {
                        assert_eq!(inner.intro, "Practice B:");
                        assert_eq!(inner.items.len(), 2);
                        assert!(matches!(
                            &inner.items[0],
                            Item {
                                number: 1,
                                content: ItemContent::Text(_)
                            }
                        ));
                        assert!(matches!(
                            &inner.items[1],
                            Item {
                                number: 2,
                                content: ItemContent::Text(_)
                            }
                        ));
                    }
                    _ => panic!("expected nested List for item (b)"),
                }
            }
            _ => panic!("expected List at alineas[0].content[0]"),
        }
        assert!(matches!(&alineas[1].content[0], Subparagraph::Text(t) if t.contains("prejudice")));
    }

    #[test]
    /// An `<ALINEA>` containing a `<P>` intro followed by a `<LIST>` must produce a
    /// single `Subparagraph::List` with the intro set and items populated —
    /// matching Article 3 of the EU AI Act (definitions article).
    fn alinea_list_expands_to_individual_alineas() {
        let xml = r#"<ARTICLE>
            <TI.ART>Article 3</TI.ART>
            <STI.ART><P>Definitions</P></STI.ART>
            <ALINEA>
                <P>For the purposes of this Regulation:</P>
                <LIST TYPE="ARAB">
                    <ITEM><NP><NO.P>(1)</NO.P><TXT>first definition</TXT></NP></ITEM>
                    <ITEM><NP><NO.P>(2)</NO.P><TXT>second definition</TXT></NP></ITEM>
                </LIST>
            </ALINEA>
        </ARTICLE>"#;
        let d = doc(xml);
        let art = parse_article(d.root_element()).unwrap();
        let ArticleContent::Alineas(ref alineas) = art.content else {
            panic!("expected Alineas");
        };
        assert_eq!(alineas.len(), 1);
        // Intro+list collapsed into a single List block inside the one Alinea.
        assert_eq!(alineas[0].content.len(), 1);
        match &alineas[0].content[0] {
            Subparagraph::List(lb) => {
                assert_eq!(lb.intro, "For the purposes of this Regulation:");
                assert_eq!(lb.items.len(), 2);
                assert!(
                    matches!(&lb.items[0], Item { number: 1, content: ItemContent::Text(t) } if t == "first definition")
                );
                assert!(
                    matches!(&lb.items[1], Item { number: 2, content: ItemContent::Text(t) } if t == "second definition")
                );
            }
            _ => panic!("expected List at alineas[0].content[0]"),
        }
    }
}
