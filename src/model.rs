use std::collections::HashMap;

use serde::{Deserialize, Serialize}; // Deserialize needed for Subparagraph/ListBlock in tests

/// The type of a cited EU legislative act.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CitedActType {
    Regulation,
    Directive,
    Decision,
}

/// Official Journal publication coordinates, extracted from a `<REF.DOC.OJ>` element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OjRef {
    /// OJ series: `"L"` (legislation) or `"C"` (communications).
    pub collection: String,
    /// Issue number, e.g. `"277"`.
    pub number: String,
    /// Publication date in `YYYYMMDD` form, e.g. `"20221027"`.
    pub date: String,
    /// First page, e.g. `1`.
    pub page: u32,
}

/// A structured reference to another EU legislative act found in a recital, paragraph, or annex section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Citation {
    pub act_type: CitedActType,
    /// Legal regime: `"EU"`, `"EC"`, `"EEC"`, `"EURATOM"`, or `None` for unrecognised forms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regime: Option<String>,
    /// Act number, e.g. `"2022/2065"` or `"207/2009"`.
    pub number: String,
    /// Official Journal reference when the citation was backed by a `<REF.DOC.OJ>` element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oj_ref: Option<OjRef>,
}

/// Bibliographic metadata extracted from the `.doc.xml` registry file.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Metadata {
    /// CELEX identifier, e.g. `"32017R1001"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub celex: Option<String>,
    /// Act signing/adoption date in `YYYYMMDD` form.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_date: Option<String>,
    /// Formex legal-value code: `"REG"`, `"DIR"`, `"DEC"`, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legal_value: Option<String>,
    /// Document language code, e.g. `"EN"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Institutional authors, e.g. `["PE", "CS"]`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,
    /// `true` when the `<EEA/>` relevance flag is present.
    pub eea_relevant: bool,
    /// Official Journal publication reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub official_journal: Option<OfficialJournal>,
    /// First page in the Official Journal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_first: Option<u32>,
    /// Last page in the Official Journal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_last: Option<u32>,
    /// Total page count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_total: Option<u32>,
    /// Production ID (absent in older format files).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prod_id: Option<String>,
    /// Final ID (absent in older format files).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fin_id: Option<String>,
}

/// Official Journal publication reference.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OfficialJournal {
    /// Series, e.g. `"L"` or `"C"`.
    pub collection: String,
    /// Issue number, e.g. `"154"`.
    pub number: String,
    /// Publication date in `YYYYMMDD` form.
    pub date: String,
    /// Language edition, e.g. `"EN"`.
    pub language: String,
}

/// A parsed EU legislative act.
///
/// The two variants reflect the two Formex publication formats:
/// - [`Act::Regular`] — an original act (`<ACT>` root) with a full preamble
///   including legal-basis citations (visas) and numbered recitals.
/// - [`Act::Consolidated`] — a consolidated version (`<CONS.ACT>` root) with a
///   slim preamble: visas and recitals are structurally absent.
///
/// Serialises without a variant tag (`#[serde(untagged)]`), so the JSON output
/// is structurally identical to the underlying struct for each variant.
///
/// Convenience methods ([`Act::title`], [`Act::enacting_terms`],
/// [`Act::annexes`], [`Act::definitions`]) provide access to shared fields
/// without pattern-matching on the variant.
#[derive(Serialize)]
#[serde(untagged)]
pub enum Act {
    /// An original act with a full preamble.
    Regular(RegularAct),
    /// A consolidated act with a slim preamble (no visas or recitals).
    Consolidated(ConsolidatedAct),
}

impl Act {
    /// Bibliographic metadata from the `.doc.xml` registry.
    pub fn metadata(&self) -> &Metadata {
        match self {
            Act::Regular(a) => &a.metadata,
            Act::Consolidated(a) => &a.metadata,
        }
    }

    /// The full title of the act.
    pub fn title(&self) -> &str {
        match self {
            Act::Regular(a) => &a.title,
            Act::Consolidated(a) => &a.title,
        }
    }

