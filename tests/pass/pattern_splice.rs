use cfg_tt::cfg_tt;

cfg_tt! {
    pub fn f(x: i32) -> i32 {
        match x {
            #[cfg(not(windows))] 0
            #[cfg(windows)] 1 => 20,
            _ => 1,
        }
    }
}

fn main() {
    #[cfg(not(windows))] {
        assert_eq!(f(0), 20);
        assert_eq!(f(1), 1);
    }
    #[cfg(windows)] {
        assert_eq!(f(0), 1);
        assert_eq!(f(1), 20);
    }
    assert_eq!(f(2), 1);
}
