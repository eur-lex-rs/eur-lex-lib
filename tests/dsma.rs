/// Integration tests against the real Copyright in the Digital Single Market
/// Directive Formex files in `data/32019L0790`.
///
/// These tests validate the structural counts established during development
/// and act as a regression guard against parser changes.
use std::path::Path;

use eur_lex_lib::loader::load_act;
use eur_lex_lib::model::{
    Act, ArticleContent, ChapterContents, EnactingTermsContent, Item, ItemContent, OfficialJournal,
    Subparagraph,
};

#[test]
fn dsma_structure() {
    let act =
        load_act(Path::new("data/32019L0790")).expect("failed to load DSMA from data/32019L0790");
    let Act::Regular(reg) = act else {
        panic!("DSMA should be a Regular act")
    };

    // Title must identify the directive number.
    assert!(
        reg.title.contains("2019/790"),
        "title did not contain '2019/790': {}",
        reg.title
    );

    // Preamble: 6 legal bases, 86 recitals.
    assert_eq!(reg.preamble.visas.len(), 6, "unexpected visa count");
    assert_eq!(reg.preamble.recitals.len(), 86, "unexpected recital count");
    assert_eq!(
        reg.preamble.recitals[0].number, "(1)",
        "first recital number mismatch"
    );

    // Enacting terms: 5 titles, 32 articles total.
    let EnactingTermsContent::Chapters(ref chapters) = reg.enacting_terms.content else {
        panic!("DSMA should have Chapters content");
    };
    assert_eq!(chapters.len(), 5, "unexpected chapter count");

    let total_articles: usize = chapters
        .iter()
        .map(|c| match &c.contents {
            ChapterContents::Articles(arts) => arts.len(),
            ChapterContents::Sections(secs) => secs.iter().map(|s| s.articles.len()).sum(),
        })
        .sum();
    assert_eq!(total_articles, 32, "unexpected total article count");

    // No annexes in the DSMA.
    assert_eq!(reg.annexes.len(), 0, "DSMA should have no annexes");

    // Title I (idx 0): 2 direct articles.
    let title1_arts = match &chapters[0].contents {
        ChapterContents::Articles(arts) => arts,
        _ => panic!("Title I should have direct articles"),
    };
    assert_eq!(title1_arts.len(), 2, "Title I should have 2 articles");

    // Article 1 ("Subject matter and scope"): 2 plain-text paragraphs.
    let art1 = &title1_arts[0];
    assert_eq!(art1.number, "Article 1");
    assert_eq!(art1.title.as_deref(), Some("Subject matter and scope"));
    let ArticleContent::Paragraphs(ref art1_paras) = art1.content else {
        panic!("Article 1 should have Paragraphs content");
    };
    assert_eq!(art1_paras.len(), 2);
    assert_eq!(art1_paras[0].number, "1.");
    assert_eq!(art1_paras[0].alineas.len(), 1);
    assert!(matches!(
        &art1_paras[0].alineas[0].content[0],
        Subparagraph::Plain(_)
    ));

    // Article 2 ("Definitions"): bare <ALINEA> with <P> intro + <LIST> (6 items)
    // → 1 Alinea with a single List block.
    let art2 = &title1_arts[1];
    assert_eq!(art2.number, "Article 2");
    assert_eq!(art2.title.as_deref(), Some("Definitions"));
    let ArticleContent::Alineas(ref art2_alineas) = art2.content else {
        panic!("Article 2 should have Alineas content");
    };
    assert_eq!(art2_alineas.len(), 1, "Article 2 should have 1 bare alinea");
    assert_eq!(
        art2_alineas[0].content.len(),
        1,
        "Article 2 alinea should have a single List block"
    );
    match &art2_alineas[0].content[0] {
        Subparagraph::List(lb) => assert_eq!(lb.items.len(), 6),
        _ => panic!("Article 2 alineas[0].content[0] should be a List"),
    }

    // Title II (idx 1): 5 direct articles.
    let title2_arts = match &chapters[1].contents {
        ChapterContents::Articles(arts) => arts,
        _ => panic!("Title II should have direct articles"),
    };
    assert_eq!(title2_arts.len(), 5, "Title II should have 5 articles");

    // Article 5 (Title II idx 2): para 1 has <P> intro + 2 list items (a)(b).
    let art5 = &title2_arts[2];
    assert_eq!(art5.number, "Article 5");
    let ArticleContent::Paragraphs(ref art5_paras) = art5.content else {
        panic!("Article 5 should have Paragraphs content");
    };
    let p1 = &art5_paras[0];
    assert_eq!(p1.number, "1.");
    assert_eq!(p1.alineas.len(), 1, "Article 5 para 1 should have 1 alinea");
    match &p1.alineas[0].content[0] {
        Subparagraph::List(lb) => {
            assert_eq!(lb.items.len(), 2);
            assert!(matches!(
                &lb.items[0],
                Item {
                    number: 1,
                    content: ItemContent::Text(_)
                }
            ));
            assert!(matches!(
                &lb.items[1],
                Item {
                    number: 2,
                    content: ItemContent::Text(_)
                }
            ));
        }
        _ => panic!("Article 5 para 1 alineas[0].content[0] should be a List"),
    }

    // Title III (idx 2): 4 sections (chapters), 7 articles total.
    let title3_secs = match &chapters[2].contents {
        ChapterContents::Sections(secs) => secs,
        _ => panic!("Title III should have sections"),
    };
    assert_eq!(title3_secs.len(), 4, "Title III should have 4 chapters");
    let title3_total: usize = title3_secs.iter().map(|s| s.articles.len()).sum();
    assert_eq!(title3_total, 7, "Title III should have 7 total articles");
    assert_eq!(
        title3_secs[0].articles.len(),
        4,
        "Title III chapter 1 should have 4 articles"
    );

    // Title IV (idx 3): 3 sections (chapters), 9 articles total.
    let title4_secs = match &chapters[3].contents {
        ChapterContents::Sections(secs) => secs,
        _ => panic!("Title IV should have sections"),
    };
    assert_eq!(title4_secs.len(), 3, "Title IV should have 3 chapters");
    let title4_total: usize = title4_secs.iter().map(|s| s.articles.len()).sum();
    assert_eq!(title4_total, 9, "Title IV should have 9 total articles");
    assert_eq!(
        title4_secs[2].articles.len(),
        6,
        "Title IV chapter 3 should have 6 articles"
    );

    // Title V (idx 4): 9 direct articles.
    let title5_arts = match &chapters[4].contents {
        ChapterContents::Articles(arts) => arts,
        _ => panic!("Title V should have direct articles"),
    };
    assert_eq!(title5_arts.len(), 9, "Title V should have 9 articles");

    // Definitions: Article 2 has 6 items, including one with sub-items.
    assert_eq!(reg.definitions.len(), 6, "DSMA should have 6 definitions");
    assert!(
        reg.definitions.contains_key("press publication"),
        "definitions should contain 'press publication'"
    );
}

#[test]
fn dsma_metadata() {
    let act =
        load_act(Path::new("data/32019L0790")).expect("failed to load DSMA from data/32019L0790");
    let Act::Regular(reg) = act else {
        panic!("DSMA should be a Regular act")
    };
    let md = &reg.metadata;

    assert_eq!(md.prod_id.as_deref(), Some("20190416015"));
    assert_eq!(md.fin_id.as_deref(), Some("250134"));
    assert_eq!(md.authors, vec!["PE", "CS"]);
    assert!(md.eea_relevant, "DSMA should be EEA relevant");

    let oj = md
        .official_journal
        .as_ref()
        .expect("official_journal should be present");
    assert_eq!(
        *oj,
        OfficialJournal {
            collection: "L".to_string(),
            number: "130".to_string(),
            date: "20190517".to_string(),
            language: "EN".to_string(),
        }
    );
}