    /// The operative body of the act.
    pub fn enacting_terms(&self) -> &EnactingTerms {
        match self {
            Act::Regular(a) => &a.enacting_terms,
            Act::Consolidated(a) => &a.enacting_terms,
        }
    }

    /// The annexes, in document order.
    pub fn annexes(&self) -> &[Annex] {
        match self {
            Act::Regular(a) => &a.annexes,
            Act::Consolidated(a) => &a.annexes,
        }
    }

    /// Definitions extracted from any "Definitions" article. Empty when absent.
    pub fn definitions(&self) -> &HashMap<String, String> {
        match self {
            Act::Regular(a) => &a.definitions,
            Act::Consolidated(a) => &a.definitions,
        }
    }
}

/// A complete original EU act (`<ACT>` root), with a full preamble.
#[derive(Serialize)]
pub struct RegularAct {
    /// Bibliographic metadata from the `.doc.xml` registry.
    pub metadata: Metadata,
    /// The full title of the act, e.g. `"Regulation (EU) 2024/1689 …"`.
    pub title: String,
    /// The preamble: opening formula, legal bases, numbered recitals, enacting formula.
    pub preamble: Preamble,
    /// The operative body of the act.
    pub enacting_terms: EnactingTerms,
    /// The annexes, in the order declared by the `.doc.fmx.xml` registry.
    pub annexes: Vec<Annex>,
    /// Definitions extracted from any "Definitions" article. Omitted from JSON when absent.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub definitions: HashMap<String, String>,
}

/// A complete consolidated EU act (`<CONS.ACT>` root), with a slim preamble.
///
/// Consolidated acts do not carry visas or recitals; use [`RegularAct`] when
/// you need those fields.
#[derive(Serialize)]
pub struct ConsolidatedAct {
    /// Bibliographic metadata from the `.doc.xml` registry.
    pub metadata: Metadata,
    /// The full title of the act.
    pub title: String,
    /// The slim preamble: opening formula and enacting formula only.
    pub preamble: ConsolidatedPreamble,
    /// The operative body of the act.
    pub enacting_terms: EnactingTerms,
    /// The annexes, parsed inline from `<CONS.ANNEX>` elements.
    pub annexes: Vec<Annex>,
    /// Definitions extracted from any "Definitions" article. Omitted from JSON when absent.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub definitions: HashMap<String, String>,
}

/// The slim preamble of a consolidated act.
///
/// Consolidated acts (`<CONS.ACT>`) carry only the opening institutional
/// formula and the enacting formula; legal-basis citations and recitals are
/// structurally absent (unlike [`Preamble`] where they are optional lists).
#[derive(Serialize)]
pub struct ConsolidatedPreamble {
    /// Opening institutional formula (`<PREAMBLE.INIT>`).
    pub init: String,
    /// Closing enacting formula (`<PREAMBLE.FINAL>`).
    pub enacting_formula: String,
}

/// The preamble of an act (`<PREAMBLE>`).
///
/// Formex splits the preamble into four structural parts: the opening
/// institutional formula (`PREAMBLE.INIT`), the legal bases (`GR.VISA`),
/// the recitals (`GR.CONSID`), and the closing enacting formula
/// (`PREAMBLE.FINAL`).
#[derive(Serialize)]
pub struct Preamble {
    /// Opening formula (`<PREAMBLE.INIT>`), e.g. `"THE EUROPEAN PARLIAMENT AND THE COUNCIL …"`.
    pub init: String,
    /// Legal basis citations (`<VISA>` elements inside `<GR.VISA>`),
    /// each rendered as plain text.
    pub visas: Vec<String>,
    /// Numbered recitals (`<CONSID>` elements inside `<GR.CONSID>`).
    pub recitals: Vec<Recital>,
    /// Closing enacting formula (`<PREAMBLE.FINAL>`), e.g. `"HAVE ADOPTED THIS REGULATION:"`.
    pub enacting_formula: String,
}

