use crate::model::{
    Act, Alinea, Annex, AnnexContent, AnnexSection, Article, ArticleContent, Chapter,
    ChapterContents, ConsolidatedAct, ConsolidatedPreamble, EnactingTerms, EnactingTermsContent,
    Item, ItemContent, LegalParagraph, ListBlock, ListType, Metadata, PhysicalNumberedParagraph,
    Preamble, Recital, RegularAct, Section, Subdivision, SubdivisionContent, Subparagraph, Table,
};

// ── Top-level rendering ───────────────────────────────────────────────────────

pub fn render_act(act: &Act) -> String {
    match act {
        Act::Regular(a) => render_regular(a),
        Act::Consolidated(a) => render_consolidated(a),
    }
}

fn render_regular(act: &RegularAct) -> String {
    let mut out = format!("# {}\n\n", act.title);
    let meta = render_metadata_line(&act.metadata);
    if !meta.is_empty() {
        out.push_str(&meta);
        out.push_str("\n\n");
    }
    out.push_str("---\n\n## Preamble\n\n");
    out.push_str(&render_preamble(&act.preamble));
    out.push_str("\n\n---\n\n");
    out.push_str(&render_enacting_terms(&act.enacting_terms));
    if !act.annexes.is_empty() {
        out.push_str("\n\n---\n\n");
        out.push_str(&render_annexes(&act.annexes));
    }
    out
}

fn render_consolidated(act: &ConsolidatedAct) -> String {
    let mut out = format!("# {}\n\n", act.title);
    let meta = render_metadata_line(&act.metadata);
    if !meta.is_empty() {
        out.push_str(&meta);
        out.push_str("\n\n");
    }
    out.push_str("---\n\n## Preamble\n\n");
    out.push_str(&render_consolidated_preamble(&act.preamble));
    out.push_str("\n\n---\n\n");
    out.push_str(&render_enacting_terms(&act.enacting_terms));
    if !act.annexes.is_empty() {
        out.push_str("\n\n---\n\n");
        out.push_str(&render_annexes(&act.annexes));
    }
    out
}

// ── Metadata ─────────────────────────────────────────────────────────────────

fn render_metadata_line(meta: &Metadata) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(ref v) = meta.celex {
        parts.push(format!("**CELEX:** {v}"));
    }
    if let Some(ref v) = meta.document_date {
        parts.push(format!("**Date:** {}", format_date(v)));
    }
    if let Some(ref v) = meta.language {
        parts.push(format!("**Language:** {v}"));
    }
    if let Some(ref v) = meta.legal_value {
        parts.push(format!("**Type:** {v}"));
    }
    parts.join(" | ")
}

fn format_date(raw: &str) -> String {
    if raw.len() == 8 {
        format!("{}-{}-{}", &raw[..4], &raw[4..6], &raw[6..8])
    } else {
        raw.to_string()
    }
}

// ── Preamble ──────────────────────────────────────────────────────────────────

pub(super) fn render_preamble(p: &Preamble) -> String {
    let mut out = String::new();
    if !p.init.is_empty() {
        out.push_str(&p.init);
        out.push_str("\n\n");
    }
    for visa in &p.visas {
        out.push_str(&format!("> {visa}\n"));
    }
    if !p.visas.is_empty() {
        out.push('\n');
    }
    for recital in &p.recitals {
        out.push_str(&render_recital(recital));
    }
    if !p.enacting_formula.is_empty() {
        out.push_str(&p.enacting_formula);
    }
    out
}

pub(super) fn render_recital(r: &Recital) -> String {
    format!("**{}** {}\n\n", r.number, r.text)
}

fn render_consolidated_preamble(p: &ConsolidatedPreamble) -> String {
    let mut out = String::new();
    if !p.init.is_empty() {
        out.push_str(&p.init);
        out.push_str("\n\n");
    }
    if !p.enacting_formula.is_empty() {
        out.push_str(&p.enacting_formula);
    }
    out
}

