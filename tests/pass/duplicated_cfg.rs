use cfg_tt::cfg_tt;

cfg_tt! {
    pub fn duplicated_cfg() -> i32 {
        #[cfg(not(windows))] {
            let x: i32 =
                #[cfg(not(windows))] 1
                #[cfg(windows)] {
                    #[cfg(windows)] {
                        2
                    }
                };
        }
        #[cfg(windows)] {
            let x: i32 =
                #[cfg(not(windows))] 1
                #[cfg(windows)] {
                    #[cfg(windows)] {
                        2
                    }
                };
        }
        x
    }
}

fn main() {
    let x = duplicated_cfg();

    #[cfg(not(windows))]
    assert_eq!(x, 1);

    #[cfg(windows)]
    assert_eq!(x, 2);
}
