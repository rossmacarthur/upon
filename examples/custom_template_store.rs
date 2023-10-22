use std::collections::HashMap;

fn main() -> upon::Result<()> {
    let engine = upon::Engine::new();

    // Construct our custom template store
    let store = {
        let mut s = HashMap::<&'static str, upon::Template<'static>>::new();
        s.insert(
            "index",
            engine.compile(include_str!("templates/index.html"))?,
        );
        s.insert(
            "header",
            engine.compile(include_str!("templates/header.html"))?,
        );
        s.insert(
            "footer",
            engine.compile(include_str!("templates/footer.html"))?,
        );
        s
    };

    // Get the template from the store
    let template = store.get("index").unwrap();

    // Render the template using the provided data
    let output = template
        .render(&engine, upon::value! { title: "My Webpage!", year: 2022 })
        .with_template_fn(|name| {
            store
                .get(name)
                .ok_or_else(|| String::from("template not found"))
        })
        .to_string()?;

    println!("{output}");

    Ok(())
}
