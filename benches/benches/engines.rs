//! Benchmark template compilation and rendering time.

use criterion::{criterion_group, criterion_main, Criterion};

use benches::context;
use benches::{Engine, Handlebars, Minijinja, Tera, TinyTemplate, Upon};

criterion_main! { benches }
criterion_group! { benches, bench_init, bench_compile, bench_render }

/// Benchmarks the time taken to create a new instance of the engine.
fn bench_init(c: &mut Criterion) {
    let mut g = c.benchmark_group("init");

    macro_rules! bench {
        ($E:ty) => {{
            g.bench_function(<$E as Engine>::name(), |b| {
                b.iter(|| <$E as Engine>::new());
            });
        }};
    }

    bench!(Handlebars);
    bench!(Minijinja);
    bench!(Tera);
    bench!(TinyTemplate);
    bench!(Upon);
}

/// Benchmarks the time taken to compile a template.
fn bench_compile(c: &mut Criterion) {
    let mut g = c.benchmark_group("compile");

    macro_rules! bench {
        ($E:ty, $source:literal) => {{
            g.bench_function(<$E as Engine>::name(), |b| {
                let source = repeat(include_str!($source), 50);
                let mut engine = <$E as Engine>::new();
                b.iter(|| engine.add_template("bench", &source));
            });
        }};
    }

    bench!(Handlebars, "../benchdata/handlebars.html");
    bench!(Minijinja, "../benchdata/liquid.html");
    bench!(Tera, "../benchdata/liquid.html");
    bench!(TinyTemplate, "../benchdata/tinytemplate.html");
    bench!(Upon, "../benchdata/liquid.html");
}

/// Benchmarks the time taken to render a template as a string.
fn bench_render(c: &mut Criterion) {
    let mut g = c.benchmark_group("render");

    let ctx = context::random(150);

    macro_rules! bench {
        ($E:ty, $source:literal) => {{
            g.bench_function(<$E as Engine>::name(), |b| {
                let source = repeat(include_str!($source), 20);
                let mut engine = <$E as Engine>::new();
                <$E as Engine>::add_template(&mut engine, "bench", &source);
                b.iter(|| <$E as Engine>::render(&engine, "bench", &ctx));
            });
        }};
    }

    bench!(Handlebars, "../benchdata/handlebars.html");
    bench!(Minijinja, "../benchdata/liquid.html");
    bench!(Tera, "../benchdata/liquid.html");
    bench!(TinyTemplate, "../benchdata/tinytemplate.html");
    bench!(Upon, "../benchdata/liquid.html");
}

fn repeat(source: &str, n: usize) -> String {
    let mut s = String::new();
    for _ in 0..n {
        s.push_str(source);
    }
    s
}
