# AutoBox Derive Macro
AutoBox Derive Macro for Rust proc-macro demo

usage:

```rust
#[cfg(test)]
mod test{
    use autobox::AutoBox;
    #[derive(AutoBox,Debug)]
    /** expression */
    pub enum E {
        /** bool        */ Bool(bool),
        /** nil         */ Nil,
        /** variable    */ Var(String),
        /** addition    */ Add(Box<E>,Box<E>),
        /** subtraction */ Sub{a:Box<E>,b:Box<E>}, 
    }
    use e::*;
    #[test]
    fn test() {
        assert_eq!(r#"Sub { a: Add(Var("a"), Nil), b: Bool(true) }"#,format!("{:?}",sub(add(var("a"),nil),bool(true))));
    }
}

fn main() {}
```
