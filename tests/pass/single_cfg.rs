use cfg_tt::cfg_tt;

pub fn f() -> i32 {
    cfg_tt! {
        return 1 #[cfg(not(windows))] (+ 1);
    }
}

fn main() {
    #[cfg(not(windows))]
    assert_eq!(f(), 2);
    #[cfg(windows)]
    assert_eq!(f(), 1);
}
