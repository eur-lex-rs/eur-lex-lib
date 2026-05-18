use crate::model::{
    Act, Alinea, Annex, AnnexContent, AnnexSection, Article, ArticleContent, Chapter,
    ChapterContents, ConsolidatedAct, ConsolidatedPreamble, EnactingTerms, EnactingTermsContent,
    Item, ItemContent, LegalParagraph, ListBlock, ListType, Metadata, PhysicalNumberedParagraph,
    Preamble, RegularAct, Section, Subdivision, SubdivisionContent, Subparagraph, Table,
};

const SEPARATOR: &str = "────────────────────────────────────────────────────────────────────────";

// ── Top-level rendering ───────────────────────────────────────────────────────

pub fn render_act(act: &Act) -> String {
    match act {
        Act::Regular(a) => render_regular(a),
        Act::Consolidated(a) => render_consolidated(a),
    }
}

fn render_regular(act: &RegularAct) -> String {
    let mut out = format!("{}\n\n", act.title);
    let meta = render_metadata_line(&act.metadata);
    if !meta.is_empty() {
        out.push_str(&meta);
        out.push_str("\n\n");
    }
    out.push_str(SEPARATOR);
    out.push_str("\n\nPREAMBLE\n\n");
    out.push_str(&render_preamble(&act.preamble));
    out.push('\n');
    out.push_str(SEPARATOR);
    out.push_str("\n\n");
    out.push_str(&render_enacting_terms(&act.enacting_terms));
    if !act.annexes.is_empty() {
        out.push('\n');
        out.push_str(SEPARATOR);
        out.push_str("\n\n");
        out.push_str(&render_annexes(&act.annexes));
    }
    out
}

fn render_consolidated(act: &ConsolidatedAct) -> String {
    let mut out = format!("{}\n\n", act.title);
    let meta = render_metadata_line(&act.metadata);
    if !meta.is_empty() {
        out.push_str(&meta);
        out.push_str("\n\n");
    }
    out.push_str(SEPARATOR);
    out.push_str("\n\nPREAMBLE\n\n");
    out.push_str(&render_consolidated_preamble(&act.preamble));
    out.push('\n');
    out.push_str(SEPARATOR);
    out.push_str("\n\n");
    out.push_str(&render_enacting_terms(&act.enacting_terms));
    if !act.annexes.is_empty() {
        out.push('\n');
        out.push_str(SEPARATOR);
        out.push_str("\n\n");
        out.push_str(&render_annexes(&act.annexes));
    }
    out
}

// ── Metadata ─────────────────────────────────────────────────────────────────

fn render_metadata_line(meta: &Metadata) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(ref v) = meta.celex {
        parts.push(format!("CELEX: {v}"));
    }
    if let Some(ref v) = meta.document_date {
        parts.push(format!("Date: {}", format_date(v)));
    }
    if let Some(ref v) = meta.language {
        parts.push(format!("Language: {v}"));
    }
    if let Some(ref v) = meta.legal_value {
        parts.push(format!("Type: {v}"));
    }
    parts.join("  ")
}

fn format_date(raw: &str) -> String {
    if raw.len() == 8 {
        format!("{}-{}-{}", &raw[..4], &raw[4..6], &raw[6..8])
    } else {
        raw.to_string()
    }
}

// ── Preamble ──────────────────────────────────────────────────────────────────

