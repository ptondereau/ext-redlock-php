# Redlock algorithm implementation for PHP in Rust

This is currently a WIP project


## Requirements

- [`cargo-php`](https://crates.io/crates/cargo-php)
- PHP with `php-dev` installed >= 8.0
- Rust >= 1.61
- CLang 5

## Generating PHP stubs

[`cargo-php`](https://crates.io/crates/cargo-php) have a builtin feature to generate stubs but it's not finalized and stable enough. We use for the moment https://github.com/sasezaki/php-extension-stub-generator to generate with this current usage:

```bash
$ cargo build
$ php \
    -dextension=target/debug/libext_redlock_php.so \
    php-extension-stub-generator.phar dump-files ext-redlock-php stubs
```

## What is redlock?

https://redis.io/docs/reference/patterns/distributed-locks/
