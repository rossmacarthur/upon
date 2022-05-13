# upon

A tiny template engine.

## Features

- Rendering values: `{{ user.name }}`
- Conditionals: `{% if user.enabled %} ... {% endif %}`
- Loops: `{% for user in users %} ... {% endfor %}`
- Customizable filter functions: `{{ value | my_filter }}`
- Configurable template delimiters: `<? user.name ?>`, `(( if user.enabled ))`
- Render any [`serde`][serde] serializable values.
- Macro for quick rendering: `data!{ name: "John", age: 42 }`

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
