use cfg_tt::cfg_tt;

mod m {
    pub fn unix() -> i32 { 1 }
    pub fn windows() -> i32 { 2 }
}

cfg_tt! {
    pub fn f() -> i32 {
        m::#[cfg(unix)] unix #[cfg(windows)] windows ()
    }
}

fn main() {
    #[cfg(unix)]
    assert_eq!(f(), 1);
    #[cfg(windows)]
    assert_eq!(f(), 2);
}