// ── Enacting terms ────────────────────────────────────────────────────────────

pub(super) fn render_enacting_terms(et: &EnactingTerms) -> String {
    match &et.content {
        EnactingTermsContent::Chapters(chapters) => {
            chapters.iter().map(render_chapter).collect::<Vec<_>>().join("\n\n")
        }
        EnactingTermsContent::Articles(articles) => {
            articles.iter().map(render_article).collect::<Vec<_>>().join("\n\n")
        }
    }
}

pub(super) fn render_chapter(chapter: &Chapter) -> String {
    let heading = match &chapter.subtitle {
        Some(sub) => format!("## {} — {sub}", chapter.title),
        None => format!("## {}", chapter.title),
    };
    let body = match &chapter.contents {
        ChapterContents::Sections(sections) => {
            sections.iter().map(render_section).collect::<Vec<_>>().join("\n\n")
        }
        ChapterContents::Articles(articles) => {
            articles.iter().map(render_article).collect::<Vec<_>>().join("\n\n")
        }
    };
    format!("{heading}\n\n{body}")
}

pub(super) fn render_section(section: &Section) -> String {
    let heading = match &section.subtitle {
        Some(sub) => format!("### {} — {sub}", section.title),
        None => format!("### {}", section.title),
    };
    let body = section
        .articles
        .iter()
        .map(render_article)
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("{heading}\n\n{body}")
}

pub(super) fn render_article(article: &Article) -> String {
    let heading = match &article.title {
        Some(t) => format!("#### {} — {t}", article.number),
        None => format!("#### {}", article.number),
    };
    let body = match &article.content {
        ArticleContent::Paragraphs(paras) => paras
            .iter()
            .map(render_legal_paragraph)
            .collect::<Vec<_>>()
            .join("\n\n"),
        ArticleContent::Alineas(alineas) => alineas
            .iter()
            .map(render_alinea)
            .collect::<Vec<_>>()
            .join("\n\n"),
        ArticleContent::Subdivisions(subdivs) => subdivs
            .iter()
            .map(render_subdivision)
            .collect::<Vec<_>>()
            .join("\n\n"),
    };
    format!("{heading}\n\n{body}")
}

fn render_legal_paragraph(p: &LegalParagraph) -> String {
    let body = p
        .alineas
        .iter()
        .map(render_alinea)
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("{}\n\n{body}", p.number)
}

