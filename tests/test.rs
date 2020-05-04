use sql_builder_test::build_query;

#[test]
fn something_is_working_at_least() {
    // let foo = Some(42_i32);
    // BUG: referring to outer variables doesn't work yet

    let a = build_query!(
        "SELECT * FROM lol WHERE "
        if true {
            "lol.id = " i
        }
    );
    assert_eq!(a, "SELECT * FROM lol WHERE lol.id = $0");
}
