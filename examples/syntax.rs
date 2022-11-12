fn main() -> upon::Result<()> {
    let syntax = upon::Syntax::builder()
        .expr("<?", "?>") // used to emit expressions (e.g. variables)
        .block("<%", "%>") // used for for loops, conditionals and with blocks
        // .comment("<#", "#>") // excluding a delimiter essentially disables it
        .build();

    let out = upon::Engine::with_syntax(syntax)
        .compile(
            "
<%- if user.is_enabled %>

Hello <? user.name ?>!

<% endif -%>
",
        )?
        .render(upon::value! {
            user: {
                is_enabled: true,
                name: "John Smith",
            }
        })?;

    println!("{}", out);

    Ok(())
}
