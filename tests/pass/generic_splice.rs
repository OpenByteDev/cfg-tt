use cfg_tt::cfg_tt;

struct Foo<T>(T);

cfg_tt! {
    type Bar = Foo::#[cfg(not(windows))](<u8>) #[cfg(windows)](<u16>);
}

macro_rules! assert_type_eq {
    ($first:ty, $($others:ty),+ $(,)*) => {
        const _: fn() = || { $({
            trait TypeEq {
                type This: ?Sized;
            }

            impl<T: ?Sized> TypeEq for T {
                type This = Self;
            }

            fn assert_type_eq_all<T, U>()
            where
                T: ?Sized + TypeEq<This = U>,
                U: ?Sized,
            {}

            assert_type_eq_all::<$first, $others>();
        })+ };
    };
}

fn main() {
    #[cfg(not(windows))]
    assert_type_eq!(Bar, Foo<u8>);
    #[cfg(windows)]
    assert_type_eq!(Bar, Foo<u16>);
}
