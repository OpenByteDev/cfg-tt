use cfg_tt::cfg_tt;

trait A { fn a() -> i32 { 1 } }
trait B { fn b() -> i32 { 2 } }

cfg_tt! {
    trait C : #[cfg(not(windows))] A #[cfg(windows)] B {}
}

struct S;
#[cfg(not(windows))]
impl A for S {}
#[cfg(windows)]
impl B for S {}
impl C for S {}

fn main() {
    #[cfg(not(windows))]
    assert_eq!(S::a(), 1);
    #[cfg(windows)]
    assert_eq!(S::b(), 2);
}
