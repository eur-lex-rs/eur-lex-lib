/// Integration tests against the Anti-Dumping Regulation Formex files in
/// `data/32016R1036` (original act, CELEX 32016R1036).
///
/// Validates that a regular act whose `<ENACTING.TERMS>` contains articles
/// directly (no `<DIVISION>` wrappers) is parsed as `EnactingTermsContent::Articles`,
/// and that Article 2 — which groups its paragraphs into four `<SUBDIV>` thematic
/// sections — is parsed as `ArticleContent::Subdivisions`.
use std::path::Path;

use eur_lex_lib::loader::load_act;
use eur_lex_lib::model::{Act, ArticleContent, EnactingTermsContent, SubdivisionContent};

#[test]
/// Validates the structural layout of the Anti-Dumping Regulation (original): flat
/// `EnactingTermsContent::Articles` (25 articles, no chapter wrappers), 32 recitals,
/// 2 annexes, and Article 2 parsed as `ArticleContent::Subdivisions` (4 thematic groups).
fn anti_dumping_regulation_structure() {
    let loaded = load_act(Path::new("../data/32016R1036"))
        .expect("failed to load Anti-Dumping Regulation from data/32016R1036");
    let Act::Regular(reg) = loaded else {
        panic!("32016R1036 should be a Regular act")
    };

    assert!(
        reg.title.contains("2016/1036"),
        "title did not contain '2016/1036': {}",
        reg.title
    );

    // 32 recitals in the preamble.
    assert_eq!(reg.preamble.recitals.len(), 32, "unexpected recital count");

    // Flat structure: articles directly in enacting terms, no DIVISION wrappers.
    let EnactingTermsContent::Articles(ref articles) = reg.enacting_terms.content else {
        panic!("32016R1036 should have flat Articles content, not Chapters")
    };
    assert_eq!(articles.len(), 25, "unexpected total article count");

    // 2 annexes (ANNEX I and ANNEX II).
    assert_eq!(reg.annexes.len(), 2, "unexpected annex count");
    assert!(
        reg.annexes[0].number.contains("ANNEX I"),
        "expected ANNEX I at index 0, got: {}",
        reg.annexes[0].number
    );
    assert!(
        reg.annexes[1].number.contains("ANNEX II"),
        "expected ANNEX II at index 1, got: {}",
        reg.annexes[1].number
    );

    // Article 2 ("Determination of dumping") uses Subdivisions.
    let art2 = &articles[1];
    assert_eq!(art2.number, "Article 2");
    assert_eq!(art2.title.as_deref(), Some("Determination of dumping"));
    let ArticleContent::Subdivisions(ref subdivs) = art2.content else {
        panic!("Article 2 should have Subdivisions content")
    };
    assert_eq!(subdivs.len(), 4, "Article 2 should have 4 subdivisions");

    // Each subdivision has the expected title and paragraph count.
    let expected = [
        ("NORMAL VALUE", 7usize),
        ("EXPORT PRICE", 2),
        ("COMPARISON", 1),
        ("DUMPING MARGIN", 2),
    ];
    for (i, (title, para_count)) in expected.iter().enumerate() {
        assert!(
            subdivs[i].title.contains(title),
            "SUBDIV {} title '{}' did not contain '{}'",
            i,
            subdivs[i].title,
            title
        );
        let SubdivisionContent::Paragraphs(ref paras) = subdivs[i].content else {
            panic!("SUBDIV {} should have Paragraphs content", i)
        };
        assert_eq!(paras.len(), *para_count, "SUBDIV {} paragraph count", i);
    }

    // Articles 22, 24 and 25 use bare Alineas (no PARAG wrapper).
    for &idx in &[21usize, 23, 24] {
        assert!(
            matches!(&articles[idx].content, ArticleContent::Alineas(_)),
            "Article at index {} should have Alineas content",
            idx
        );
    }
}