/// A single numbered recital (`<CONSID>`).
///
/// In Formex the content is wrapped in a numbered paragraph (`<NP>`):
/// `<NO.P>` holds the label and `<TXT>` holds the body.
#[derive(Debug, PartialEq, Serialize)]
pub struct Recital {
    /// The recital label, e.g. `"(1)"`.
    pub number: String,
    /// The plain-text body of the recital.
    pub text: String,
    /// Structured citations to other EU acts found in this recital.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub citations: Vec<Citation>,
}

/// Discriminates the top-level structure of enacting terms.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum EnactingTermsContent {
    /// The act is divided into chapters (`<DIVISION>` elements inside `<ENACTING.TERMS>`).
    Chapters(Vec<Chapter>),
    /// The act has articles directly in `<ENACTING.TERMS>` with no chapter wrapper.
    Articles(Vec<Article>),
}

/// The operative body of the act (`<ENACTING.TERMS>`).
#[derive(Serialize)]
pub struct EnactingTerms {
    /// Top-level content: either chapters (subdivided acts) or articles directly
    /// (flat acts with no `<DIVISION>` wrapper).
    pub content: EnactingTermsContent,
}

/// A chapter of the act (`<DIVISION>` at the top level of `<ENACTING.TERMS>`).
///
/// Chapters either contain sections (themselves containing articles) or
/// articles directly — never both.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    /// Chapter heading, e.g. `"CHAPTER I"` (from `<TITLE><TI>`).
    pub title: String,
    /// Optional chapter subtitle, e.g. `"General provisions"` (from `<TITLE><STI>`).
    pub subtitle: Option<String>,
    /// Either sections (each grouping articles) or articles directly —
    /// the two forms never mix within a single chapter.
    pub contents: ChapterContents,
}

/// Discriminates whether a chapter is sub-divided into sections.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ChapterContents {
    /// The chapter groups its articles under named sections.
    Sections(Vec<Section>),
    /// The chapter contains articles directly, with no section level.
    Articles(Vec<Article>),
}

/// A section within a chapter (`<DIVISION>` nested inside a top-level `<DIVISION>`).
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Section {
    /// Section heading, e.g. `"SECTION 1"` (from `<TITLE><TI>`).
    pub title: String,
    /// Optional section subtitle (from `<TITLE><STI>`); present only in some acts.
    pub subtitle: Option<String>,
    /// Articles in this section. Sections are never nested further.
    pub articles: Vec<Article>,
}

/// Discriminates the top-level content structure of an article.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ArticleContent {
    /// The article contains `<PARAG>`-wrapped numbered paragraphs.
    Paragraphs(Vec<LegalParagraph>),
    /// The article contains bare `<ALINEA>` elements with no `<PARAG>` wrapper.
    Alineas(Vec<Alinea>),
    /// The article contains `<SUBDIV>`-grouped sections.
    Subdivisions(Vec<Subdivision>),
}

/// A single article (`<ARTICLE>`).
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Article {
    /// Article number as printed, e.g. `"Article 6"` (from `<TI.ART>`).
    pub number: String,
    /// Optional article title, e.g. `"Classification rules for high-risk AI systems"`
    /// (from `<STI.ART>`).
    pub title: Option<String>,
    /// The content of the article: numbered paragraphs, bare alineas, or thematic subdivisions.
    pub content: ArticleContent,
}

/// A numbered legal paragraph within an article (`<PARAG>`).
///
/// Always has a number (from `<NO.PARAG>`) and contains one or more alineas.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LegalParagraph {
    /// Paragraph number, e.g. `"1."` (from `<NO.PARAG>`).
    pub number: String,
    /// The alineas of this paragraph (each maps to one `<ALINEA>` element).
    pub alineas: Vec<Alinea>,
    /// Structured citations to other EU acts found in this paragraph.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub citations: Vec<Citation>,
}

