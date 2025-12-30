use cfg_tt::cfg_tt;

trait A { fn a(&self) -> i32; }
trait B { fn b(&self) -> i32; }

struct S;

impl A for S { fn a(&self) -> i32 { 1 } }
impl B for S { fn b(&self) -> i32 { 2 } }

cfg_tt! {
    pub fn f<T>(t: &T) -> i32
    where
        T: #[cfg(unix)] A #[cfg(windows)] B
    {
        #[cfg(unix)] { t.a() }
        #[cfg(windows)] { t.b() }
    }
}

fn main() {
    let v = f(&S);
    assert!(v == 1 || v == 2);
}
