use cfg_tt::cfg_tt;

cfg_tt! {
    /// some doc comment
    #[allow(clippy::pendantic)]
    pub fn foo() {
        #[cfg(windows)]
        let _ = "noop";
    }
}

fn main() {
    foo();
}
