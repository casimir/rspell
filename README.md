# rspell

[![crates.io](https://meritbadge.herokuapp.com/rspell)](https://crates.io/crates/rspell) [![rspell docs](https://docs.rs/rspell/badge.svg)](https://docs.rs/rspell)

A simple practical spellcheker.

## Dependencies caveats

This crates wraps hunspell's source directly. To do so it uses the [`cc-rs`](https://crates.io/crates/cc)
crate when building. As such the same limitations applies, for example a compiler must be
installed on the system.

## Example

```rust
let spell = rspell::Spell::new("en_US").unwrap();

assert!(!spell.check_word("colour").correct());
assert!(spell.check_word("color").correct());

for bad in spell.check("Wht color is this flg?") {
    println!(
        "{} (offset: {}): possible corrections: {:?}",
        bad.word, bad.offset, bad.suggestions
    );
}
```

## Loose goals

- multi-lang support
- remove the need for the `cc-rs` crate

## License

This project is licensed under either of
- Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (http://opensource.org/licenses/MIT)


Hunspell's licensing applies to hunspell's source files.