//! Parser for Formex annex XML files (`<ANNEX>` root) and for inline
//! consolidated annexes (`<CONS.ANNEX>` elements inside `<CONS.ACT>`).
//!
//! [`parse_annex`] handles standalone annex files; [`parse_cons_annex`] extracts
//! all `<CONS.ANNEX>` elements from a consolidated act document. Both delegate
//! to the shared helper [`parse_annex_node`].

use roxmltree::{Document, Node};

use super::text::extract_text;
use super::{
    child, extract_citations, list_type_from, parse_block_children, parse_items, parse_single_tbl,
    parse_table,
};
use crate::error::Error;
use crate::model::*;

/// Parses a Formex annex XML string (`<ANNEX>` root) into an [`Annex`].
///
/// If the `<CONTENTS>` element's top-level children include any `<GR.SEQ>`
/// elements the annex is parsed as [`AnnexContent::Sections`]; otherwise it is
/// parsed as [`AnnexContent::Paragraphs`], treating each `<NP>` as a
/// [`PhysicalNumberedParagraph`] and grouping surrounding `<P>`/`<LIST>` elements into
/// anonymous paragraphs.
///
/// # Errors
///
/// Returns [`crate::error::Error::Xml`] for malformed XML and
/// [`crate::error::Error::MissingElement`] if `<TITLE>` is absent.
pub fn parse_annex(xml: &str) -> Result<Annex, Error> {
    let doc = Document::parse(xml)?;
    parse_annex_node(doc.root_element())
}

/// Parses all `<CONS.ANNEX>` elements embedded in a `<CONS.ACT>` document,
/// returning them in document order.
///
/// Consolidated acts keep their annexes inline inside `<CONS.DOC>` rather
/// than as separate files. Each `<CONS.ANNEX>` has the same `<TITLE>` +
/// `<CONTENTS>` structure as a standalone `<ANNEX>` and is parsed identically.
///
/// # Errors
///
/// Returns [`crate::error::Error::Xml`] for malformed XML and
/// [`crate::error::Error::MissingElement`] if `<CONS.DOC>` is absent.
pub fn parse_cons_annex(xml: &str) -> Result<Vec<Annex>, Error> {
    let doc = Document::parse(xml)?;
    let root = doc.root_element();
    let cons_doc = root
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "CONS.DOC")
        .ok_or(Error::MissingElement("CONS.DOC"))?;
    cons_doc
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "CONS.ANNEX")
        .map(parse_annex_node)
        .collect()
}

/// Parses a single annex node — works for both `<ANNEX>` and `<CONS.ANNEX>`.
fn parse_annex_node(root: Node) -> Result<Annex, Error> {
    let title_node = child(root, "TITLE")?;

    let number = title_node
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "TI")
        .map(extract_text)
        .unwrap_or_default();

    let subtitle = title_node
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "STI")
        .map(extract_text);

    let content = root
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "CONTENTS")
        .map(|contents| {
            let has_sections = contents
                .children()
                .any(|n| n.is_element() && n.tag_name().name() == "GR.SEQ");
            if has_sections {
                AnnexContent::Sections(parse_annex_sections(contents))
            } else {
                AnnexContent::Paragraphs(parse_annex_paragraphs(contents))
            }
        })
        .unwrap_or(AnnexContent::Paragraphs(vec![]));

    Ok(Annex {
        number,
        subtitle,
        content,
    })
}

/// Collects all top-level `<GR.SEQ>` children of `node` as [`AnnexSection`]s.
fn parse_annex_sections(node: Node) -> Vec<AnnexSection> {
    node.children()
        .filter(|n| n.is_element() && n.tag_name().name() == "GR.SEQ")
        .map(|gr| {
            let title = gr
                .children()
                .find(|n| n.is_element() && n.tag_name().name() == "TITLE")
                .and_then(|t| {
                    t.children()
                        .find(|n| n.is_element() && n.tag_name().name() == "TI")
                })
                .map(extract_text)
                .unwrap_or_default();
            let citations = extract_citations(gr);
            AnnexSection {
                title,
                alineas: parse_block_children(gr),
                citations,
            }
        })
        .collect()
}