fn render_preamble(p: &Preamble) -> String {
    let mut out = String::new();
    if !p.init.is_empty() {
        out.push_str(&p.init);
        out.push_str("\n\n");
    }
    for visa in &p.visas {
        out.push_str(visa);
        out.push('\n');
    }
    if !p.visas.is_empty() {
        out.push('\n');
    }
    for recital in &p.recitals {
        out.push_str(&format!("{} {}\n\n", recital.number, recital.text));
    }
    if !p.enacting_formula.is_empty() {
        out.push_str(&p.enacting_formula);
    }
    out
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

fn render_enacting_terms(et: &EnactingTerms) -> String {
    match &et.content {
        EnactingTermsContent::Chapters(chapters) => {
            chapters.iter().map(render_chapter).collect::<Vec<_>>().join("\n\n")
        }
        EnactingTermsContent::Articles(articles) => {
            articles.iter().map(render_article).collect::<Vec<_>>().join("\n\n")
        }
    }
}

fn render_chapter(chapter: &Chapter) -> String {
    let heading = match &chapter.subtitle {
        Some(sub) => format!("{} — {sub}", chapter.title),
        None => chapter.title.clone(),
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

fn render_section(section: &Section) -> String {
    let heading = match &section.subtitle {
        Some(sub) => format!("  {} — {sub}", section.title),
        None => format!("  {}", section.title),
    };
    let body = section
        .articles
        .iter()
        .map(render_article)
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("{heading}\n\n{body}")
}

fn render_article(article: &Article) -> String {
    let heading = match &article.title {
        Some(t) => format!("  {} — {t}", article.number),
        None => format!("  {}", article.number),
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
    format!("  {}\n\n{body}", p.number)
}

fn render_alinea(a: &Alinea) -> String {
    a.content
        .iter()
        .map(|sp| render_subparagraph(sp, 2))
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
    format!("  {}\n\n{body}", subdiv.title)
}

// ── Block content ─────────────────────────────────────────────────────────────

fn render_subparagraph(sp: &Subparagraph, indent: usize) -> String {
    let pad = " ".repeat(indent);
    match sp {
        Subparagraph::Plain(text) => format!("{pad}{text}"),
        Subparagraph::List(list) => render_list(list, indent),
        Subparagraph::Table(table) => render_table(table, indent),
        Subparagraph::Numbered(np) => render_numbered_paragraph(np, indent),
    }
}

fn render_numbered_paragraph(np: &PhysicalNumberedParagraph, indent: usize) -> String {
    let pad = " ".repeat(indent);
    let body = np
        .alineas
        .iter()
        .map(|sp| render_subparagraph(sp, indent))
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("{pad}{}\n\n{body}", np.number)
}

// ── Lists ─────────────────────────────────────────────────────────────────────

fn render_list(list: &ListBlock, indent: usize) -> String {
    let pad = " ".repeat(indent);
    let mut out = String::new();
    if !list.intro.is_empty() {
        out.push_str(&format!("{pad}{}\n", list.intro));
    }
    for item in &list.items {
        out.push_str(&render_item(item, &list.list_type, indent + 2));
    }
    out.trim_end().to_string()
}

fn render_item(item: &Item, list_type: &Option<ListType>, indent: usize) -> String {
    let pad = " ".repeat(indent);
    let prefix = item_prefix(list_type, item.number);
    match &item.content {
        ItemContent::Plain(text) => format!("{pad}{prefix}{text}\n"),
        ItemContent::List(nested) => {
            let mut out = String::new();
            if !nested.intro.is_empty() {
                out.push_str(&format!("{pad}{prefix}{}\n", nested.intro));
            }
            for nested_item in &nested.items {
                out.push_str(&render_item(nested_item, &nested.list_type, indent + 2));
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

fn render_table(table: &Table, indent: usize) -> String {
    if table.rows.is_empty() {
        return String::new();
    }
    let pad = " ".repeat(indent);
    let mut out = String::new();
    if let Some(ref title) = table.title {
        out.push_str(&format!("{pad}{title}\n\n"));
    }

    // Compute column widths across all rows.
    let col_count = table.rows.iter().map(|r| r.cells.len()).max().unwrap_or(0);
    let mut widths = vec![0usize; col_count];
    for row in &table.rows {
        for (i, cell) in row.cells.iter().enumerate() {
            widths[i] = widths[i].max(cell.text.len());
        }
    }

    let mut rows = table.rows.iter();
    if let Some(header) = rows.next() {
        let line = header
            .cells
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{:<width$}", c.text, width = widths[i]))
            .collect::<Vec<_>>()
            .join("  ");
        out.push_str(&format!("{pad}{line}\n"));
        let underline = widths.iter().map(|&w| "-".repeat(w)).collect::<Vec<_>>().join("  ");
        out.push_str(&format!("{pad}{underline}\n"));
    }
    for row in rows {
        let line = row
            .cells
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{:<width$}", c.text, width = widths[i]))
            .collect::<Vec<_>>()
            .join("  ");
        out.push_str(&format!("{pad}{line}\n"));
    }
    out.trim_end().to_string()
}

// ── Annexes ───────────────────────────────────────────────────────────────────

fn render_annexes(annexes: &[Annex]) -> String {
    annexes
        .iter()
        .map(render_annex)
        .collect::<Vec<_>>()
        .join(&format!("\n\n{SEPARATOR}\n\n"))
}

fn render_annex(annex: &Annex) -> String {
    let heading = match &annex.subtitle {
        Some(sub) => format!("{} — {sub}", annex.number),
        None => annex.number.clone(),
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

fn render_annex_section(section: &AnnexSection) -> String {
    let body = section
        .alineas
        .iter()
        .map(|sp| render_subparagraph(sp, 2))
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("  {}\n\n{body}", section.title)
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
            "CELEX: 32022R2065  Date: 2022-10-19  Language: EN  Type: REG"
        );
    }

    #[test]
    fn metadata_no_markup() {
        let meta = Metadata {
            celex: Some("32024R1689".to_string()),
            language: Some("EN".to_string()),
            ..empty_metadata()
        };
        let line = render_metadata_line(&meta);
        assert!(!line.contains("**"));
        assert_eq!(line, "CELEX: 32024R1689  Language: EN");
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
    fn item_prefix_roman_upper() {
        assert_eq!(item_prefix(&Some(ListType::RomanUpper), 4), "IV. ");
    }

    #[test]
    fn item_prefix_bullet() {
        assert_eq!(item_prefix(&Some(ListType::Bullet), 1), "- ");
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
        // indent=0 → intro has no pad, items have 2-space indent
        let out = render_list(&list, 0);
        assert!(out.contains("The following apply:"));
        assert!(out.contains("  a. first"));
        assert!(out.contains("  b. second"));
    }

    #[test]
    fn list_no_markup_chars() {
        let list = ListBlock {
            list_type: Some(ListType::Bullet),
            intro: String::new(),
            items: vec![Item { number: 1, content: ItemContent::Plain("item".to_string()) }],
        };
        let out = render_list(&list, 0);
        assert!(!out.contains('*'));
        assert!(!out.contains('#'));
        assert!(!out.contains('|'));
    }

    // ── render_table ─────────────────────────────────────────────────────────

    fn make_row(texts: &[&str], is_header: bool) -> Row {
        let cells =
            texts.iter().map(|t| Cell { text: t.to_string(), is_header }).collect::<Vec<_>>();
        let cell_count = cells.len();
        Row { cells, cell_count, is_header }
    }

    #[test]
    fn table_header_underlined() {
        let table = Table {
            col_count: 2,
            title: None,
            rows: vec![make_row(&["Name", "Value"], true), make_row(&["foo", "bar"], false)],
            row_count: 2,
        };
        let out = render_table(&table, 0);
        // Header row followed by dashes
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], "Name  Value");
        assert!(lines[1].contains("----"));
        assert_eq!(lines[2], "foo   bar");
    }

    #[test]
    fn table_no_pipe_chars() {
        let table = Table {
            col_count: 2,
            title: None,
            rows: vec![make_row(&["A", "B"], true), make_row(&["C", "D"], false)],
            row_count: 2,
        };
        assert!(!render_table(&table, 0).contains('|'));
    }

    #[test]
    fn table_empty_returns_empty() {
        let table = Table { col_count: 0, title: None, rows: vec![], row_count: 0 };
        assert_eq!(render_table(&table, 0), "");
    }

    // ── render_article ────────────────────────────────────────────────────────

    #[test]
    fn article_heading_no_title() {
        let art = simple_article("Article 1", None, "text");
        assert!(render_article(&art).starts_with("  Article 1\n\n"));
    }

    #[test]
    fn article_heading_with_title() {
        let art = simple_article("Article 6", Some("Scope"), "text");
        assert!(render_article(&art).starts_with("  Article 6 — Scope\n\n"));
    }

    #[test]
    fn article_no_markup() {
        let art = simple_article("Article 1", Some("Scope"), "body text");
        let out = render_article(&art);
        assert!(!out.contains("####"));
        assert!(!out.contains("**"));
    }

    // ── render_chapter / render_section ──────────────────────────────────────

    #[test]
    fn chapter_heading_no_subtitle() {
        let ch = Chapter {
            title: "CHAPTER I".to_string(),
            subtitle: None,
            contents: ChapterContents::Articles(vec![simple_article("Article 1", None, "t")]),
        };
        assert!(render_chapter(&ch).starts_with("CHAPTER I\n\n"));
    }

    #[test]
    fn chapter_heading_with_subtitle() {
        let ch = Chapter {
            title: "CHAPTER II".to_string(),
            subtitle: Some("General provisions".to_string()),
            contents: ChapterContents::Articles(vec![simple_article("Article 2", None, "t")]),
        };
        assert!(render_chapter(&ch).starts_with("CHAPTER II — General provisions\n\n"));
    }

    #[test]
    fn section_indented() {
        let sec = Section {
            title: "SECTION 1".to_string(),
            subtitle: None,
            articles: vec![simple_article("Article 1", None, "t")],
        };
        assert!(render_section(&sec).starts_with("  SECTION 1\n\n"));
    }

    // ── render_preamble ───────────────────────────────────────────────────────

    #[test]
    fn preamble_visas_no_blockquote() {
        let p = Preamble {
            init: String::new(),
            visas: vec!["Having regard to the Treaty,".to_string()],
            recitals: vec![],
            enacting_formula: String::new(),
        };
        let out = render_preamble(&p);
        assert!(!out.contains('>'));
        assert!(out.contains("Having regard to the Treaty,"));
    }

    #[test]
    fn preamble_recitals_plain() {
        let p = Preamble {
            init: String::new(),
            visas: vec![],
            recitals: vec![
                Recital {
                    number: "(1)".to_string(),
                    text: "First.".to_string(),
                    citations: vec![],
                },
            ],
            enacting_formula: String::new(),
        };
        let out = render_preamble(&p);
        assert!(out.contains("(1) First."));
        assert!(!out.contains("**(1)**"));
    }

    // ── render_annexes ────────────────────────────────────────────────────────

    #[test]
    fn annex_heading_no_subtitle() {
        let annex = Annex {
            number: "ANNEX I".to_string(),
            subtitle: None,
            content: AnnexContent::Paragraphs(vec![plain("content")]),
        };
        assert!(render_annex(&annex).starts_with("ANNEX I\n\n"));
    }

    #[test]
    fn annex_section_indented() {
        let section = AnnexSection {
            title: "Part 1".to_string(),
            alineas: vec![plain("section text")],
            citations: vec![],
        };
        assert!(render_annex_section(&section).starts_with("  Part 1\n\n"));
    }

    #[test]
    fn multiple_annexes_have_separator() {
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
        assert!(render_annexes(&annexes).contains(SEPARATOR));
    }

    // ── Integration ───────────────────────────────────────────────────────────

    #[test]
    fn dsa_renders_title_and_structure() {
        let act = load_act(Path::new("../data/32022R2065")).expect("failed to load DSA fixture");
        let out = render_act(&act);
        assert!(out.starts_with("Regulation (EU) 2022/2065"));
        assert!(out.contains("Date: 2022-10-19"));
        assert!(out.contains("PREAMBLE"));
        assert!(out.contains("CHAPTER"));
        assert!(out.contains("Article 1"));
    }

    #[test]
    fn dsa_no_markdown_syntax() {
        let act = load_act(Path::new("../data/32022R2065")).expect("failed to load DSA fixture");
        let out = render_act(&act);
        assert!(!out.contains("**"));
        assert!(!out.contains("##"));
        assert!(!out.contains("> "));
    }

    #[test]
    fn dsa_visas_not_blockquoted() {
        let act = load_act(Path::new("../data/32022R2065")).expect("failed to load DSA fixture");
        let out = render_act(&act);
        assert!(out.contains("Having regard to"));
        assert!(!out.contains("> Having regard to"));
    }

    #[test]
    fn trademark_act_consolidated_renders() {
        let act =
            load_act(Path::new("../data/32017R1001")).expect("failed to load trademark fixture");
        let out = render_act(&act);
        assert!(out.starts_with("Regulation (EU) 2017/1001"));
        assert!(out.contains("PREAMBLE"));
        assert!(out.contains(SEPARATOR));
    }

    #[test]
    fn eu_ai_act_has_chapters_and_annexes() {
        let act =
            load_act(Path::new("../data/32024R1689")).expect("failed to load EU AI Act fixture");
        let out = render_act(&act);
        assert!(out.contains("CHAPTER"));
        assert!(out.contains("ANNEX"));
        assert!(out.contains(SEPARATOR));
    }
}
