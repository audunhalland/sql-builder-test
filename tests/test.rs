use sql_builder_test::build_query;

#[test]
fn its_actually_working_with_proc_macro_hack() {
    let foo = Some(42_i32);
    let query = build_query!(
        "SELECT * FROM lol WHERE "
        if let Some(i) = foo {
            "lol.id = " i
        } else {
            "TRUE"
        }
    );
    assert_eq!(query.sql, "SELECT * FROM lol WHERE lol.id = $0");
    assert_eq!(query.args_count, 1);
    assert_eq!(query.args_size, 4);
}
