use cfg_tt::cfg_tt;

struct Foo<T>(T);

cfg_tt! {
    type Bar = Foo::#[cfg(unix)](<u8>) #[cfg(windows)](<u16>);
}

fn main() {
    #[cfg(unix)]
    static_assertions::assert_type_eq_all!(Bar, Foo<u8>);
    #[cfg(windows)]
    static_assertions::assert_type_eq_all!(Bar, Foo<u16>);
}