/// Discriminates the content of a [`Subdivision`].
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum SubdivisionContent {
    /// The subdivision contains `<PARAG>`-wrapped numbered paragraphs.
    Paragraphs(Vec<LegalParagraph>),
    /// The subdivision contains bare `<ALINEA>` elements.
    Alineas(Vec<Alinea>),
    /// The subdivision contains nested `<SUBDIV>` elements.
    Subdivisions(Vec<Subdivision>),
}

/// A titled subdivision within an article (`<SUBDIV>`).
///
/// Content is one of: numbered paragraphs, bare alineas, or nested subdivisions.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Subdivision {
    /// Subdivision title (from `<TITLE>`).
    pub title: String,
    /// The content of this subdivision.
    pub content: SubdivisionContent,
}

/// An alinea within a [`LegalParagraph`] or a bare alinea within an article (`<ALINEA>`).
///
/// An alinea is the lowest-level numbered structure of a legal article. It has no
/// number of its own; its content is one or more block elements.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Alinea {
    /// Block content of this alinea (text, list, or table elements).
    pub content: Vec<Subparagraph>,
    /// Structured citations to other EU acts found in this alinea.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub citations: Vec<Citation>,
}

/// A numbered physical paragraph in an annex (`<NP>`).
///
/// The number is always present, given by the `<NO.P>` child element.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicalNumberedParagraph {
    /// Paragraph number, e.g. `"1."` (from `<NO.P>`).
    pub number: String,
    /// Content blocks of this paragraph.
    pub alineas: Vec<Subparagraph>,
    /// Structured citations to other EU acts found in this paragraph.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub citations: Vec<Citation>,
}

/// An unnumbered physical paragraph (`<P>`).
///
/// A `<P>` element carries no number. Its content is one or more block
/// elements (plain text, list, or table).
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicalParagraph {
    /// Block content of this paragraph.
    pub content: Vec<Subparagraph>,
    /// Structured citations to other EU acts found in this paragraph.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub citations: Vec<Citation>,
}

/// An item in the flat paragraph sequence of an annex.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum AnnexParagraph {
    /// A numbered paragraph from a `<NP>` element.
    Numbered(PhysicalNumberedParagraph),
    /// An unnumbered paragraph from a `<P>` element (or accumulated block content).
    Plain(PhysicalParagraph),
}

impl AnnexParagraph {
    /// Returns the content blocks of this paragraph regardless of variant.
    pub fn subparagraphs(&self) -> &[Subparagraph] {
        match self {
            AnnexParagraph::Numbered(np) => &np.alineas,
            AnnexParagraph::Plain(pp) => &pp.content,
        }
    }
}

/// A content element within an [`Alinea`], [`PhysicalNumberedParagraph`], [`PhysicalParagraph`], or [`AnnexSection`].
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Subparagraph {
    /// Plain text block (a bare `<P>` or `<ALINEA>`).
    Text(String),
    /// A list (`<LIST>`), with an optional intro and its items.
    List(ListBlock),
    /// A table parsed from a `<GR.TBL>` or `<TBL>` element.
    Table(Table),
}

/// A single cell within a [`Row`] (`<CELL>`).
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    /// Plain-text content of the cell. Empty string for `<IE/>` (idem/empty marker).
    pub text: String,
    /// `true` when the cell carries `TYPE="HEADER"`.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub is_header: bool,
}

/// A row within a [`Table`] (`<ROW>`).
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Row {
    /// The cells in this row.
    pub cells: Vec<Cell>,
    /// Number of cells (convenience field matching `cells.len()`).
    pub cell_count: usize,
    /// `true` when the row carries `TYPE="HEADER"`.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub is_header: bool,
}

/// A table parsed from a `<TBL>` element inside `<GR.TBL>`.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Table {
    /// Number of columns declared in the `COLS` attribute of `<TBL>`.
    pub col_count: usize,
    /// Optional table title (from `<TITLE><TI>`). Omitted from JSON when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The rows of the table body (`<CORPUS><ROW>`).
    pub rows: Vec<Row>,
    /// Number of rows (convenience field matching `rows.len()`).
    pub row_count: usize,
}

