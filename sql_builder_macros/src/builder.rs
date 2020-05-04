use std::fmt::Write;

// TODO at some point these types must move out of the proc macro

trait Builder {
    const MAX_LEN: usize;

    type Args;
    type Children;

    fn prepare(self, buf: &mut dyn Write) -> Self::Args;
}

fn sub_builder(
    arg0: i32,
    arg1: Option<Vec<String>>,
) -> impl Builder<Args = (i32, Option<Vec<String>>)> {
    struct B(i32, Option<Vec<String>>);
    impl Builder for B {
        const MAX_LEN: usize = 10;
        type Args = (i32, Option<Vec<String>>);
        type Children = ();

        fn prepare(self, buf: &mut dyn Write) -> Self::Args {
            match &self.1 {
                Some(_) => write!(buf, "id = ?"),
                None => write!(buf, "TRUE"),
            }
            .unwrap();
            (self.0, self.1)
        }
    }
    B(arg0, arg1)
}

fn parent_builder(arg0: &'static str, arg1: Option<Vec<String>>) -> impl Builder {
    struct B(&'static str, Option<Vec<String>>);
    impl Builder for B {
        const MAX_LEN: usize = 10;
        type Args = (&'static str, i32, Option<Vec<String>>);
        type Children = ();

        fn prepare(self, buf: &mut dyn Write) -> Self::Args {
            write!(buf, "SELECT ? FROM ? WHERE (").unwrap();
            let st = sub_builder(0, self.1).prepare(buf);
            write!(buf, ")").unwrap();

            (self.0, st.0, st.1)
        }
    }
    B(arg0, arg1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let b = parent_builder("yo", None);
        let mut sql = String::new();
        let _args = b.prepare(&mut sql);

        assert_eq!(sql, "SELECT ? FROM ? WHERE (TRUE)");
    }
}
