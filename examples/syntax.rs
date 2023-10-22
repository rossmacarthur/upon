fn main() -> upon::Result<()> {
    let syntax = upon::Syntax::builder()
        .expr("<?", "?>") // used to emit expressions (e.g. variables)
        .block("<%", "%>") // used for for loops, conditionals and with blocks
        // .comment("<#", "#>") // excluding a delimiter essentially disables it
        .build();

    let engine = upon::Engine::with_syntax(syntax);

    let out = engine
        .compile(
            "
<%- if user.is_enabled %>

Hello <? user.name ?>!

<% endif -%>
",
        )?
        .render(
            &engine,
            upon::value! {
                user: {
                    is_enabled: true,
                    name: "John Smith",
                }
            },
        )
        .to_string()?;

    println!("{out}");

    Ok(())
}
