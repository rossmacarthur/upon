## 0.7.0

*Unreleased*

### Features

- [Redesign the render API.][51d011b] This improvement allows us to add more
  configuration to the rendering process in the future. It also prevents the
  need for a proliferation of rendering functions. The new public `Renderer`
  struct is created using one of the three functions: `render`, `render_from`,
  or `render_from_fn`. And then the renderer can be consumed using
  `.to_string()` or `.to_writer(..)`.
- [Support `?.` optional chaining operator.][5f4f345]
- [Allow values to be fetched lazily.][fb2a904] This allows you to provide
  the global context from an outside source during rendering.
- [Replace ValueCow::as_bool with truthiness function.][a4d3c59] This change
  implements a truthiness function for evaluating conditionals. `None`, `false`,
  `0` integer and `0.0` float, empty string, list, and map as falsy. Everything
  else is truthy.
  *Contributed by [**@lunabunn**](https://github.com/lunabunn)*

[51d011b]: https://github.com/rossmacarthur/upon/commit/51d011b49e70817e9cf2c42b907a0661bd65700b
[5f4f345]: https://github.com/rossmacarthur/upon/commit/5f4f345a2c7b903eba4deab701d9f901e1df0aaf
[fb2a904]: https://github.com/rossmacarthur/upon/commit/fb2a90444bd6da9baa15c9e8e9f378a9231e1d5f
[a4d3c59]: https://github.com/rossmacarthur/upon/commit/a4d3c599786505f14ac0ca66834f17a0686c22ec

## 0.6.0

*November 20th, 2022*

### Features

- [Expand supported filter argument types][fbef89f4] Adds the unit type, all
  integer primitive types, and `f32`.

### Fixes

- [Fix bug in `value!` macro][7b43c9a0]
- [Use index implementation for looking up spans][c910bca4] This removes all the
  unsafe code in this library. I think the decrease in performance (about 5-10%)
  is worth not having any unsafe code.

[fbef89f4]: https://github.com/rossmacarthur/upon/commit/fbef89f44e455843a58e468be4d69937c9001066
[7b43c9a0]: https://github.com/rossmacarthur/upon/commit/7b43c9a04e23685d4a34fc5fcc9f2b23e5865f55
[c910bca4]: https://github.com/rossmacarthur/upon/commit/c910bca4382943c53f3be7071b68bf512f598266

## 0.5.0

*November 14th, 2022*

### Features

- [Add methods to remove templates and functions][dd8c2478] `remove_template`
  allows you to remove a template from the engine. `remove_function` allows you
  to remove a formatter or filter from the engine. `add_formatter` and
  `add_filter` now return the previous type of function if a function of the
  same name already existed in the engine.
- [Add some `From` and `FromIterator` impls for `Value`][5f6f70ce]
- [Support and test on Rust 1.60][327c504a]
- [Support Unicode identifiers][9e8ae85e]

### Fixes

- [Parse list indices in lexer][ee02c419] This fixes a bug where a path like
  `lorem.123.ipsum` would not parse correctly because the lexer would return a
  `Token::Number` for the span "123.ipsum". This requires adding an additional
  state to the lexer so that it knows when it is lexing a path versus a float
  literal.
- [Refactor formatter and filter errors][329e8423] Move formatter types to new
  module, add `fmt::Error` type. Add filter error trait and kind.
- [Add template name to error, update error format][d16eaf66]

[ee02c419]: https://github.com/rossmacarthur/upon/commit/ee02c4195b00ad4c584ebbb198519be08bc14ddb
[dd8c2478]: https://github.com/rossmacarthur/upon/commit/dd8c24781b90467b702dd174a592736bf715d246
[5f6f70ce]: https://github.com/rossmacarthur/upon/commit/5f6f70ce66c40fccd99093b6c0428fc83a7e2aad
[327c504a]: https://github.com/rossmacarthur/upon/commit/327c504a278de8b41b90676c44be879498fabbbd
[9e8ae85e]: https://github.com/rossmacarthur/upon/commit/9e8ae85ef380efab840c402bc0227948016c9c90
[329e8423]: https://github.com/rossmacarthur/upon/commit/329e842339ffb09200b609830ac0dba9742fdc99
[d16eaf66]: https://github.com/rossmacarthur/upon/commit/d16eaf662347069f7642590ab34ef6a387ab3889

## 0.4.0

*October 3rd, 2022*

### Features

- [Make various filter traits public][ee694cd5] This allows you to refer to them
  if you want to be generic over filters.
- [Support `else if`][68da8b14] This is simply desugared to a nested `if`
  statement in the parser.
- [Support scientific notation for float literals][e2b6367f]
- [Use `Cow<str>` for map keys and template sources][270252b8] This allows you
  to store template sources computed at runtime in the engine while preserving
  support for templates included in the binary.

### Fixes

- [Fix bug with include statement scopes][88feac35]
- [Restrict maximum include depth][509526ba]. This prevents infinite recursion
  when including templates. This can be configured using the
  `set_max_include_depth` method on an `Engine`.
- [Disallow configuring syntax using empty strings][be1acdd5] We now panic if
  you pass an empty string to `SyntaxBuilder`.

[ee694cd5]: https://github.com/rossmacarthur/upon/commit/ee694cd558a45e0735693894e9afb0e77329a4ef
[68da8b14]: https://github.com/rossmacarthur/upon/commit/68da8b14c4571826e21eac0d278c41fbda37fd92
[e2b6367f]: https://github.com/rossmacarthur/upon/commit/e2b6367f95d1ea0e3e89c94cbb04b729d59fd057
[be1acdd5]: https://github.com/rossmacarthur/upon/commit/be1acdd5ae244e93ccfc63f99ed96ae66e573756
[270252b8]: https://github.com/rossmacarthur/upon/commit/270252b80260185a4493291ea2c829a953e3a12e
[88feac35]: https://github.com/rossmacarthur/upon/commit/88feac35ce7c03ebaa3e6147d416407903f852d1
[509526ba]: https://github.com/rossmacarthur/upon/commit/509526ba951e2ca33566e24f27adbc478591c954
