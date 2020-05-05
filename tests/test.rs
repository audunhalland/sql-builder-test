use sql_builder_test::build_query;
use sql_builder_test::build_query2;

#[test]
fn something_is_working_at_least_but_the_implementation_is_bugged() {
    // let foo = Some(42_i32);
    // BUG: referring to outer variables doesn't work here

    let test = build_query!(
        "SELECT * FROM lol WHERE "
        if true {
            "lol.id = " i
        }
    );
    assert_eq!(test, "SELECT * FROM lol WHERE lol.id = $0");
}

#[test]
fn its_actually_working_with_proc_macro_hack() {
    let foo = Option::<i32>::None;
    let test = build_query2!(
        "SELECT * FROM lol WHERE "
        if let Some(i) = foo {
            "lol.id = " i
        } else {
            "TRUE"
        }
    );
    assert_eq!(test, "SELECT * FROM lol WHERE TRUE");
}
