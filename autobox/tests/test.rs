#[macro_use]
extern crate autobox;

#[cfg(test)]
mod test_enum {
    #[derive(AutoBox,Debug)]
    /** expression */
    pub enum E {
        /** bool        */ Bool(bool),
        /** nil         */ Nil,
        /** variable    */ Var(String),
        /** addition    */ Add(Box<E>,Box<E>),
        /** subtraction */ Sub{a:Box<E>,b:Box<E>}, 
    }
    #[test]
    fn test() {
        assert_eq!(
            r#"Sub { a: Add(Var("a"), Nil), b: Bool(true) }"#,
            format!("{:?}",e::sub(e::add(e::var("a"),e::nil),e::bool(true))));
    }
    #[test]
    fn test2() {
        use e::*;
        assert_eq!(
            r#"Sub { a: Add(Var("a"), Nil), b: Bool(true) }"#,
            format!("{:?}",sub(add(var("a"),nil),bool(true))));
    }
}

#[cfg(test)]
mod test_struct {
    #[allow(dead_code)]
    #[derive(AutoBox,Debug)]
    pub struct Sample {
        field1: String,
        field2: Box<String>,
    }
    use sample::*;
    #[test]
    fn test() {
        assert_eq!(
            r#"Sample { field1: "field 1", field2: "field 2" }"#,
            format!("{:?}",sample::sample("field 1","field 2")));
        assert_eq!(
            r#"Sample { field1: "field 1", field2: "field 2" }"#,
            format!("{:?}",sample("field 1","field 2")));
    }
}

#[cfg(test)]
mod test_struct2 {
    #[derive(AutoBox,Debug)]
    pub struct Sample2(String,Box<String>);
    use sample2::*;
    #[test]
    fn test() {
        assert_eq!(
            r#"Sample2("a", "b")"#,
            format!("{:?}",sample2::sample2("a","b")));
        assert_eq!(
            r#"Sample2("a", "b")"#,
            format!("{:?}",sample2("a","b")));
    }
}

#[cfg(test)]
mod test_struct3 {
    #[derive(AutoBox,Debug)]
    pub struct Sample3;
    use sample3::*;
    #[test]
    fn test() {
        assert_eq!(
            r#"Sample3"#,
            format!("{:?}",sample3::sample3));
        assert_eq!(
            r#"Sample3"#,
            format!("{:?}",sample3));
    }
}
