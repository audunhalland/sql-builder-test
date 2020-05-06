use sql_builder_test::build_query;

#[test]
fn its_actually_working_with_proc_macro_hack() {
    let foo = Option::<i32>::None;
    let test = build_query!(
        "SELECT * FROM lol WHERE "
        if let Some(i) = foo {
            "lol.id = " i
        } else {
            "TRUE"
        }
    );
    assert_eq!(test, "SELECT * FROM lol WHERE TRUE");
}
