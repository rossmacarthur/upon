# upon

[![Crates.io Version](https://badgers.space/crates/version/upon)](https://crates.io/crates/upon)
[![Docs.rs Latest](https://badgers.space/badge/docs.rs/latest/orange)](https://docs.rs/upon)
[![Build Status](https://badgers.space/github/checks/rossmacarthur/upon?label=build)](https://github.com/rossmacarthur/upon/actions/workflows/build.yaml?query=branch%3Atrunk)

{{ summary }}

## Table of Contents

{{ toc }}- [Benchmarks](#benchmarks)
- [License](#license)

{{ contents }}

## Benchmarks

`upon` was benchmarked against several popular template rendering engines in the
Rust ecosystem. Obviously, each of these engines has a completely different
feature set so the benchmark just compares the performance of some of the
features that they share.

- [handlebars](https://crates.io/crates/handlebars) v6.3.0
- [liquid](https://crates.io/crates/liquid) v0.26.9
- [minijinja](https://crates.io/crates/minijinja) v2.6.0
- [tera](https://crates.io/crates/tera) v1.20.0
- [tinytemplate](https://crates.io/crates/tinytemplate) v1.2.1
- upon v0.9.0

![Violin plot of compile results](./benches/results/compile.svg)
![Violin plot of render results](./benches/results/render.svg)
![Violin plot of render with filters results](./benches/results/filters.svg)

Benchmarking was done using [criterion](https://crates.io/crates/criterion).

**Host**

- Apple M2 Max
- 32 GB RAM
- macOS 15.2
- Rust 1.84.1

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
