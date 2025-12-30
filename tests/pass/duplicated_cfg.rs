use cfg_tt::cfg_tt;

cfg_tt! {
    pub fn duplicated_cfg() -> i32 {
        #[cfg(unix)] {
            let x: i32 =
                #[cfg(unix)] 1
                #[cfg(not(unix))] {
                    #[cfg(not(unix))] {
                        2
                    }
                };
        }
        #[cfg(not(unix))] {
            let x: i32 =
                #[cfg(unix)] 1
                #[cfg(not(unix))] {
                    #[cfg(not(unix))] {
                        2
                    }
                };
        }
        x
    }
}

fn main() {
    let x = duplicated_cfg();

    #[cfg(unix)]
    assert_eq!(x, 1);

    #[cfg(not(unix))]
    assert_eq!(x, 2);
}