/// Parses flat annex content as a sequence of [`Subparagraph`]s.
///
/// Each top-level `<NP>` becomes [`Subparagraph::Numbered`]. Bare `<P>` text
/// becomes [`Subparagraph::Plain`]; a `<P>` text immediately preceding a sibling
/// `<LIST>` is absorbed into [`ListBlock::intro`]. `<LIST>`, `<TBL>`, and `<GR.TBL>`
/// become `List` and `Table` items directly.
fn parse_annex_paragraphs(node: Node) -> Vec<Subparagraph> {
    let mut result: Vec<Subparagraph> = Vec::new();
    let mut pending_intro: Option<String> = None;

    for elem in node.children().filter(|n| n.is_element()) {
        match elem.tag_name().name() {
            "NP" => {
                if let Some(t) = pending_intro.take() {
                    result.push(Subparagraph::Plain(t));
                }
                result.push(Subparagraph::Numbered(np_to_physical_numbered_paragraph(elem)));
            }
            "P" => {
                let nested_blocks: Vec<_> = elem
                    .children()
                    .filter(|n| n.is_element() && matches!(n.tag_name().name(), "LIST" | "TBL"))
                    .collect();
                if !nested_blocks.is_empty() {
                    if let Some(t) = pending_intro.take() {
                        result.push(Subparagraph::Plain(t));
                    }
                    for block in nested_blocks {
                        match block.tag_name().name() {
                            "LIST" => result.push(Subparagraph::List(ListBlock {
                                list_type: list_type_from(block),
                                intro: String::new(),
                                items: parse_items(block),
                            })),
                            _ => result.push(parse_single_tbl(block)),
                        }
                    }
                } else {
                    if let Some(t) = pending_intro.take() {
                        result.push(Subparagraph::Plain(t));
                    }
                    let t = extract_text(elem);
                    if !t.is_empty() {
                        pending_intro = Some(t);
                    }
                }
            }
            "LIST" => {
                let intro = pending_intro.take().unwrap_or_default();
                result.push(Subparagraph::List(ListBlock {
                    list_type: list_type_from(elem),
                    intro,
                    items: parse_items(elem),
                }));
            }
            "TITLE" => {}
            "GR.TBL" => {
                if let Some(t) = pending_intro.take() {
                    result.push(Subparagraph::Plain(t));
                }
                result.extend(parse_table(elem));
            }
            "TBL" => {
                if let Some(t) = pending_intro.take() {
                    result.push(Subparagraph::Plain(t));
                }
                result.push(parse_single_tbl(elem));
            }
            _ => {
                if let Some(t) = pending_intro.take() {
                    result.push(Subparagraph::Plain(t));
                }
                let t = extract_text(elem);
                if !t.is_empty() {
                    pending_intro = Some(t);
                }
            }
        }
    }

    if let Some(t) = pending_intro {
        result.push(Subparagraph::Plain(t));
    }
    result
}

