use cfg_tt::cfg_tt;

cfg_tt! {
    pub fn f() -> String {
        format!(#[cfg(not(windows))] "u{}" #[cfg(windows)] "w{}", 7)
    }
}

fn main() {
    let s = f();
    #[cfg(not(windows))]
    assert_eq!(s, "u7");
    #[cfg(windows)]
    assert_eq!(s, "w7");
}
