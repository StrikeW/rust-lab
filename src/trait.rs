use std::fmt;
use std::ops::Add;
use bytes::BufMut;


const MAX_LEN: u32 = 4;
const MY_MASK: u32 = 0xff000000;

trait Graph {
    type N: fmt::Display;
    type E;

    fn has_edge(&self, w: &Self::N, v: &Self::N) -> bool;
    fn edges(&self, node: &Self::N) -> Vec<Self::E>;
    // Etc.
}

fn distance<G: Graph>(graph: &G, start: &G::N, end: &G::N) -> u32 {
    0
}


trait Foo {
    fn method(&self) -> String;
}

impl Foo for u8 {
    fn method(&self) -> String {
        format!("u8: {}", *self)
    }
}

impl Foo for String {
    fn method(&self) -> String {
        format!("string: {}", *self)
    }
}

fn do_sth<T: Foo>(x: T) {
    println!("do_sth{}", x.method());
}

fn do_sth_dynamic(x: &dyn Foo) {
    println!("do_sth_dynamic {}", x.method());
}

fn add<T: Add<Output=T>>(x: T, y: T) -> T {
    x + y
}

fn id<T>(x: T) -> T {
    return x;
}

fn main() {
    let x  = 5u8;
    let y = "Hello".to_string();
    do_sth(x);
    do_sth(y);

    println!("{}", id(10));
    println!("{}", id("some text"));

}