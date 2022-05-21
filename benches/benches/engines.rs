//! Benchmark template compilation and rendering time.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;

criterion_main! { benches }
criterion_group! { benches, bench_compile, bench_render }

fn bench_compile(c: &mut Criterion) {
    let mut g = c.benchmark_group("compile");

    const N: usize = 20;

    g.bench_function("handlebars", |b| {
        let mut hbs = handlebars::Handlebars::new();
        let source = repeat(include_str!("benchdata/handlebars.html"), N);
        b.iter(|| black_box(hbs.register_template_string("bench", &source).unwrap()));
    });

    g.bench_function("minijinja", |b| {
        let mut env = minijinja::Environment::new();
        let source = repeat(include_str!("benchdata/liquid.html"), N);
        b.iter(|| black_box(env.add_template("bench", &source).unwrap()));
    });

    g.bench_function("tera", |b| {
        let mut tera = tera::Tera::default();
        let source = repeat(include_str!("benchdata/liquid.html"), N);
        b.iter(|| black_box(tera.add_raw_template("bench", &source).unwrap()));
    });

    g.bench_function("tinytemplate", |b| {
        let mut tt = tinytemplate::TinyTemplate::new();
        let source = repeat(include_str!("benchdata/tinytemplate.html"), N);
        b.iter(|| black_box(tt.add_template("bench", &source).unwrap()));
    });

    g.bench_function("upon", |b| {
        let mut engine = upon::Engine::new();
        let source = repeat(include_str!("benchdata/liquid.html"), N);
        b.iter(|| black_box(engine.add_template("bench", &source).unwrap()));
    });
}

fn bench_render(c: &mut Criterion) {
    let mut g = c.benchmark_group("render");

    let ctx = random_context(150);

    const N: usize = 20;

    g.bench_function("handlebars", |b| {
        let mut hbs = handlebars::Handlebars::new();
        let source = repeat(include_str!("benchdata/handlebars.html"), N);
        hbs.register_template_string("bench", &source).unwrap();
        b.iter(|| black_box(hbs.render("bench", &ctx).unwrap()));
    });

    g.bench_function("minijinja", |b| {
        let mut env = minijinja::Environment::new();
        let source = repeat(include_str!("benchdata/liquid.html"), N);
        env.add_template("bench", &source).unwrap();
        b.iter(|| black_box(env.get_template("bench").unwrap().render(&ctx).unwrap()));
    });

    g.bench_function("tera", |b| {
        let mut tera = tera::Tera::default();
        let source = repeat(include_str!("benchdata/liquid.html"), N);
        tera.add_raw_template("bench", &source).unwrap();
        let ctx = tera::Context::from_serialize(&ctx).unwrap();
        b.iter(|| black_box(tera.render("bench", &ctx).unwrap()));
    });

    g.bench_function("tinytemplate", |b| {
        let mut tt = tinytemplate::TinyTemplate::new();
        let source = repeat(include_str!("benchdata/tinytemplate.html"), N);
        tt.add_template("bench", &source).unwrap();
        b.iter(|| black_box(tt.render("bench", &ctx).unwrap()));
    });

    g.bench_function("upon", |b| {
        let mut engine = upon::Engine::new();
        let source = repeat(include_str!("benchdata/liquid.html"), N);
        engine.add_template("bench", &source).unwrap();
        b.iter(|| black_box(engine.get_template("bench").unwrap().render(&ctx).unwrap()));
    });
}

#[derive(serde::Serialize)]
struct Context {
    title: String,
    users: Vec<User>,
}

#[derive(serde::Serialize)]
struct User {
    name: String,
    age: u32,
    is_enabled: bool,
}

fn random_context(n: usize) -> Context {
    let mut rng = rand::thread_rng();
    let title = (0..20).map(|_| rng.gen_range('a'..='z')).collect();
    let users = (0..n)
        .map(|_| User {
            name: (0..20).map(|_| rng.gen_range('a'..='z')).collect(),
            age: rng.gen_range(21..100),
            is_enabled: rng.gen_ratio(3, 4),
        })
        .collect();
    Context { title, users }
}

fn repeat(source: &str, n: usize) -> String {
    let mut s = String::new();
    for _ in 0..n {
        s.push_str(source);
    }
    s
}