/// The numbering style of a [`ListBlock`].
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListType {
    /// Alphabetic items: (a), (b), (c)…
    Alpha,
    /// Roman numeral items: (i), (ii), (iii)…
    Roman,
    /// Arabic numeral items: (1), (2), (3)…
    Arab,
    /// Dash items: —
    Dash,
}

/// A single `<ITEM>` within a [`ListBlock`].
///
/// Always carries a 1-based position. The display label ("a", "ii", "1", "—")
/// is derivable from the parent list's [`ListType`] and this number.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    /// 1-based position within the enclosing list.
    pub number: u32,
    /// The item content: plain text or a nested sub-list.
    pub content: ItemContent,
}

/// The content of a list [`Item`].
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ItemContent {
    /// A plain text item (no nested list).
    Text(String),
    /// An item whose body is itself a list, with optional intro text.
    List(ListBlock),
}

/// A `<LIST>` element: optional intro text, a style, and its items.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ListBlock {
    /// Numbering style of the list items. Omitted from JSON when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_type: Option<ListType>,
    /// The text that introduces the list (may be empty).
    pub intro: String,
    /// The items of the list.
    pub items: Vec<Item>,
}

/// A titled content section within an [`Annex`] (`<GR.SEQ>`).
///
/// Used when an annex organises its content under named headings.  For annexes
/// that consist of flat numbered paragraphs or plain text, [`AnnexContent::Paragraphs`]
/// is used instead.
#[derive(Debug, PartialEq, Serialize)]
pub struct AnnexSection {
    /// Section heading (from `<TITLE><TI>`).
    pub title: String,
    /// Content items nested inside this section.
    pub alineas: Vec<Subparagraph>,
    /// Structured citations to other EU acts found in this section.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub citations: Vec<Citation>,
}

/// Discriminates the top-level structure of an annex.
#[derive(Serialize)]
pub enum AnnexContent {
    /// The annex is divided into titled sections (`<GR.SEQ>`).
    Sections(Vec<AnnexSection>),
    /// The annex contains flat content: numbered paragraphs, lists, or plain text.
    Paragraphs(Vec<AnnexParagraph>),
}

/// A parsed annex file (`<ANNEX>`).
#[derive(Serialize)]
pub struct Annex {
    /// Annex identifier, e.g. `"ANNEX I"` (from `<TITLE><TI>`).
    pub number: String,
    /// Optional descriptive subtitle (from `<TITLE><STI>`); present only in some annexes.
    pub subtitle: Option<String>,
    /// Top-level content: either titled sections or flat paragraphs.
    pub content: AnnexContent,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta() -> Metadata {
        Metadata {
            celex: None,
            document_date: None,
            legal_value: None,
            language: None,
            authors: vec![],
            eea_relevant: false,
            official_journal: None,
            page_first: None,
            page_last: None,
            page_total: None,
            prod_id: None,
            fin_id: None,
        }
    }

    fn empty_enacting_terms() -> EnactingTerms {
        EnactingTerms { content: EnactingTermsContent::Chapters(vec![]) }
    }

    fn regular_act(title: &str) -> Act {
        Act::Regular(RegularAct {
            metadata: meta(),
            title: title.into(),
            preamble: Preamble {
                init: String::new(),
                visas: vec![],
                recitals: vec![],
                enacting_formula: String::new(),
            },
            enacting_terms: empty_enacting_terms(),
            annexes: vec![],
            definitions: HashMap::new(),
        })
    }

    fn consolidated_act(title: &str) -> Act {
        Act::Consolidated(ConsolidatedAct {
            metadata: meta(),
            title: title.into(),
            preamble: ConsolidatedPreamble {
                init: String::new(),
                enacting_formula: String::new(),
            },
            enacting_terms: empty_enacting_terms(),
            annexes: vec![],
            definitions: HashMap::new(),
        })
    }

    // ── title() ───────────────────────────────────────────────────────────────

