use cfg_tt::cfg_tt;

mod m {
    pub fn unix() -> i32 { 1 }
    pub fn windows() -> i32 { 2 }
}

cfg_tt! {
    pub fn f() -> i32 {
        m::#[cfg(not(windows))] unix #[cfg(windows)] windows ()
    }
}

fn main() {
    #[cfg(not(windows))]
    assert_eq!(f(), 1);
    #[cfg(windows)]
    assert_eq!(f(), 2);
}
