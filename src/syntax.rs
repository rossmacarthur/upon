//! Documents the template syntax.
//!
//! An `upon` template is simply a piece of text. Whether it is read from a file
//! or included directly in the binary is irrelevant. A template contains
//! [**expressions**](#expressions) for rendering values and
//! [**blocks**](#blocks) for controlling logic. These require you to use
//! specific syntax tags/delimiters in the template. Because `upon` allows you
//! to configure these tags/delimiters, this document will only refer to the
//! [**default**][crate::Syntax::default] configuration.
//!
//! # Expressions
//!
//! Expressions are marked with `{{ ... }}` and are used to render single
//! values. This allows you to refer to a variable in the current scope. The
//! following would lookup the field "name" in the current scope and insert it
//! into the rendered output.
//!
//! ```text
//! Hello {{ name }}!
//! ```
//!
//! You can access nested fields using a dotted path. The following would first
//! lookup the field "user" and then lookup the field "name" within it.
//!
//! ```text
//! Hello {{ user.name }}!
//! ```
//!
//! You can also use this syntax to lookup a particular index of a list. For
//! each of the expressions in the following code, first the field "users" is
//! looked up and then a particular use is selected from the list by index.
//! Finally, the field "name" is looked up from the selected user.
//!
//! ```text
//! Hello {{ users.0.name }}!
//! And hello {{ users.1.name }}!
//! And also hello {{ users.2.name }}!
//! ```
//!
//! # Blocks
//!
//! Blocks are marked with an opening `{% ... %}` and a closing `{% ... %}`.
//! There are two kinds of blocks: [**conditionals**](#conditionals) and
//! [**loops**](#loops).
//!
//! ## Conditionals
//!
//! Conditionals are marked using an opening `if` block and a closing `endif`
//! block. It can also have an optional `else` clause. A conditional renders the
//! contents of the block based on the specified condition which can be any
//! [**expression**](#expressions) but it must resolve to a boolean value.
//!
//! Consider the following template. This would only render the HTML table if
//! the nested field `user.is_enabled` returns `true` and only render the
//! paragraph if it returns `false`.
//!
//! ```html
//! {% if user.is_enabled %}
//!     <table>...</table>
//! {% else %}
//!     <p>User is disabled</p>
//! {% endif %}
//! ```
//!
//! A boolean expression can be negated by using `if not`.
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
//! # Filters
//!
//! In all places where [**expressions**](#expressions) are valid, filters can
//! be applied to the value using the `|` operator. The simplest filters take no
//! extra arguments and are just specified by name.
//!
//! For example, assuming a filter called `lower` is registered in the engine
//! the following would produce an expression that transforms the `user.name`
//! value to lowercase.
//!
//! ```html
//! {{ user.name | lower }}
//! ```
//!
//! Filters can also take arguments which can either be values or literals. In
//! the following we lookup `page.path` and append a suffix to it.
//!
//! ```html
//! {{ page.path | append: ".html" }}
//! ```
//!
//! See the [`Filter`][crate::Filter] trait documentation for more information
//! on filters.
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
