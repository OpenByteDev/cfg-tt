use cfg_tt::cfg_tt;

cfg_tt! {
    pub fn f(a: i32, b: i32) -> i32 {
        let x = a #[cfg(unix)] + #[cfg(windows)] * b;
        x
    }
}

fn main() {
    #[cfg(unix)]
    assert_eq!(f(2, 3), 5);
    #[cfg(windows)]
    assert_eq!(f(2, 3), 6);
}
