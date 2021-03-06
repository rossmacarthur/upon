<!-- generated by tools/gen-readme -->

# upon

[![Crates.io Version](https://img.shields.io/crates/v/upon.svg)](https://crates.io/crates/upon)
[![Docs.rs Latest](https://img.shields.io/badge/docs.rs-latest-blue.svg)](https://docs.rs/upon)
[![Build Status](https://img.shields.io/github/workflow/status/rossmacarthur/upon/build/trunk)](https://github.com/rossmacarthur/upon/actions?query=workflow%3Abuild)

{{ docs }}

## Benchmarks

The following shows a violin plot of the benchmark results for `upon` compared
to the following template rendering engines.
- [handlebars](https://crates.io/crates/handlebars)
- [liquid](https://crates.io/crates/liquid)
- [minijinja](https://crates.io/crates/minijinja)
- [tera](https://crates.io/crates/tera)
- [tinytemplate](https://crates.io/crates/tinytemplate)

Obviously, each of these engines has a completely different feature set so this
just compares the performance of some of the features that they share.

![Violin plot of benchmark results](./benches/results/violin.svg)

**Host**
- MacBook Pro (14-inch, 2021)
- Chipset: Apple M1 Pro
- Memory: 16 GB

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
