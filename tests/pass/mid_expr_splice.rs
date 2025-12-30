use cfg_tt::cfg_tt;

cfg_tt! {
    pub fn f() -> i32 {
        let x = #[cfg(unix)] (10 +) 1 #[cfg(windows)] (+ 20);
        x
    }
}

fn main() {
    let v = f();
    #[cfg(unix)]
    assert_eq!(v, 11);
    #[cfg(windows)]
    assert_eq!(v, 21);
}
