use std::io;

fn main() -> upon::Result<()> {
    let mut stdout = io::BufWriter::new(io::stdout());

    let ctx = upon::value! { user: { name: "John Smith" } };

    upon::Engine::new()
        .compile("Hello {{ user.name }}!\n")?
        .render_to_writer(&mut stdout, ctx)?;

    Ok(())
}
