use nu_test_support::{nu, pipeline};

#[test]

fn can_query_single_table() {
    let actual = nu!(
        cwd: "tests/fixtures/formats", pipeline(
        r#"
            open sample.db
            | query db "select * from strings"
            | where x =~ ell
            | length
        "#
    ));

    assert_eq!(actual.out, "4");
}
