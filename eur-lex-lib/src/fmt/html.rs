use crate::model::{
    Act, Alinea, Annex, AnnexContent, AnnexSection, Article, ArticleContent, Chapter,
    ChapterContents, ConsolidatedAct, ConsolidatedPreamble, EnactingTerms, EnactingTermsContent,
    Item, ItemContent, LegalParagraph, ListBlock, ListType, Metadata, PhysicalNumberedParagraph,
    Preamble, Recital, RegularAct, Section, Subdivision, SubdivisionContent, Subparagraph, Table,
};

const STYLE: &str = r#"<style>
  body { font-family: Georgia, serif; max-width: 900px; margin: 0 auto; padding: 2rem; line-height: 1.6; }
  h1 { font-size: 1.4rem; border-bottom: 2px solid #333; padding-bottom: 0.4rem; }
  h2 { font-size: 1.2rem; margin-top: 2rem; }
  h3 { font-size: 1.05rem; margin-top: 1.5rem; }
  h4 { font-size: 1rem; margin-top: 1rem; }
  .metadata { color: #555; font-size: 0.9rem; margin-bottom: 1rem; }
  .para-number { font-weight: bold; margin-bottom: 0.2rem; }
  blockquote { border-left: 3px solid #bbb; margin: 0.5rem 0 0.5rem 1rem; padding-left: 1rem; color: #444; }
  table { border-collapse: collapse; width: 100%; margin: 1rem 0; }
  th, td { border: 1px solid #ccc; padding: 0.4rem 0.6rem; text-align: left; vertical-align: top; }
  th { background: #f0f0f0; font-weight: bold; }
  caption { font-weight: bold; margin-bottom: 0.4rem; caption-side: top; }
  hr { border: none; border-top: 1px solid #ddd; margin: 2rem 0; }
  .preamble-formula { font-style: italic; }
</style>"#;

// ── Top-level rendering ───────────────────────────────────────────────────────

/// Renders a parsed EU legislative act as a self-contained HTML5 document.
pub fn render_act(act: &Act) -> String {
    match act {
        Act::Regular(a) => render_regular(a),
        Act::Consolidated(a) => render_consolidated(a),
    }
}

fn html_doc(title: &str, body: String) -> String {
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"UTF-8\">\n  <title>{}</title>\n{STYLE}\n</head>\n<body>\n{}</body>\n</html>\n",
        escape(title),
        body
    )
}

fn render_regular(act: &RegularAct) -> String {
    let mut body = format!("<h1>{}</h1>\n", escape(&act.title));
    let meta = render_metadata(&act.metadata);
    if !meta.is_empty() {
        body.push_str(&meta);
    }
    body.push_str("<hr>\n");
    body.push_str(&render_preamble_section(&act.preamble));
    body.push_str("<hr>\n");
    body.push_str(&render_enacting_terms(&act.enacting_terms));
    if !act.annexes.is_empty() {
        body.push_str("<hr>\n");
        body.push_str(&render_annexes(&act.annexes));
    }
    html_doc(&act.title, body)
}

fn render_consolidated(act: &ConsolidatedAct) -> String {
    let mut body = format!("<h1>{}</h1>\n", escape(&act.title));
    let meta = render_metadata(&act.metadata);
    if !meta.is_empty() {
        body.push_str(&meta);
    }
    body.push_str("<hr>\n");
    body.push_str(&render_consolidated_preamble_section(&act.preamble));
    body.push_str("<hr>\n");
    body.push_str(&render_enacting_terms(&act.enacting_terms));
    if !act.annexes.is_empty() {
        body.push_str("<hr>\n");
        body.push_str(&render_annexes(&act.annexes));
    }
    html_doc(&act.title, body)
}

// ── Metadata ─────────────────────────────────────────────────────────────────

fn render_metadata(meta: &Metadata) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(ref v) = meta.celex {
        parts.push(format!("<strong>CELEX:</strong> {}", escape(v)));
    }
    if let Some(ref v) = meta.document_date {
        parts.push(format!("<strong>Date:</strong> {}", escape(&format_date(v))));
    }
    if let Some(ref v) = meta.language {
        parts.push(format!("<strong>Language:</strong> {}", escape(v)));
    }
    if let Some(ref v) = meta.legal_value {
        parts.push(format!("<strong>Type:</strong> {}", escape(v)));
    }
    if parts.is_empty() {
        return String::new();
    }
    format!("<p class=\"metadata\">{}</p>\n", parts.join(" | "))
}

fn format_date(raw: &str) -> String {
    if raw.len() == 8 {
        format!("{}-{}-{}", &raw[..4], &raw[4..6], &raw[6..8])
    } else {
        raw.to_string()
    }
}

// ── HTML escaping ─────────────────────────────────────────────────────────────

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ── Preamble ──────────────────────────────────────────────────────────────────

pub(super) fn render_preamble_section(p: &Preamble) -> String {
    let mut out = "<section class=\"preamble\">\n<h2>Preamble</h2>\n".to_string();
    if !p.init.is_empty() {
        out.push_str(&format!("<p>{}</p>\n", escape(&p.init)));
    }
    if !p.visas.is_empty() {
        out.push_str("<blockquote>\n");
        for visa in &p.visas {
            out.push_str(&format!("<p>{}</p>\n", escape(visa)));
        }
        out.push_str("</blockquote>\n");
    }
    for recital in &p.recitals {
        out.push_str(&render_recital(recital));
    }
    if !p.enacting_formula.is_empty() {
        out.push_str(&format!(
            "<p class=\"preamble-formula\">{}</p>\n",
            escape(&p.enacting_formula)
        ));
    }
    out.push_str("</section>\n");
    out
}

pub(super) fn render_recital(r: &Recital) -> String {
    format!("<p><strong>{}</strong> {}</p>\n", escape(&r.number), escape(&r.text))
}

fn render_consolidated_preamble_section(p: &ConsolidatedPreamble) -> String {
    let mut out = "<section class=\"preamble\">\n<h2>Preamble</h2>\n".to_string();
    if !p.init.is_empty() {
        out.push_str(&format!("<p>{}</p>\n", escape(&p.init)));
    }
    if !p.enacting_formula.is_empty() {
        out.push_str(&format!(
            "<p class=\"preamble-formula\">{}</p>\n",
            escape(&p.enacting_formula)
        ));
    }
    out.push_str("</section>\n");
    out
}

// ── Enacting terms ────────────────────────────────────────────────────────────

pub(super) fn render_enacting_terms(et: &EnactingTerms) -> String {
    let mut out = "<section class=\"enacting-terms\">\n".to_string();
    match &et.content {
        EnactingTermsContent::Chapters(chapters) => {
            for ch in chapters {
                out.push_str(&render_chapter(ch));
            }
        }
        EnactingTermsContent::Articles(articles) => {
            for art in articles {
                out.push_str(&render_article(art));
            }
        }
    }
    out.push_str("</section>\n");
    out
}

pub(super) fn render_chapter(chapter: &Chapter) -> String {
    let heading = match &chapter.subtitle {
        Some(sub) => format!("{} — {}", escape(&chapter.title), escape(sub)),
        None => escape(&chapter.title),
    };
    let mut out = format!("<section class=\"chapter\">\n<h2>{heading}</h2>\n");
    match &chapter.contents {
        ChapterContents::Sections(sections) => {
            for sec in sections {
                out.push_str(&render_section(sec));
            }
        }
        ChapterContents::Articles(articles) => {
            for art in articles {
                out.push_str(&render_article(art));
            }
        }
    }
    out.push_str("</section>\n");
    out
}

pub(super) fn render_section(section: &Section) -> String {
    let heading = match &section.subtitle {
        Some(sub) => format!("{} — {}", escape(&section.title), escape(sub)),
        None => escape(&section.title),
    };
    let mut out = format!("<section class=\"section\">\n<h3>{heading}</h3>\n");
    for art in &section.articles {
        out.push_str(&render_article(art));
    }
    out.push_str("</section>\n");
    out
}

pub(super) fn render_article(article: &Article) -> String {
    let heading = match &article.title {
        Some(t) => format!("{} — {}", escape(&article.number), escape(t)),
        None => escape(&article.number),
    };
    let mut out = format!("<section class=\"article\">\n<h4>{heading}</h4>\n");
    let body = match &article.content {
        ArticleContent::Paragraphs(paras) => {
            paras.iter().map(render_legal_paragraph).collect::<String>()
        }
        ArticleContent::Alineas(alineas) => {
            alineas.iter().map(render_alinea).collect::<String>()
        }
        ArticleContent::Subdivisions(subdivs) => {
            subdivs.iter().map(render_subdivision).collect::<String>()
        }
    };
    out.push_str(&body);
    out.push_str("</section>\n");
    out
}

fn render_legal_paragraph(p: &LegalParagraph) -> String {
    let mut out = format!("<p class=\"para-number\">{}</p>\n", escape(&p.number));
    for alinea in &p.alineas {
        out.push_str(&render_alinea(alinea));
    }
    out
}

fn render_alinea(a: &Alinea) -> String {
    a.content.iter().map(render_subparagraph).collect()
}

fn render_subdivision(subdiv: &Subdivision) -> String {
    let mut out = format!("<p><strong>{}</strong></p>\n", escape(&subdiv.title));
    let body = match &subdiv.content {
        SubdivisionContent::Paragraphs(paras) => {
            paras.iter().map(render_legal_paragraph).collect::<String>()
        }
        SubdivisionContent::Alineas(alineas) => {
            alineas.iter().map(render_alinea).collect::<String>()
        }
        SubdivisionContent::Subdivisions(subdivs) => {
            subdivs.iter().map(render_subdivision).collect::<String>()
        }
    };
    out.push_str(&body);
    out
}

// ── Block content ─────────────────────────────────────────────────────────────

fn render_subparagraph(sp: &Subparagraph) -> String {
    match sp {
        Subparagraph::Plain(text) => format!("<p>{}</p>\n", escape(text)),
        Subparagraph::List(list) => render_list(list),
        Subparagraph::Table(table) => render_table(table),
        Subparagraph::Numbered(np) => render_numbered_paragraph(np),
    }
}

fn render_numbered_paragraph(np: &PhysicalNumberedParagraph) -> String {
    let mut out = format!("<p class=\"para-number\">{}</p>\n", escape(&np.number));
    for sp in &np.alineas {
        out.push_str(&render_subparagraph(sp));
    }
    out
}

// ── Lists ─────────────────────────────────────────────────────────────────────

fn list_tag(list_type: &Option<ListType>) -> (&'static str, Option<&'static str>) {
    match list_type {
        None
        | Some(ListType::Bullet)
        | Some(ListType::Dash)
        | Some(ListType::Ndash)
        | Some(ListType::NoPrefix)
        | Some(ListType::Other) => ("ul", None),
        Some(ListType::Arab) => ("ol", Some("1")),
        Some(ListType::Alpha) => ("ol", Some("a")),
        Some(ListType::AlphaUpper) => ("ol", Some("A")),
        Some(ListType::Roman) => ("ol", Some("i")),
        Some(ListType::RomanUpper) => ("ol", Some("I")),
    }
}

fn render_list(list: &ListBlock) -> String {
    let mut out = String::new();
    if !list.intro.is_empty() {
        out.push_str(&format!("<p>{}</p>\n", escape(&list.intro)));
    }
    let (tag, type_attr) = list_tag(&list.list_type);
    match type_attr {
        Some(t) => out.push_str(&format!("<{tag} type=\"{t}\">\n")),
        None => out.push_str(&format!("<{tag}>\n")),
    }
    for item in &list.items {
        out.push_str(&render_item(item));
    }
    out.push_str(&format!("</{tag}>\n"));
    out
}

fn render_item(item: &Item) -> String {
    match &item.content {
        ItemContent::Plain(text) => format!("<li>{}</li>\n", escape(text)),
        ItemContent::List(nested) => {
            let mut out = "<li>".to_string();
            if !nested.intro.is_empty() {
                out.push_str(&escape(&nested.intro));
            }
            out.push('\n');
            let (tag, type_attr) = list_tag(&nested.list_type);
            match type_attr {
                Some(t) => out.push_str(&format!("<{tag} type=\"{t}\">\n")),
                None => out.push_str(&format!("<{tag}>\n")),
            }
            for nested_item in &nested.items {
                out.push_str(&render_item(nested_item));
            }
            out.push_str(&format!("</{tag}>\n"));
            out.push_str("</li>\n");
            out
        }
    }
}

// ── Tables ────────────────────────────────────────────────────────────────────

fn render_table(table: &Table) -> String {
    if table.rows.is_empty() {
        return String::new();
    }
    let mut out = "<table>\n".to_string();
    if let Some(ref title) = table.title {
        out.push_str(&format!("<caption>{}</caption>\n", escape(title)));
    }
    let mut rows = table.rows.iter().peekable();
    // First row or rows marked is_header go into <thead>.
    if rows.peek().map(|r| r.is_header).unwrap_or(false) {
        out.push_str("<thead>\n");
        while rows.peek().map(|r| r.is_header).unwrap_or(false) {
            let row = rows.next().unwrap();
            out.push_str("<tr>\n");
            for cell in &row.cells {
                let tag = if cell.is_header { "th" } else { "td" };
                out.push_str(&format!("<{tag}>{}</{tag}>\n", escape(&cell.text)));
            }
            out.push_str("</tr>\n");
        }
        out.push_str("</thead>\n");
    }
    if rows.peek().is_some() {
        out.push_str("<tbody>\n");
        for row in rows {
            out.push_str("<tr>\n");
            for cell in &row.cells {
                let tag = if cell.is_header { "th" } else { "td" };
                out.push_str(&format!("<{tag}>{}</{tag}>\n", escape(&cell.text)));
            }
            out.push_str("</tr>\n");
        }
        out.push_str("</tbody>\n");
    }
    out.push_str("</table>\n");
    out
}

// ── Annexes ───────────────────────────────────────────────────────────────────

fn render_annexes(annexes: &[Annex]) -> String {
    annexes.iter().map(render_annex).collect()
}

pub(super) fn render_annex(annex: &Annex) -> String {
    let heading = match &annex.subtitle {
        Some(sub) => format!("{} — {}", escape(&annex.number), escape(sub)),
        None => escape(&annex.number),
    };
    let mut out = format!("<section class=\"annex\">\n<h2>{heading}</h2>\n");
    match &annex.content {
        AnnexContent::Sections(sections) => {
            for sec in sections {
                out.push_str(&render_annex_section(sec));
            }
        }
        AnnexContent::Paragraphs(paras) => {
            for sp in paras {
                out.push_str(&render_subparagraph(sp));
            }
        }
    }
    out.push_str("</section>\n");
    out
}

pub(super) fn render_annex_section(section: &AnnexSection) -> String {
    let mut out = format!("<section class=\"annex-section\">\n<h3>{}</h3>\n", escape(&section.title));
    for sp in &section.alineas {
        out.push_str(&render_subparagraph(sp));
    }
    out.push_str("</section>\n");
    out
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::loader::load_act;
    use crate::model::{
        Alinea, Annex, AnnexContent, AnnexSection, Article, ArticleContent, Cell, Chapter,
        ChapterContents, Item, ItemContent, LegalParagraph, ListBlock, ListType, Metadata,
        Preamble, Recital, Row, Section, Subparagraph, Table,
    };

    use super::*;

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn empty_metadata() -> Metadata {
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

    fn plain(text: &str) -> Subparagraph {
        Subparagraph::Plain(text.to_string())
    }

    fn alinea(text: &str) -> Alinea {
        Alinea { content: vec![plain(text)], citations: vec![] }
    }

    fn simple_article(number: &str, title: Option<&str>, text: &str) -> Article {
        Article {
            number: number.to_string(),
            title: title.map(str::to_string),
            content: ArticleContent::Alineas(vec![alinea(text)]),
        }
    }

    fn make_row(texts: &[&str], is_header: bool) -> Row {
        let cells =
            texts.iter().map(|t| Cell { text: t.to_string(), is_header }).collect::<Vec<_>>();
        let cell_count = cells.len();
        Row { cells, cell_count, is_header }
    }

    // ── escape ────────────────────────────────────────────────────────────────

    #[test]
    /// Verifies that `<`, `>`, `&`, and `"` are escaped to their HTML entity equivalents.
    fn escape_special_chars() {
        assert_eq!(escape("<b>AT&T \"inc\"</b>"), "&lt;b&gt;AT&amp;T &quot;inc&quot;&lt;/b&gt;");
    }

    #[test]
    /// Plain ASCII with no special characters passes through unchanged.
    fn escape_plain_text_unchanged() {
        assert_eq!(escape("hello world"), "hello world");
    }

    // ── format_date ───────────────────────────────────────────────────────────

    #[test]
    /// An 8-character YYYYMMDD string is reformatted as YYYY-MM-DD.
    fn format_date_yyyymmdd() {
        assert_eq!(format_date("20220101"), "2022-01-01");
    }

    #[test]
    /// Strings that are not exactly 8 characters pass through unchanged.
    fn format_date_passthrough_when_not_8_chars() {
        assert_eq!(format_date("2022-10"), "2022-10");
        assert_eq!(format_date(""), "");
    }

    // ── render_metadata ───────────────────────────────────────────────────────

    #[test]
    /// All-empty metadata produces no output (empty string, not a blank `<p>`).
    fn metadata_empty_produces_no_element() {
        assert_eq!(render_metadata(&empty_metadata()), "");
    }

    #[test]
    /// All four metadata fields render as a `<p class="metadata">` with `<strong>` labels.
    fn metadata_all_fields() {
        let meta = Metadata {
            celex: Some("32022R2065".to_string()),
            document_date: Some("20221019".to_string()),
            language: Some("EN".to_string()),
            legal_value: Some("REG".to_string()),
            ..empty_metadata()
        };
        let out = render_metadata(&meta);
        assert!(out.starts_with("<p class=\"metadata\">"));
        assert!(out.contains("<strong>CELEX:</strong> 32022R2065"));
        assert!(out.contains("<strong>Date:</strong> 2022-10-19"));
        assert!(out.contains("<strong>Language:</strong> EN"));
        assert!(out.contains("<strong>Type:</strong> REG"));
    }

    #[test]
    /// Metadata text containing `&` or `<` is HTML-escaped in the output.
    fn metadata_values_are_escaped() {
        let meta = Metadata {
            celex: Some("A&B<C".to_string()),
            ..empty_metadata()
        };
        let out = render_metadata(&meta);
        assert!(out.contains("A&amp;B&lt;C"));
        assert!(!out.contains("A&B<C"));
    }

    // ── list_tag ─────────────────────────────────────────────────────────────

    #[test]
    /// Bullet, Dash, and None list types map to `<ul>` with no type attribute.
    fn list_tag_unordered() {
        assert_eq!(list_tag(&Some(ListType::Bullet)), ("ul", None));
        assert_eq!(list_tag(&Some(ListType::Dash)), ("ul", None));
        assert_eq!(list_tag(&None), ("ul", None));
    }

    #[test]
    /// Arab, Alpha, AlphaUpper, Roman, and RomanUpper map to `<ol>` with the correct type attribute.
    fn list_tag_ordered_variants() {
        assert_eq!(list_tag(&Some(ListType::Arab)), ("ol", Some("1")));
        assert_eq!(list_tag(&Some(ListType::Alpha)), ("ol", Some("a")));
        assert_eq!(list_tag(&Some(ListType::AlphaUpper)), ("ol", Some("A")));
        assert_eq!(list_tag(&Some(ListType::Roman)), ("ol", Some("i")));
        assert_eq!(list_tag(&Some(ListType::RomanUpper)), ("ol", Some("I")));
    }

    // ── render_list ───────────────────────────────────────────────────────────

    #[test]
    /// An alpha-ordered list with an intro renders an intro `<p>` followed by `<ol type="a">`.
    fn list_ordered_alpha_with_intro() {
        let list = ListBlock {
            list_type: Some(ListType::Alpha),
            intro: "The following apply:".to_string(),
            items: vec![
                Item { number: 1, content: ItemContent::Plain("first".to_string()) },
                Item { number: 2, content: ItemContent::Plain("second".to_string()) },
            ],
        };
        let out = render_list(&list);
        assert!(out.contains("<p>The following apply:</p>"));
        assert!(out.contains("<ol type=\"a\">"));
        assert!(out.contains("<li>first</li>"));
        assert!(out.contains("<li>second</li>"));
    }

    #[test]
    /// A bullet list with no intro renders only `<ul>` and `<li>` elements.
    fn list_unordered_no_intro() {
        let list = ListBlock {
            list_type: Some(ListType::Bullet),
            intro: String::new(),
            items: vec![Item { number: 1, content: ItemContent::Plain("item".to_string()) }],
        };
        let out = render_list(&list);
        assert!(!out.contains("<p>"));
        assert!(out.contains("<ul>"));
        assert!(out.contains("<li>item</li>"));
    }

    #[test]
    /// A nested list renders as a `<li>` wrapping the intro text and a child ordered list.
    fn list_nested_renders_child_list() {
        let inner = ListBlock {
            list_type: Some(ListType::Arab),
            intro: "nested intro".to_string(),
            items: vec![Item { number: 1, content: ItemContent::Plain("sub-item".to_string()) }],
        };
        let outer = ListBlock {
            list_type: Some(ListType::Alpha),
            intro: String::new(),
            items: vec![Item { number: 1, content: ItemContent::List(inner) }],
        };
        let out = render_list(&outer);
        assert!(out.contains("<ol type=\"a\">"));
        assert!(out.contains("<li>nested intro"));
        assert!(out.contains("<ol type=\"1\">"));
        assert!(out.contains("<li>sub-item</li>"));
    }

    // ── render_table ─────────────────────────────────────────────────────────

    #[test]
    /// A table whose first row is a header renders `<thead>` with `<th>` cells and `<tbody>` for the rest.
    fn table_header_row_uses_thead_and_th() {
        let table = Table {
            col_count: 2,
            title: None,
            rows: vec![make_row(&["Name", "Value"], true), make_row(&["foo", "bar"], false)],
            row_count: 2,
        };
        let out = render_table(&table);
        assert!(out.contains("<thead>"));
        assert!(out.contains("<th>Name</th>"));
        assert!(out.contains("<th>Value</th>"));
        assert!(out.contains("<tbody>"));
        assert!(out.contains("<td>foo</td>"));
        assert!(out.contains("<td>bar</td>"));
    }

    #[test]
    /// A table with a title renders a `<caption>` element.
    fn table_with_title_renders_caption() {
        let table = Table {
            col_count: 1,
            title: Some("Correlation table".to_string()),
            rows: vec![make_row(&["Header"], true), make_row(&["Row"], false)],
            row_count: 2,
        };
        let out = render_table(&table);
        assert!(out.contains("<caption>Correlation table</caption>"));
    }

    #[test]
    /// Text in table cells that contains `<`, `>`, or `&` is HTML-escaped.
    fn table_cell_text_is_escaped() {
        let table = Table {
            col_count: 1,
            title: None,
            rows: vec![make_row(&["a<b>&c"], false)],
            row_count: 1,
        };
        let out = render_table(&table);
        assert!(out.contains("a&lt;b&gt;&amp;c"));
    }

    #[test]
    /// An empty table (no rows) produces an empty string, not a bare `<table>`.
    fn table_empty_returns_empty_string() {
        let table = Table { col_count: 0, title: None, rows: vec![], row_count: 0 };
        assert_eq!(render_table(&table), "");
    }

    // ── render_article ────────────────────────────────────────────────────────

    #[test]
    /// An article without a title renders its number alone inside `<h4>`.
    fn article_heading_no_title() {
        let art = simple_article("Article 1", None, "text");
        assert!(render_article(&art).contains("<h4>Article 1</h4>"));
    }

    #[test]
    /// An article with a title renders `number — title` inside `<h4>`.
    fn article_heading_with_title() {
        let art = simple_article("Article 6", Some("Scope"), "text");
        assert!(render_article(&art).contains("<h4>Article 6 — Scope</h4>"));
    }

    #[test]
    /// Each numbered paragraph renders a `<p class="para-number">` before its body.
    fn article_paragraph_number_element() {
        let art = Article {
            number: "Article 2".to_string(),
            title: None,
            content: ArticleContent::Paragraphs(vec![LegalParagraph {
                number: "1.".to_string(),
                alineas: vec![alinea("text")],
                citations: vec![],
            }]),
        };
        assert!(render_article(&art).contains("<p class=\"para-number\">1.</p>"));
    }

    // ── render_chapter / render_section ──────────────────────────────────────

    #[test]
    /// A chapter without a subtitle renders its title alone inside `<h2>`.
    fn chapter_heading_no_subtitle() {
        let ch = Chapter {
            title: "CHAPTER I".to_string(),
            subtitle: None,
            contents: ChapterContents::Articles(vec![simple_article("Article 1", None, "t")]),
        };
        assert!(render_chapter(&ch).contains("<h2>CHAPTER I</h2>"));
    }

    #[test]
    /// A chapter with a subtitle renders `title — subtitle` inside `<h2>`.
    fn chapter_heading_with_subtitle() {
        let ch = Chapter {
            title: "CHAPTER II".to_string(),
            subtitle: Some("General provisions".to_string()),
            contents: ChapterContents::Articles(vec![simple_article("Article 2", None, "t")]),
        };
        assert!(render_chapter(&ch).contains("<h2>CHAPTER II — General provisions</h2>"));
    }

    #[test]
    /// A section renders its title inside `<h3>`.
    fn section_heading_uses_h3() {
        let sec = Section {
            title: "SECTION 1".to_string(),
            subtitle: Some("Definitions".to_string()),
            articles: vec![simple_article("Article 1", None, "t")],
        };
        assert!(render_section(&sec).contains("<h3>SECTION 1 — Definitions</h3>"));
    }

    // ── render_preamble_section ───────────────────────────────────────────────

    #[test]
    /// Visas render as `<p>` elements inside a `<blockquote>`, not plain paragraphs.
    fn preamble_visas_in_blockquote() {
        let p = Preamble {
            init: String::new(),
            visas: vec!["Having regard to the Treaty,".to_string()],
            recitals: vec![],
            enacting_formula: String::new(),
        };
        let out = render_preamble_section(&p);
        assert!(out.contains("<blockquote>"));
        assert!(out.contains("<p>Having regard to the Treaty,</p>"));
    }

    #[test]
    /// Recital numbers render as `<strong>` and the text as plain content in the same `<p>`.
    fn preamble_recitals_with_strong_number() {
        let p = Preamble {
            init: String::new(),
            visas: vec![],
            recitals: vec![
                Recital { number: "(1)".to_string(), text: "First.".to_string(), citations: vec![] },
                Recital { number: "(2)".to_string(), text: "Second.".to_string(), citations: vec![] },
            ],
            enacting_formula: String::new(),
        };
        let out = render_preamble_section(&p);
        assert!(out.contains("<strong>(1)</strong> First."));
        assert!(out.contains("<strong>(2)</strong> Second."));
    }

    // ── render_annex ─────────────────────────────────────────────────────────

    #[test]
    /// An annex without a subtitle renders its number alone inside `<h2>`.
    fn annex_heading_no_subtitle() {
        let annex = Annex {
            number: "ANNEX I".to_string(),
            subtitle: None,
            content: AnnexContent::Paragraphs(vec![plain("content")]),
        };
        assert!(render_annex(&annex).contains("<h2>ANNEX I</h2>"));
    }

    #[test]
    /// An annex section renders its title inside `<h3>`.
    fn annex_section_uses_h3() {
        let section = AnnexSection {
            title: "Part 1".to_string(),
            alineas: vec![plain("text")],
            citations: vec![],
        };
        assert!(render_annex_section(&section).contains("<h3>Part 1</h3>"));
    }

    // ── Integration: render_act against real fixtures ─────────────────────────

    #[test]
    /// The full DSA renders as a valid HTML5 document with correct structure markers.
    fn dsa_renders_as_html_document() {
        let act = load_act(Path::new("../data/32022R2065")).expect("failed to load DSA fixture");
        let out = render_act(&act);
        assert!(out.starts_with("<!DOCTYPE html>"));
        assert!(out.contains("<html lang=\"en\">"));
        assert!(out.contains("<title>"));
        assert!(out.contains("2022/2065"));
        assert!(out.contains("<h2>Preamble</h2>"));
        assert!(out.contains("<h2>CHAPTER"));
        assert!(out.contains("<h4>Article 1"));
        assert!(out.ends_with("</html>\n"));
    }

    #[test]
    /// Visa text in the DSA appears inside a `<blockquote>`, recitals use `<strong>` for numbers.
    fn dsa_preamble_structure() {
        let act = load_act(Path::new("../data/32022R2065")).expect("failed to load DSA fixture");
        let out = render_act(&act);
        assert!(out.contains("<blockquote>"));
        assert!(out.contains("<strong>(1)</strong>"));
        assert!(out.contains("<strong>(156)</strong>"));
    }

    #[test]
    /// The EU AI Act renders all 13 chapter headings and annex headings in HTML.
    fn eu_ai_act_chapters_and_annexes_in_html() {
        let act =
            load_act(Path::new("../data/32024R1689")).expect("failed to load EU AI Act fixture");
        let out = render_act(&act);
        assert!(out.contains("<h2>CHAPTER"));
        assert!(out.contains("<h2>ANNEX"));
    }

    #[test]
    /// Plain text content in the DSA does not contain raw `<` or `>` outside of HTML tags.
    fn dsa_text_content_is_escaped() {
        let act = load_act(Path::new("../data/32022R2065")).expect("failed to load DSA fixture");
        let out = render_act(&act);
        // Strip all HTML tags and check the remaining text has no unescaped angle brackets.
        let text_only = out
            .split('<')
            .enumerate()
            .filter_map(|(i, chunk)| {
                if i == 0 {
                    Some(chunk.to_string())
                } else {
                    chunk.find('>').map(|pos| chunk[pos + 1..].to_string())
                }
            })
            .collect::<String>();
        assert!(!text_only.contains('<'), "found unescaped '<' in text content");
        assert!(!text_only.contains('>'), "found unescaped '>' in text content");
    }
}
