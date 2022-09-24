//! Demonstrates how you can implement a file loader with `upon`.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let template_dir = PathBuf::from_iter([env!("CARGO_WORKSPACE_DIR"), "examples", "templates"]);

    let mut engine = upon::Engine::new();
    add_templates(&mut engine, &template_dir)?;

    let result = engine
        .get_template("index")
        .unwrap()
        .render(upon::value! { title: "My Webpage!", year: 2022 })?;
    println!("{}", result);

    Ok(())
}

/// Adds all HTML templates in the given directory to the engine, using the
/// file name as the template name.
fn add_templates(engine: &mut upon::Engine<'_>, dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();

        if path.extension() != Some(OsStr::new("html")) {
            continue;
        }

        // Converts a file name like 'templates/index.html' to the name 'index'
        let name = path
            .file_stem()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();

        // Reads the template source
        let text = fs::read_to_string(&path)?;

        // Compiles and add the template to the engine!
        engine.add_template(name, text)?;
    }
    Ok(())
}