/// Converts a single `<NP>` element into a [`PhysicalNumberedParagraph`].
///
/// `<NO.P>` becomes the paragraph number, `<TXT>` becomes the first alinea,
/// and any `<P><LIST>` nested inside the `<NP>` become additional alineas.
fn np_to_physical_numbered_paragraph(node: Node) -> PhysicalNumberedParagraph {
    let number = node
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "NO.P")
        .map(extract_text)
        .unwrap_or_default();

    let txt = node
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "TXT")
        .map(extract_text)
        .unwrap_or_default();

    let nested_lists: Vec<Node> = node
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "P")
        .flat_map(|p| {
            p.children()
                .filter(|n| n.is_element() && n.tag_name().name() == "LIST")
        })
        .collect();

    let alineas = if nested_lists.is_empty() {
        if txt.is_empty() {
            vec![]
        } else {
            vec![Subparagraph::Plain(txt)]
        }
    } else {
        let list_type = nested_lists.first().and_then(|n| list_type_from(*n));
        let items = nested_lists.into_iter().flat_map(parse_items).collect();
        vec![Subparagraph::List(ListBlock {
            list_type,
            intro: txt,
            items,
        })]
    };

    let citations = extract_citations(node);
    PhysicalNumberedParagraph { number, alineas, citations }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Wraps `contents_inner` in a minimal `<ANNEX>` document and returns the
    /// parsed sections, panicking if content is not `AnnexContent::Sections`.
    fn parse_sections(contents_inner: &str) -> Vec<AnnexSection> {
        let xml = format!(
            r#"<ANNEX><TITLE><TI><P>ANNEX X</P></TI></TITLE><CONTENTS>{}</CONTENTS></ANNEX>"#,
            contents_inner
        );
        match parse_annex(&xml).unwrap().content {
            AnnexContent::Sections(s) => s,
            AnnexContent::Paragraphs(_) => panic!("expected Sections"),
        }
    }

    /// Wraps `contents_inner` in a minimal `<ANNEX>` document and returns the
    /// parsed paragraphs, panicking if content is not `AnnexContent::Paragraphs`.
    fn parse_paragraphs(contents_inner: &str) -> Vec<Subparagraph> {
        let xml = format!(
            r#"<ANNEX><TITLE><TI><P>ANNEX X</P></TI></TITLE><CONTENTS>{}</CONTENTS></ANNEX>"#,
            contents_inner
        );
        match parse_annex(&xml).unwrap().content {
            AnnexContent::Paragraphs(p) => p,
            AnnexContent::Sections(_) => panic!("expected Paragraphs"),
        }
    }

    // ── parse_annex errors ────────────────────────────────────────────────────

    #[test]
    /// An `<ANNEX>` with no `<TITLE>` element returns a `MissingElement` error.
    fn parse_annex_missing_title() {
        let result = parse_annex("<ANNEX><CONTENTS/></ANNEX>");
        assert!(matches!(
            result,
            Err(crate::error::Error::MissingElement(_))
        ));
    }

    // ── parse_cons_annex ────────────────────────────────────────────────

    #[test]
    /// Two `<CONS.ANNEX>` elements inside `<CONS.DOC>` are each parsed: the first
    /// as `Paragraphs` mode, the second (containing `<GR.SEQ>`) as `Sections` mode.
    fn parse_cons_annex_basic() {
        let xml = r#"<CONS.ACT>
            <INFO.CONSLEG/>
            <CONS.DOC>
                <TITLE><TI><P>Act</P></TI></TITLE>
                <CONS.ANNEX>
                    <TITLE><TI><P>ANNEX I</P></TI><STI><P>Subtitle</P></STI></TITLE>
                    <CONTENTS><P>Some content.</P></CONTENTS>
                </CONS.ANNEX>
                <CONS.ANNEX>
                    <TITLE><TI><P>ANNEX II</P></TI></TITLE>
                    <CONTENTS>
                        <GR.SEQ>
                            <TITLE><TI><P>Part A</P></TI></TITLE>
                            <P>Content.</P>
                        </GR.SEQ>
                    </CONTENTS>
                </CONS.ANNEX>
            </CONS.DOC>
        </CONS.ACT>"#;
        let annexes = parse_cons_annex(xml).unwrap();
        assert_eq!(annexes.len(), 2);
        assert_eq!(annexes[0].number, "ANNEX I");
        assert_eq!(annexes[0].subtitle.as_deref(), Some("Subtitle"));
        assert!(matches!(&annexes[0].content, AnnexContent::Paragraphs(_)));
        assert_eq!(annexes[1].number, "ANNEX II");
        assert!(matches!(&annexes[1].content, AnnexContent::Sections(s) if s.len() == 1));
    }

    #[test]
    /// A `<CONS.DOC>` with no `<CONS.ANNEX>` children returns an empty vec.
    fn parse_cons_annex_empty_when_no_cons_annex() {
        let xml = r#"<CONS.ACT>
            <CONS.DOC>
                <TITLE><TI><P>Act</P></TI></TITLE>
            </CONS.DOC>
        </CONS.ACT>"#;
        let annexes = parse_cons_annex(xml).unwrap();
        assert!(annexes.is_empty());
    }

    // ── parse_annex ───────────────────────────────────────────────────────────

    #[test]
    /// `<TI>` inside `<TITLE>` becomes the annex `number` field and `<STI>` becomes
    /// `subtitle`.
    fn annex_title_and_subtitle() {
        let xml = r#"<ANNEX>
            <TITLE>
                <TI><P>ANNEX I</P></TI>
                <STI><P>List of legislation</P></STI>
            </TITLE>
            <CONTENTS/>
        </ANNEX>"#;
        let annex = parse_annex(xml).unwrap();
        assert_eq!(annex.number, "ANNEX I");
        assert_eq!(annex.subtitle.as_deref(), Some("List of legislation"));
    }

    #[test]
    /// An `<ANNEX>` with no `<STI>` element produces `subtitle: None`.
    fn annex_no_subtitle() {
        let xml = r#"<ANNEX>
            <TITLE><TI><P>ANNEX II</P></TI></TITLE>
            <CONTENTS/>
        </ANNEX>"#;
        let annex = parse_annex(xml).unwrap();
        assert!(annex.subtitle.is_none());
    }

    // ── Sections mode ─────────────────────────────────────────────────────────

    #[test]
    /// Two `<GR.SEQ>` elements produce two `AnnexSection`s with the correct titles
    /// and one text alinea each.
    fn gr_seq_produces_annex_sections() {
        let xml = r#"<GR.SEQ>
            <TITLE><TI><P>Part A</P></TI></TITLE>
            <P>Content paragraph.</P>
        </GR.SEQ>
        <GR.SEQ>
            <TITLE><TI><P>Part B</P></TI></TITLE>
            <P>Another paragraph.</P>
        </GR.SEQ>"#;
        let sections = parse_sections(xml);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].title, "Part A");
        assert_eq!(sections[0].alineas.len(), 1);
        assert!(
            matches!(&sections[0].alineas[0], Subparagraph::Plain(t) if t == "Content paragraph.")
        );
        assert_eq!(sections[1].title, "Part B");
    }

    #[test]
    /// A `<GR.SEQ>` containing a `<P>` intro and a `<LIST>` collapses into a single
    /// `Subparagraph::List` with the intro text and two items.
    fn gr_seq_with_list_inside() {
        let xml = r#"<GR.SEQ>
            <TITLE><TI><P>Section I</P></TI></TITLE>
            <P>Intro text.</P>
            <LIST TYPE="alpha">
                <ITEM><NP><NO.P>(a)</NO.P><TXT>Item a.</TXT></NP></ITEM>
                <ITEM><NP><NO.P>(b)</NO.P><TXT>Item b.</TXT></NP></ITEM>
            </LIST>
        </GR.SEQ>"#;
        let sections = parse_sections(xml);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].alineas.len(), 1);
        match &sections[0].alineas[0] {
            Subparagraph::List(lb) => {
                assert_eq!(lb.intro, "Intro text.");
                assert_eq!(lb.items.len(), 2);
            }
            _ => panic!("expected List"),
        }
    }

    // ── Paragraphs mode ───────────────────────────────────────────────────────

    #[test]
    /// A bare `<P>` with no surrounding `<NP>` becomes a `Subparagraph::Plain` item.
    fn plain_paragraphs_become_anonymous_paragraph() {
        let paras = parse_paragraphs("<P>Some text.</P>");
        assert_eq!(paras.len(), 1);
        assert!(matches!(&paras[0], Subparagraph::Plain(t) if t == "Some text."));
    }

    #[test]
    /// A `<P>` containing only whitespace produces no paragraphs (empty result).
    fn empty_contents_produces_no_paragraphs() {
        let paras = parse_paragraphs("<P>   </P>");
        assert_eq!(paras.len(), 0);
    }

    #[test]
    /// A single `<NP>` with `<NO.P>` and `<TXT>` becomes a `Subparagraph::Numbered`
    /// with one `Plain` alinea.
    fn np_becomes_numbered_paragraph() {
        let paras = parse_paragraphs("<NP><NO.P>1.</NO.P><TXT>First item.</TXT></NP>");
        assert_eq!(paras.len(), 1);
        let Subparagraph::Numbered(ref np) = paras[0] else { panic!("expected Numbered") };
        assert_eq!(np.number, "1.");
        assert_eq!(np.alineas.len(), 1);
        assert!(matches!(&np.alineas[0], Subparagraph::Plain(t) if t == "First item."));
    }

    #[test]
    /// Two consecutive `<NP>` elements each become a separate `Subparagraph::Numbered`.
    fn multiple_nps_become_separate_paragraphs() {
        let xml = r#"<NP><NO.P>1.</NO.P><TXT>First.</TXT></NP>
                     <NP><NO.P>2.</NO.P><TXT>Second.</TXT></NP>"#;
        let paras = parse_paragraphs(xml);
        assert_eq!(paras.len(), 2);
        let Subparagraph::Numbered(ref np0) = paras[0] else { panic!("expected Numbered") };
        let Subparagraph::Numbered(ref np1) = paras[1] else { panic!("expected Numbered") };
        assert_eq!(np0.number, "1.");
        assert_eq!(np1.number, "2.");
    }

    #[test]
    /// An `<NP>` whose `<TXT>` is followed by a `<P><LIST>` produces a single
    /// `Subparagraph::List` alinea with the `<TXT>` as intro and the list items.
    fn np_with_nested_list_becomes_list_alinea() {
        let xml = r#"<NP>
            <NO.P>1.</NO.P>
            <TXT>The following apply:</TXT>
            <P><LIST TYPE="alpha">
                <ITEM><NP><NO.P>(a)</NO.P><TXT>Item a.</TXT></NP></ITEM>
                <ITEM><NP><NO.P>(b)</NO.P><TXT>Item b.</TXT></NP></ITEM>
            </LIST></P>
        </NP>"#;
        let paras = parse_paragraphs(xml);
        assert_eq!(paras.len(), 1);
        let Subparagraph::Numbered(ref np) = paras[0] else { panic!("expected Numbered") };
        assert_eq!(np.number, "1.");
        assert_eq!(np.alineas.len(), 1);
        match &np.alineas[0] {
            Subparagraph::List(lb) => {
                assert_eq!(lb.intro, "The following apply:");
                assert_eq!(lb.items.len(), 2);
            }
            _ => panic!("expected List alinea"),
        }
    }

    #[test]
    /// A `<P>` immediately followed by a sibling `<LIST>` has its text promoted to
    /// the `intro` of the resulting `Subparagraph::List`; both collapse into one item.
    fn p_before_list_becomes_intro() {
        let xml = r#"<P>Items:</P>
                     <LIST TYPE="DASH">
                         <ITEM><NO.P>—</NO.P><P>One.</P></ITEM>
                         <ITEM><NO.P>—</NO.P><P>Two.</P></ITEM>
                     </LIST>"#;
        let paras = parse_paragraphs(xml);
        assert_eq!(paras.len(), 1);
        match &paras[0] {
            Subparagraph::List(lb) => {
                assert_eq!(lb.intro, "Items:");
                assert_eq!(lb.items.len(), 2);
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    /// A `<P>` that directly wraps a `<LIST>` (no plain text siblings) produces a
    /// `Subparagraph::List` directly with an empty intro.
    fn p_wrapping_list_becomes_list_block() {
        let xml = r#"<P><LIST TYPE="ARAB">
            <ITEM><NP><NO.P>1.</NO.P><TXT>First.</TXT></NP></ITEM>
            <ITEM><NP><NO.P>2.</NO.P><TXT>Second.</TXT></NP></ITEM>
        </LIST></P>"#;
        let paras = parse_paragraphs(xml);
        assert_eq!(paras.len(), 1);
        match &paras[0] {
            Subparagraph::List(lb) => {
                assert_eq!(lb.items.len(), 2);
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    /// A plain `<P>` before the first `<NP>` becomes a `Plain` item; each `<NP>`
    /// becomes a separate `Numbered` item — three flat items in total.
    fn mixed_p_and_np_groups_correctly() {
        let xml = r#"<P>Preamble text.</P>
                     <NP><NO.P>1.</NO.P><TXT>Item one.</TXT></NP>
                     <NP><NO.P>2.</NO.P><TXT>Item two.</TXT></NP>"#;
        let paras = parse_paragraphs(xml);
        assert_eq!(paras.len(), 3);
        assert!(matches!(&paras[0], Subparagraph::Plain(t) if t == "Preamble text."));
        let Subparagraph::Numbered(ref np1) = paras[1] else { panic!("expected Numbered") };
        let Subparagraph::Numbered(ref np2) = paras[2] else { panic!("expected Numbered") };
        assert_eq!(np1.number, "1.");
        assert_eq!(np2.number, "2.");
    }
}
