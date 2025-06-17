//! Documents the template syntax.
//!
//! An `upon` template is simply a piece of UTF-8 text. It can be embedded in
//! the binary or provided at runtime (e.g. read from a file). A template
//! contains [**expressions**](#expressions) for rendering values and
//! [**blocks**](#blocks) for controlling logic. These require you to use
//! specific syntax delimiters in the template. Because `upon` allows you to
//! configure these delimiters, this document will only refer to the **default**
//! configuration.
//!
//! # Expressions
//!
//! Expressions are available everywhere in templates and they can be emitted by
//! wrapping them in `{{ ... }}`.
//!
//! ## Literals
//!
//! The simplest form of expressions are literals. The following types are
//! available in templates.
//!
//! - Booleans: `true`, `false`
//! - Integers: `42`, `0o52`, `-0x2a`
//! - Floats: `0.123`, `-3.14`, `5.23e10`
//! - Strings: `"Hello World!"`, escape characters are supported: `\r`, `\n`,
//!   `\t`, `\\`, `\"`
//! - Lists: `[1, 2, 3]`, `[value, "string", 3.14]`
//! - Maps: `{"a": value, "b": "string", "c": 3.14}`, literal map keys are
//!   always constant strings
//!
//! Both lists and maps can contain any type of value including literals and
//! [values](#values).
//!
//! ## Values
//!
//! You can lookup up existing values in the current scope by name. The
//! following would lookup the field "name" in the current scope and insert it
//! into the rendered output.
//!
//! ```text
//! Hello {{ .name }}!
//! ```
//!
//! You can access nested fields using a dotted path. The following would first
//! lookup the field "user" and then lookup the field "name" within it.
//!
//! ```text
//! Hello {{ .user.name }}!
//! ```
//!
//! For convenience, you can also omit the leading dot (`.`) in the first path
//! member. The following is equivalent to the previous example.
//! ```text
//! Hello {{ user.name }}!
//! ```
//!
//! You can also use this syntax to lookup a particular index of a list. For
//! each of the expressions in the following code, first the field "users" is
//! looked up, then a particular user is selected from the list by index.
//! Finally, the field "name" is looked up from the selected user.
//!
//! ```text
//! Hello {{ users.0.name }}!
//! And hello {{ users.1.name }}!
//! And also hello {{ users.2.name }}!
//! ```
//!
//! The dotted path syntax will raise an error when the field or index is not
//! found. If you want to try lookup a field and return [`Value::None`] when it
//! is not found then you can use the optional dotted path syntax (`?.`). The
//! following would try lookup the field "surname" from "user" and return
//! [`Value::None`] if it is not found.
//!
//! ```text
//! Hello {{ user.name }} {{ user?.surname }}!
//! ```
//!
//! This is useful when checking if the first member of a path might not be
//! defined.
//! ```text
//! {% if ?.user %} ... {% endif %}
//! ```
//!
//! [`Value::None`]: crate::Value::None
//!
//! ## Filters
//!
//! Filters are functions that can be applied to existing expressions using the
//! `|` (pipe) operator. The simplest filters take no extra arguments and are
//! just specified by name. For example, assuming a function called `lower` is
//! registered in the engine the following would produce an expression with the
//! `user.name` value transformed to lowercase.
//!
//! ```html
//! {{ user.name | lower }}
//! ```
//!
//! Filters can also take arguments which must be a sequence of comma separated
//! values or literals. In the following we lookup the value `page.path` and
//! append a suffix to it.
//!
//! ```html
//! {{ page.path | append: ".html" }}
//! ```
//!
//! See the [`functions`][crate::functions] module documentation for more
//! information on filters.
//!
//! ## Functions
//!
//! Functions can also be called with the `name(args...)` syntax. This allows
//! you to use functions in other contexts like arguments to other functions or
//! at the start of an expression. For example, assuming a function called
//! `now` is registered in the engine, the following would produce an expression
//! with the current date and time.
//!
//! ```html
//! {{ now() }}
//! ```
//!
//! Functions can also take arguments which must be a sequence of comma
//! separated values, literals, or function calls. In the following we call a
//! function `add` with two arguments.
//!
//! ```html
//! {{ add(1, 2) }}
//! ```
//!
//! See the [`functions`][crate::functions] module documentation for more
//! information on functions.
//!
//! # Blocks
//!
//! Blocks are marked with an opening `{% ... %}` and a closing `{% ... %}`.
//!
//! ## Conditionals
//!
//! Conditionals are marked using an opening `if` block and a closing `endif`
//! block. It can also have zero or more optional `else if` clauses and an
//! optional `else` clause. A conditional renders the contents of the block
//! based on the specified condition which can be any
//! [**expression**](#expressions). An expression can be negated by applying the
//! prefix `not`. The conditional evaluates the expression based on it's
//! truthiness. The following values are considered falsy, every other value is
//! truthy and will pass the condition:
//! - `None`
//! - Boolean `false`
//! - An integer with value `0`
//! - A float with value `0.0`
//! - An empty string
//! - An empty list
//! - An empty map
//!
//! Consider the following template. If the nested field `user.is_enabled` is
//! returns `false` then the first paragraph would be rendered. Otherwise if
//! `user.has_permission` returns true then the second paragraph would be
//! rendered. If neither condition is satisfied then the HTML table would be
//! rendered.
//!
//! ```html
//! {% if not user.is_enabled %}
//!     <p>User is disabled</p>
//! {% else if user.has_permission %}
//!     <p>User has insufficient permissions</p>
//! {% else %}
//!     <table>...</table>
//! {% endif %}
//! ```
//!
//! ## Loops
//!
//! Loops are marked using an opening `for` block and a closing `endfor` block.
//! A loop renders the contents of the block once for each item in the specified
//! sequence. This is done by unpacking each item into one or two variables.
//! These variables are added to the scope within the loop block and shadow any
//! variables with the same name in the outer scope. The specified sequence can
//! be any [**expression**](#expressions) but it must resolve to a list or map.
//! Additionally, for lists there must a single loop variable and for maps there
//! must be key and value loop variables.
//!
//! Consider the following template. This would render an HTML paragraph for
//! each user in the list.
//!
//! ```html
//! {% for user in users %}
//!     <p>{{ user.name }}</p>
//! {% endfor %}
//! ```
//!
//! Here is an example where `users` is a map.
//!
//! ```html
//! {% for id, user in users %}
//!     <div>
//!         <p>ID: {{ id }}</p>
//!         <p>Name: {{ user.name }}</p>
//!     </div>
//! {% endfor %}
//! ```
//!
//! Additionally, there are three special values available within loops.
//!
//! - `loop.index`: a zero-based index of the current value in the iterable
//! - `loop.first`: `true` if this is the first iteration of the loop
//! - `loop.last`: `true` if this is the last iteration of the loop
//!
//! ```html
//! <ul>
//! {% for user in users %}
//!     <li>{{ loop.index }}. {{ user.name }}</li>
//! {% endfor %}
//! </ul>
//! ```
//!
//! ## With
//!
//! "With" blocks can be used to create a variable from an
//! [**expression**](#expressions). The variable is only valid within the block
//! and it shadows any outer variables with the same name.
//!
//! ```html
//! {% with user.names | join: " " as fullname %}
//!     Hello {{ fullname }}!
//! {% endwith %}
//! ```
//!
//! ## Include
//!
//! "Include" blocks can be used to render nested templates. The nested template
//! must have been registered in the engine before rendering. For example,
//! assuming a template "footer" has been registered in the engine the following
//! would render the template "footer" in the place of the include block. All
//! variables in the current template will be available to the nested template.
//!
//! ```html
//! <body>
//!     ...
//!
//!     {% include "footer" %}
//!
//! </body>
//! ```
//!
//! You can also include the nested using a specific context. In this case the
//! nested template would not have any access to the current template's
//! variables and `path.to.footer.info` would form the global context for the
//! nested template.
//!
//! ```html
//! <body>
//!     ...
//!
//!     {% include "footer" with path.to.footer.info %}
//!
//! </body>
//! ```
//!
//! Self-referential templates and include cycles are allowed but the maximum
//! include depth is restricted by the engine setting
//! [`set_max_include_depth`][crate::Engine::set_max_include_depth].
//!
//! # Whitespace control
//!
//! If an expression or block includes a hyphen `-` character, like `{{-`,
//! `-}}`, `{%-`, and `-%}` then any whitespace in the template adjacent to the
//! tag will be skipped when the template is rendered.
//!
//! Consider the following template.
//!
//! ```text
//! Hello,
//!     {% if user.is_welcome %} and welcome, {% endif %}
//!     {{ user.name }}!
//! ```
//!
//! This doesn't use any whitespace trimming so it would be rendered something
//! like this:
//! ```text
//! Hello,
//!      and welcome,
//!     John!
//! ```
//!
//! We can add whitespace trimming like this.
//! ```text
//! Hello,
//!     {%- if user.is_welcome %} and welcome, {% endif -%}
//!     {{ user.name }}!
//! ```
//!
//! Now it will be rendered without the newlines or extra spaces.
//! ```text
//! Hello, and welcome, John!
//! ```
