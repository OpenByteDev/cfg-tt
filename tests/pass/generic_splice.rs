use cfg_tt::cfg_tt;

struct Foo<T>(T);

cfg_tt! {
    type Bar = Foo::#[cfg(not(windows))](<u8>) #[cfg(windows)](<u16>);
}

fn main() {
    #[cfg(not(windows))]
    static_assertions::assert_type_eq_all!(Bar, Foo<u8>);
    #[cfg(windows)]
    static_assertions::assert_type_eq_all!(Bar, Foo<u16>);
}
