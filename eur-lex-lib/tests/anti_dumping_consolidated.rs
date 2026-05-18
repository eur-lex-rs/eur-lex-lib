/// Integration tests against the consolidated Anti-Dumping Regulation Formex
/// files in `data/02016R1036-20180608` (CELEX 02016R1036-20180608).
///
/// Validates consolidated-act parsing (CONS.ACT root, no recitals, inline
/// CONS.ANNEX elements) for a flat act (no DIVISION wrappers).
///
/// Compared to the original regulation (32016R1036, 25 articles), this
/// consolidated version adds Articles 14a, 19a and 23a (28 articles total)
/// and the SUBDIV titles are prefixed with a letter (A., B., C., D.).
use std::path::Path;

use eur_lex_lib::loader::load_act;
use eur_lex_lib::model::{Act, ArticleContent, EnactingTermsContent, SubdivisionContent};

#[test]
/// Validates the structural layout of the Anti-Dumping Regulation (consolidated, 2018):
/// flat `EnactingTermsContent::Articles` (28 articles — 3 more than the original),
/// 3 inline annexes (ANNEX I, Ia, II), and Article 2 subdivisions with letter-prefixed titles.
fn anti_dumping_consolidated_structure() {
    let loaded = load_act(Path::new("../data/02016R1036-20180608"))
        .expect("failed to load consolidated Anti-Dumping Regulation");
    let Act::Consolidated(act) = loaded else {
        panic!("02016R1036-20180608 should be a Consolidated act")
    };

    assert!(
        act.title.contains("2016/1036"),
        "title did not contain '2016/1036': {}",
        act.title
    );

    // Flat structure: 28 articles directly in enacting terms, no DIVISION wrappers.
    let EnactingTermsContent::Articles(ref articles) = act.enacting_terms.content else {
        panic!("consolidated anti-dumping regulation should have flat Articles content")
    };
    assert_eq!(articles.len(), 28, "unexpected total article count");

    // 3 inline CONS.ANNEX elements: ANNEX I, ANNEX Ia, ANNEX II.
    assert_eq!(act.annexes.len(), 3, "unexpected annex count");
    assert!(
        act.annexes[0].number.contains("ANNEX I"),
        "expected ANNEX I at index 0, got: {}",
        act.annexes[0].number
    );
    assert!(
        act.annexes[1].number.contains("ANNEX Ia"),
        "expected ANNEX Ia at index 1, got: {}",
        act.annexes[1].number
    );
    assert!(
        act.annexes[2].number.contains("ANNEX II"),
        "expected ANNEX II at index 2, got: {}",
        act.annexes[2].number
    );

    // Article 1 (index 0): Paragraphs (4 numbered paragraphs).
    let art1 = &articles[0];
    assert_eq!(art1.number, "Article 1");
    let ArticleContent::Paragraphs(ref art1_paras) = art1.content else {
        panic!("Article 1 should have Paragraphs content")
    };
    assert_eq!(art1_paras.len(), 4, "Article 1 should have 4 paragraphs");

    // Article 2 (index 1): Subdivisions — four thematic groups.
    let art2 = &articles[1];
    assert_eq!(art2.number, "Article 2");
    assert_eq!(art2.title.as_deref(), Some("Determination of dumping"));
    let ArticleContent::Subdivisions(ref subdivs) = art2.content else {
        panic!("Article 2 should have Subdivisions content")
    };
    assert_eq!(subdivs.len(), 4, "Article 2 should have 4 subdivisions");

    // Each subdivision holds PARAG-wrapped paragraphs (same as the original act).
    // The consolidated version added 1 paragraph to subdivision A (8 vs 7 in the original).
    let expected = [
        ("A.NORMAL VALUE", 8usize),
        ("B.EXPORT PRICE", 2),
        ("C.COMPARISON", 1),
        ("D.DUMPING MARGIN", 2),
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

    // Articles 22, 24 and 25 (indices 23, 26, 27) use bare Alineas (no PARAG wrapper).
    for &(idx, name) in &[(23usize, "Article 22"), (26, "Article 24"), (27, "Article 25")] {
        assert_eq!(articles[idx].number, name);
        assert!(
            matches!(&articles[idx].content, ArticleContent::Alineas(_)),
            "{} (index {}) should have Alineas content",
            name,
            idx
        );
    }
}
