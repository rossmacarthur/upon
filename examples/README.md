# examples

Examples can be run from anywhere in the repo using the following.

```sh
cargo run --example <name>
```

- [quick](quick.rs): Demonstrates compiling and rendering a template in one line
  of code.

- [serde](serde.rs): Demonstrates rendering a simple template using a custom
  global context that implements `serde::Serialize`.

- [syntax](syntax.rs): Demonstrates how to configure the template engine with
  custom syntax delimiters.

- [escape_html](escape_html.rs): Demonstrates how to configure a _value
  formatter_ to escape strings for HTML and add an "unescape" _value formatter_
  for outputting values unescaped.

- [filters](filters.rs): Demonstrates how to configure custom filters for
  transforming data before rendering.

- [static_templates](static_templates.rs): Demonstrates how to statically
  include templates in the binary using the `include_str!` macro.

- [runtime_templates](runtime_templates.rs): Demonstrates how to implement a
  file loader. Files are are loaded at _runtime_ from the `templates/` directory
  and added to the engine.

- [render_to_writer](render_to_writer.rs): Demonstrates how to render directly
  to a type implementing `std::io::Write` instead of to a string.