    #[test]
    fn title_regular() {
        let act = regular_act("Regulation (EU) 2024/1689");
        assert_eq!(act.title(), "Regulation (EU) 2024/1689");
    }

    #[test]
    fn title_consolidated() {
        let act = consolidated_act("Regulation (EU) 2017/1001");
        assert_eq!(act.title(), "Regulation (EU) 2017/1001");
    }

    // ── metadata() ────────────────────────────────────────────────────────────

    #[test]
    fn metadata_regular_celex() {
        let mut act = regular_act("");
        if let Act::Regular(ref mut r) = act {
            r.metadata.celex = Some("32024R1689".into());
        }
        assert_eq!(act.metadata().celex.as_deref(), Some("32024R1689"));
    }

    #[test]
    fn metadata_consolidated_eea_relevant() {
        let mut act = consolidated_act("");
        if let Act::Consolidated(ref mut c) = act {
            c.metadata.eea_relevant = true;
        }
        assert!(act.metadata().eea_relevant);
    }

    // ── enacting_terms() ──────────────────────────────────────────────────────

    #[test]
    fn enacting_terms_regular_empty_chapters() {
        let act = regular_act("");
        assert!(matches!(
            &act.enacting_terms().content,
            EnactingTermsContent::Chapters(ch) if ch.is_empty()
        ));
    }

    #[test]
    fn enacting_terms_consolidated_empty_chapters() {
        let act = consolidated_act("");
        assert!(matches!(
            &act.enacting_terms().content,
            EnactingTermsContent::Chapters(ch) if ch.is_empty()
        ));
    }

    // ── annexes() ─────────────────────────────────────────────────────────────

    #[test]
    fn annexes_regular_empty() {
        let act = regular_act("");
        assert!(act.annexes().is_empty());
    }

    #[test]
    fn annexes_consolidated_empty() {
        let act = consolidated_act("");
        assert!(act.annexes().is_empty());
    }

    #[test]
    fn annexes_regular_populated() {
        let mut act = regular_act("");
        if let Act::Regular(ref mut r) = act {
            r.annexes.push(Annex {
                number: "ANNEX I".into(),
                subtitle: None,
                content: AnnexContent::Paragraphs(vec![]),
            });
        }
        assert_eq!(act.annexes().len(), 1);
        assert_eq!(act.annexes()[0].number, "ANNEX I");
    }

    #[test]
    fn annexes_consolidated_populated() {
        let mut act = consolidated_act("");
        if let Act::Consolidated(ref mut c) = act {
            c.annexes.push(Annex {
                number: "ANNEX II".into(),
                subtitle: Some("Technical requirements".into()),
                content: AnnexContent::Paragraphs(vec![]),
            });
        }
        assert_eq!(act.annexes().len(), 1);
        assert_eq!(act.annexes()[0].number, "ANNEX II");
        assert_eq!(
            act.annexes()[0].subtitle.as_deref(),
            Some("Technical requirements")
        );
    }

    // ── definitions() ─────────────────────────────────────────────────────────

    #[test]
    fn definitions_regular_empty() {
        let act = regular_act("");
        assert!(act.definitions().is_empty());
    }

    #[test]
    fn definitions_consolidated_empty() {
        let act = consolidated_act("");
        assert!(act.definitions().is_empty());
    }

    #[test]
    fn definitions_regular_populated() {
        let mut act = regular_act("");
        if let Act::Regular(ref mut r) = act {
            r.definitions
                .insert("AI system".into(), "a machine-based system…".into());
        }
        assert_eq!(
            act.definitions().get("AI system").map(String::as_str),
            Some("a machine-based system…")
        );
    }

    #[test]
    fn definitions_consolidated_populated() {
        let mut act = consolidated_act("");
        if let Act::Consolidated(ref mut c) = act {
            c.definitions
                .insert("trade mark".into(), "a sign capable of…".into());
        }
        assert_eq!(
            act.definitions().get("trade mark").map(String::as_str),
            Some("a sign capable of…")
        );
    }
}