fn render_alinea(a: &Alinea) -> String {
    a.content
        .iter()
        .map(|sp| render_subparagraph(sp, 0))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_subdivision(subdiv: &Subdivision) -> String {
    let body = match &subdiv.content {
        SubdivisionContent::Paragraphs(paras) => paras
            .iter()
            .map(render_legal_paragraph)
            .collect::<Vec<_>>()
            .join("\n\n"),
        SubdivisionContent::Alineas(alineas) => alineas
            .iter()
            .map(render_alinea)
            .collect::<Vec<_>>()
            .join("\n\n"),
        SubdivisionContent::Subdivisions(subdivs) => subdivs
            .iter()
            .map(render_subdivision)
            .collect::<Vec<_>>()
            .join("\n\n"),
    };
    format!("**{}**\n\n{body}", subdiv.title)
}

// ── Block content ─────────────────────────────────────────────────────────────

fn render_subparagraph(sp: &Subparagraph, depth: usize) -> String {
    match sp {
        Subparagraph::Plain(text) => text.clone(),
        Subparagraph::List(list) => render_list(list, depth),
        Subparagraph::Table(table) => render_table(table),
        Subparagraph::Numbered(np) => render_numbered_paragraph(np, depth),
    }
}

fn render_numbered_paragraph(np: &PhysicalNumberedParagraph, depth: usize) -> String {
    let body = np
        .alineas
        .iter()
        .map(|sp| render_subparagraph(sp, depth))
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("**{}**\n\n{body}", np.number)
}

// ── Lists ─────────────────────────────────────────────────────────────────────

fn render_list(list: &ListBlock, depth: usize) -> String {
    let mut out = String::new();
    if !list.intro.is_empty() {
        out.push_str(&list.intro);
        out.push('\n');
    }
    for item in &list.items {
        out.push_str(&render_item(item, &list.list_type, depth));
    }
    out.trim_end().to_string()
}

fn render_item(item: &Item, list_type: &Option<ListType>, depth: usize) -> String {
    let indent = "  ".repeat(depth);
    let prefix = item_prefix(list_type, item.number);
    match &item.content {
        ItemContent::Plain(text) => format!("{indent}{prefix}{text}\n"),
        ItemContent::List(nested) => {
            let mut out = String::new();
            // The nested list's intro serves as this item's text.
            if !nested.intro.is_empty() {
                out.push_str(&format!("{indent}{prefix}{}\n", nested.intro));
            }
            for nested_item in &nested.items {
                out.push_str(&render_item(nested_item, &nested.list_type, depth + 1));
            }
            out
        }
    }
}

fn item_prefix(list_type: &Option<ListType>, n: u32) -> String {
    match list_type {
        None
        | Some(ListType::Bullet)
        | Some(ListType::Dash)
        | Some(ListType::Ndash)
        | Some(ListType::NoPrefix)
        | Some(ListType::Other) => "- ".to_string(),
        Some(ListType::Arab) => format!("{n}. "),
        Some(ListType::Alpha) => format!("{}. ", nth_alpha(n, b'a')),
        Some(ListType::AlphaUpper) => format!("{}. ", nth_alpha(n, b'A')),
        Some(ListType::Roman) => format!("{}. ", to_roman(n).to_lowercase()),
        Some(ListType::RomanUpper) => format!("{}. ", to_roman(n)),
    }
}

fn nth_alpha(n: u32, base: u8) -> char {
    char::from(base + (n.saturating_sub(1) as u8 % 26))
}

fn to_roman(n: u32) -> String {
    const VALS: &[(u32, &str)] = &[
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut n = n;
    let mut s = String::new();
    for &(v, sym) in VALS {
        while n >= v {
            s.push_str(sym);
            n -= v;
        }
    }
    s
}

// ── Tables ────────────────────────────────────────────────────────────────────

fn render_table(table: &Table) -> String {
    if table.rows.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    if let Some(ref title) = table.title {
        out.push_str(&format!("**{title}**\n\n"));
    }
    let mut rows = table.rows.iter();
    // Use the first row as the Markdown header row.
    if let Some(first) = rows.next() {
        let cells: Vec<String> = first
            .cells
            .iter()
            .map(|c| c.text.replace('|', "\\|"))
            .collect();
        out.push_str(&format!("| {} |\n", cells.join(" | ")));
        let sep = vec!["---"; first.cells.len()];
        out.push_str(&format!("| {} |\n", sep.join(" | ")));
    }
    for row in rows {
        let cells: Vec<String> = row
            .cells
            .iter()
            .map(|c| c.text.replace('|', "\\|"))
            .collect();
        out.push_str(&format!("| {} |\n", cells.join(" | ")));
    }
    out.trim_end().to_string()
}

// ── Annexes ───────────────────────────────────────────────────────────────────

fn render_annexes(annexes: &[Annex]) -> String {
    annexes
        .iter()
        .map(render_annex)
        .collect::<Vec<_>>()
        .join("\n\n---\n\n")
}

pub(super) fn render_annex(annex: &Annex) -> String {
    let heading = match &annex.subtitle {
        Some(sub) => format!("## {} — {sub}", annex.number),
        None => format!("## {}", annex.number),
    };
    let body = match &annex.content {
        AnnexContent::Sections(sections) => sections
            .iter()
            .map(render_annex_section)
            .collect::<Vec<_>>()
            .join("\n\n"),
        AnnexContent::Paragraphs(paras) => paras
            .iter()
            .map(|sp| render_subparagraph(sp, 0))
            .collect::<Vec<_>>()
            .join("\n\n"),
    };
    format!("{heading}\n\n{body}")
}

pub(super) fn render_annex_section(section: &AnnexSection) -> String {
    let body = section
        .alineas
        .iter()
        .map(|sp| render_subparagraph(sp, 0))
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("### {}\n\n{body}", section.title)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::loader::load_act;
    use crate::model::{
        Alinea, Annex, AnnexContent, AnnexSection, Article, ArticleContent, Cell, Chapter,
        ChapterContents, Item, ItemContent, LegalParagraph, ListBlock, ListType, Metadata,
        Preamble, Recital, Row, Section, Subdivision, SubdivisionContent, Subparagraph, Table,
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

    // ── format_date ───────────────────────────────────────────────────────────

    #[test]
    fn format_date_yyyymmdd() {
        assert_eq!(format_date("20220101"), "2022-01-01");
    }

    #[test]
    fn format_date_passthrough_when_not_8_chars() {
        assert_eq!(format_date("2022-10"), "2022-10");
        assert_eq!(format_date(""), "");
    }

    // ── render_metadata_line ──────────────────────────────────────────────────

    #[test]
    fn metadata_empty() {
        assert_eq!(render_metadata_line(&empty_metadata()), "");
    }

    #[test]
    fn metadata_all_fields() {
        let meta = Metadata {
            celex: Some("32022R2065".to_string()),
            document_date: Some("20221019".to_string()),
            language: Some("EN".to_string()),
            legal_value: Some("REG".to_string()),
            ..empty_metadata()
        };
        assert_eq!(
            render_metadata_line(&meta),
            "**CELEX:** 32022R2065 | **Date:** 2022-10-19 | **Language:** EN | **Type:** REG"
        );
    }

    #[test]
    fn metadata_partial_fields() {
        let meta = Metadata {
            celex: Some("32024R1689".to_string()),
            language: Some("EN".to_string()),
            ..empty_metadata()
        };
        assert_eq!(
            render_metadata_line(&meta),
            "**CELEX:** 32024R1689 | **Language:** EN"
        );
    }

    // ── to_roman / nth_alpha ──────────────────────────────────────────────────

    #[test]
    fn roman_numerals() {
        assert_eq!(to_roman(1), "I");
        assert_eq!(to_roman(4), "IV");
        assert_eq!(to_roman(9), "IX");
        assert_eq!(to_roman(14), "XIV");
        assert_eq!(to_roman(40), "XL");
        assert_eq!(to_roman(90), "XC");
        assert_eq!(to_roman(399), "CCCXCIX");
    }

    #[test]
    fn alpha_lower() {
        assert_eq!(nth_alpha(1, b'a'), 'a');
        assert_eq!(nth_alpha(3, b'a'), 'c');
        assert_eq!(nth_alpha(26, b'a'), 'z');
    }

    #[test]
    fn alpha_upper() {
        assert_eq!(nth_alpha(1, b'A'), 'A');
        assert_eq!(nth_alpha(3, b'A'), 'C');
    }

    // ── item_prefix ───────────────────────────────────────────────────────────

    #[test]
    fn item_prefix_arab() {
        assert_eq!(item_prefix(&Some(ListType::Arab), 3), "3. ");
    }

    #[test]
    fn item_prefix_alpha_lower() {
        assert_eq!(item_prefix(&Some(ListType::Alpha), 1), "a. ");
        assert_eq!(item_prefix(&Some(ListType::Alpha), 3), "c. ");
    }

    #[test]
    fn item_prefix_alpha_upper() {
        assert_eq!(item_prefix(&Some(ListType::AlphaUpper), 2), "B. ");
    }

    #[test]
    fn item_prefix_roman_upper() {
        assert_eq!(item_prefix(&Some(ListType::RomanUpper), 4), "IV. ");
    }

    #[test]
    fn item_prefix_roman_lower() {
        assert_eq!(item_prefix(&Some(ListType::Roman), 9), "ix. ");
    }

    #[test]
    fn item_prefix_bullet_and_dash() {
        assert_eq!(item_prefix(&Some(ListType::Bullet), 1), "- ");
        assert_eq!(item_prefix(&Some(ListType::Dash), 1), "- ");
        assert_eq!(item_prefix(&Some(ListType::Ndash), 1), "- ");
        assert_eq!(item_prefix(&None, 1), "- ");
    }

    // ── render_list ───────────────────────────────────────────────────────────

    #[test]
    fn list_plain_items() {
        let list = ListBlock {
            list_type: Some(ListType::Alpha),
            intro: "The following apply:".to_string(),
            items: vec![
                Item { number: 1, content: ItemContent::Plain("first".to_string()) },
                Item { number: 2, content: ItemContent::Plain("second".to_string()) },
            ],
        };
        assert_eq!(
            render_list(&list, 0),
            "The following apply:\na. first\nb. second"
        );
    }

    #[test]
    fn list_no_intro() {
        let list = ListBlock {
            list_type: Some(ListType::Arab),
            intro: String::new(),
            items: vec![Item { number: 1, content: ItemContent::Plain("one".to_string()) }],
        };
        assert_eq!(render_list(&list, 0), "1. one");
    }

    #[test]
    fn list_nested() {
        let inner = ListBlock {
            list_type: Some(ListType::Arab),
            intro: "inner intro".to_string(),
            items: vec![Item { number: 1, content: ItemContent::Plain("nested".to_string()) }],
        };
        let outer = ListBlock {
            list_type: Some(ListType::Alpha),
            intro: String::new(),
            items: vec![Item { number: 1, content: ItemContent::List(inner) }],
        };
        assert_eq!(render_list(&outer, 0), "a. inner intro\n  1. nested");
    }

    #[test]
    fn list_indented_at_depth() {
        let list = ListBlock {
            list_type: Some(ListType::Bullet),
            intro: String::new(),
            items: vec![Item { number: 1, content: ItemContent::Plain("item".to_string()) }],
        };
        assert_eq!(render_list(&list, 2), "    - item");
    }

    // ── render_table ─────────────────────────────────────────────────────────

    fn make_row(texts: &[&str], is_header: bool) -> Row {
        let cells = texts
            .iter()
            .map(|t| Cell { text: t.to_string(), is_header })
            .collect::<Vec<_>>();
        let cell_count = cells.len();
        Row { cells, cell_count, is_header }
    }

    #[test]
    fn table_basic() {
        let table = Table {
            col_count: 2,
            title: None,
            rows: vec![make_row(&["Name", "Value"], true), make_row(&["foo", "bar"], false)],
            row_count: 2,
        };
        assert_eq!(
            render_table(&table),
            "| Name | Value |\n| --- | --- |\n| foo | bar |"
        );
    }

    #[test]
    fn table_with_title() {
        let table = Table {
            col_count: 1,
            title: Some("My table".to_string()),
            rows: vec![make_row(&["Header"], true), make_row(&["Data"], false)],
            row_count: 2,
        };
        assert!(render_table(&table).starts_with("**My table**\n\n"));
    }

    #[test]
    fn table_pipe_in_cell_escaped() {
        let table = Table {
            col_count: 1,
            title: None,
            rows: vec![make_row(&["a|b"], false), make_row(&["c"], false)],
            row_count: 2,
        };
        assert!(render_table(&table).contains("a\\|b"));
    }

    #[test]
    fn table_empty_returns_empty() {
        let table = Table { col_count: 0, title: None, rows: vec![], row_count: 0 };
        assert_eq!(render_table(&table), "");
    }

    // ── render_article ────────────────────────────────────────────────────────

    #[test]
    fn article_heading_no_title() {
        let art = simple_article("Article 1", None, "text");
        assert!(render_article(&art).starts_with("#### Article 1\n\n"));
    }

    #[test]
    fn article_heading_with_title() {
        let art = simple_article("Article 6", Some("Scope"), "text");
        assert!(render_article(&art).starts_with("#### Article 6 — Scope\n\n"));
    }

    #[test]
    fn article_with_paragraphs() {
        let art = Article {
            number: "Article 2".to_string(),
            title: None,
            content: ArticleContent::Paragraphs(vec![
                LegalParagraph {
                    number: "1.".to_string(),
                    alineas: vec![alinea("First paragraph.")],
                    citations: vec![],
                },
                LegalParagraph {
                    number: "2.".to_string(),
                    alineas: vec![alinea("Second paragraph.")],
                    citations: vec![],
                },
            ]),
        };
        let md = render_article(&art);
        assert!(md.contains("1.\n\nFirst paragraph."));
        assert!(md.contains("2.\n\nSecond paragraph."));
    }

    #[test]
    fn article_with_subdivisions() {
        let art = Article {
            number: "Article 3".to_string(),
            title: None,
            content: ArticleContent::Subdivisions(vec![Subdivision {
                title: "Part A".to_string(),
                content: SubdivisionContent::Alineas(vec![alinea("subdivision text")]),
            }]),
        };
        assert!(render_article(&art).contains("**Part A**\n\nsubdivision text"));
    }

    // ── render_chapter / render_section ──────────────────────────────────────

    #[test]
    fn chapter_heading_no_subtitle() {
        let ch = Chapter {
            title: "CHAPTER I".to_string(),
            subtitle: None,
            contents: ChapterContents::Articles(vec![simple_article("Article 1", None, "t")]),
        };
        assert!(render_chapter(&ch).starts_with("## CHAPTER I\n\n"));
    }

    #[test]
    fn chapter_heading_with_subtitle() {
        let ch = Chapter {
            title: "CHAPTER II".to_string(),
            subtitle: Some("General provisions".to_string()),
            contents: ChapterContents::Articles(vec![simple_article("Article 2", None, "t")]),
        };
        assert!(render_chapter(&ch).starts_with("## CHAPTER II — General provisions\n\n"));
    }

    #[test]
    fn section_heading_with_subtitle() {
        let sec = Section {
            title: "SECTION 1".to_string(),
            subtitle: Some("Definitions".to_string()),
            articles: vec![simple_article("Article 1", None, "t")],
        };
        assert!(render_section(&sec).starts_with("### SECTION 1 — Definitions\n\n"));
    }

    // ── render_preamble ───────────────────────────────────────────────────────

    #[test]
    fn preamble_visas_blockquoted() {
        let p = Preamble {
            init: "THE COUNCIL,".to_string(),
            visas: vec![
                "Having regard to the Treaty,".to_string(),
                "Having regard to the proposal,".to_string(),
            ],
            recitals: vec![],
            enacting_formula: "HAS ADOPTED:".to_string(),
        };
        let md = render_preamble(&p);
        assert!(md.contains("> Having regard to the Treaty,\n"));
        assert!(md.contains("> Having regard to the proposal,\n"));
    }

    #[test]
    fn preamble_recitals_bold_numbered() {
        let p = Preamble {
            init: String::new(),
            visas: vec![],
            recitals: vec![
                Recital {
                    number: "(1)".to_string(),
                    text: "First.".to_string(),
                    citations: vec![],
                },
                Recital {
                    number: "(2)".to_string(),
                    text: "Second.".to_string(),
                    citations: vec![],
                },
            ],
            enacting_formula: String::new(),
        };
        let md = render_preamble(&p);
        assert!(md.contains("**(1)** First.\n\n"));
        assert!(md.contains("**(2)** Second.\n\n"));
    }

    // ── render_annexes ────────────────────────────────────────────────────────

    #[test]
    fn annex_heading_no_subtitle() {
        let annex = Annex {
            number: "ANNEX I".to_string(),
            subtitle: None,
            content: AnnexContent::Paragraphs(vec![plain("content")]),
        };
        assert!(render_annex(&annex).starts_with("## ANNEX I\n\n"));
    }

    #[test]
    fn annex_heading_with_subtitle() {
        let annex = Annex {
            number: "ANNEX II".to_string(),
            subtitle: Some("Technical requirements".to_string()),
            content: AnnexContent::Paragraphs(vec![]),
        };
        assert!(render_annex(&annex).starts_with("## ANNEX II — Technical requirements\n\n"));
    }

    #[test]
    fn annex_section_renders_heading() {
        let section = AnnexSection {
            title: "Part 1".to_string(),
            alineas: vec![plain("section text")],
            citations: vec![],
        };
        assert_eq!(render_annex_section(&section), "### Part 1\n\nsection text");
    }

    #[test]
    fn multiple_annexes_separated_by_hr() {
        let annexes = vec![
            Annex {
                number: "ANNEX I".to_string(),
                subtitle: None,
                content: AnnexContent::Paragraphs(vec![]),
            },
            Annex {
                number: "ANNEX II".to_string(),
                subtitle: None,
                content: AnnexContent::Paragraphs(vec![]),
            },
        ];
        assert!(render_annexes(&annexes).contains("\n\n---\n\n"));
    }

    // ── Integration: render_act against real fixtures ─────────────────────────

    #[test]
    fn dsa_renders_title_and_structure() {
        let act = load_act(Path::new("../data/32022R2065")).expect("failed to load DSA fixture");
        let md = render_act(&act);
        assert!(md.starts_with("# Regulation (EU) 2022/2065"));
        assert!(md.contains("**Date:** 2022-10-19"));
        assert!(md.contains("## Preamble"));
        assert!(md.contains("## CHAPTER"));
        assert!(md.contains("#### Article 1"));
    }

    #[test]
    fn dsa_visas_are_blockquoted() {
        let act = load_act(Path::new("../data/32022R2065")).expect("failed to load DSA fixture");
        assert!(render_act(&act).contains("> Having regard to"));
    }

    #[test]
    fn dsa_recitals_are_bold() {
        let act = load_act(Path::new("../data/32022R2065")).expect("failed to load DSA fixture");
        let md = render_act(&act);
        assert!(md.contains("**(1)**"));
        assert!(md.contains("**(156)**"));
    }

    #[test]
    fn trademark_act_renders() {
        let act =
            load_act(Path::new("../data/32017R1001")).expect("failed to load trademark fixture");
        let md = render_act(&act);
        assert!(md.starts_with("# Regulation (EU) 2017/1001"));
        assert!(md.contains("## Preamble"));
        assert!(md.contains("---"));
    }

    #[test]
    /// A consolidated act renders without visa blockquotes or bold recital numbers.
    fn consolidated_act_renders() {
        let act = load_act(Path::new("../data/02016R1036-20180608"))
            .expect("failed to load consolidated anti-dumping fixture");
        let md = render_act(&act);
        assert!(md.contains("2016/1036"));
        assert!(md.contains("## Preamble"));
        assert!(!md.contains("\n> "), "consolidated act must not render visa blockquotes");
        assert!(!md.contains("**(1)**"), "consolidated act must not render recital numbers");
        assert!(md.contains("#### Article 1"));
        assert!(md.contains("## ANNEX"));
    }

    #[test]
    fn eu_ai_act_has_chapters_and_annexes() {
        let act =
            load_act(Path::new("../data/32024R1689")).expect("failed to load EU AI Act fixture");
        let md = render_act(&act);
        assert!(md.contains("## CHAPTER"));
        assert!(md.contains("## ANNEX"));
        assert!(md.contains("---"));
    }
}
