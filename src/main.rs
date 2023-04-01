use autobox::AutoBox;

#[allow(dead_code)]
#[derive(AutoBox,Debug)]
pub struct Sample {
    field1: String,
    field2: Box<String>,
}
use sample::*;

#[derive(AutoBox,Debug)]
pub struct Sample2(String,Box<String>);
use sample2::*;

#[derive(AutoBox,Debug)]
pub struct Sample3;
use sample3::*;

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
fn main() {
    let s = sample("field 1","field 2");
    println!("{:?}", s);
    println!("{:?}",sub(add(var("a"),nil),bool(true)));
    println!("{:?}",sample2("a","b"));
    println!("{:?}",sample3);
}
