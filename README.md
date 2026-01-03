# cfg-tt

[![CI](https://github.com/OpenByteDev/cfg-tt/actions/workflows/ci.yml/badge.svg)](https://github.com/OpenByteDev/cfg-tt/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/cfg-tt.svg)](https://crates.io/crates/cfg-tt)
[![Documentation](https://docs.rs/cfg-tt/badge.svg)](https://docs.rs/cfg-tt)
[![dependency status](https://deps.rs/repo/github/openbytedev/cfg-tt/status.svg)](https://deps.rs/repo/github/openbytedev/cfg-tt)
[![MIT](https://img.shields.io/crates/l/cfg-tt.svg)](https://github.com/OpenByteDev/cfg-tt/blob/master/LICENSE)

`cfg_tt!` is a procedural macro that allows using `#[cfg(...)]` **anywhere** and at **token granularity**.

## Why

Standard `#[cfg]` attributes are constrained by Rust’s grammar and are applied only after parsing.
As a result, conditional compilation is limited to syntactically valid positions.

`cfg_tt!` operates directly on raw tokens, allowing conditional inclusion or exclusion in places where Rust normally disallows `#[cfg]`, such as within expressions, generics, where clauses, etc.

## How it works
Given the following code:

```rust
cfg_tt::cfg_tt! {
    pub fn f() -> i32 {
        1 #[cfg(windows)] + #[cfg(not(windows))] * 1
    }
}
```

It (currently) expands to:
```rust
#[cfg(windows)]
pub fn f() -> i32 {
    1 + 1
}

#[cfg(not(windows))]
pub fn f() -> i32 {
    1 * 1
}
```

## Usage
Within the `cfg_tt!` macro, `#[cfg(...)]` may appear anywhere.

Each `#[cfg(...)]` applies to exactly the next token tree, which may be:

- a group (`{ ... }`, `( ... )`, `[ ... ]`)
- an identifier (e.g. `foo`)
- a literal (e.g. `42`, `"x"`)
- a punctuation token (e.g. `+`, `::`)

To conditionally include multiple token trees, they must be wrapped in a group:
```rust
cfg_tt::cfg_tt! {
    let x =
        #[cfg(not(windows))] { 10 + 20 }
        #[cfg(windows)] { 1 + 2 };
}
```

## Limitations

The following usages are not (yet) supported:

- Stacked `#[cfg]` attributes
  (e.g. `#[cfg(x)] #[cfg(x)]`)

- Nested overlapping `#[cfg]`s  
  (e.g. `#[cfg(any(x, y))] { #[cfg(x)] { pub struct Foo; } }`)

## License
This project is licensed under the MIT License. See the [LICENSE](https://github.com/OpenByteDev/cfg-tt/blob/master/LICENSE) file for details.
