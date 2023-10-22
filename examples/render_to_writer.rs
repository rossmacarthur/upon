use std::io;

fn main() -> upon::Result<()> {
    let mut stdout = io::BufWriter::new(io::stdout());

    let engine = upon::Engine::new();

    let ctx = upon::value! { user: { name: "John Smith" } };

    engine
        .compile("Hello {{ user.name }}!\n")?
        .render(&engine, ctx)
        .to_writer(&mut stdout)?;

    Ok(())
}
