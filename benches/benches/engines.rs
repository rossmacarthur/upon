//! Benchmark template compilation and rendering time.

use criterion::{criterion_group, criterion_main, Criterion};

use benches::{context, Liquid};
use benches::{Engine, Handlebars, Minijinja, Tera, TinyTemplate, Upon};

criterion_main! { benches }

criterion_group! {
    benches,
    bench_init,
    bench_compile,
    bench_render,
}

/// Benchmarks the time taken to create a new instance of the engine.
pub fn bench_init(c: &mut Criterion) {
    let mut g = c.benchmark_group("init");

    macro_rules! bench {
        ($E:ty) => {{
            g.bench_function(<$E as Engine>::name(), |b| {
                b.iter(|| <$E as Engine>::new());
            });
        }};
    }

    bench!(Handlebars);
    bench!(Liquid);
    bench!(Minijinja);
    bench!(Tera);
    bench!(TinyTemplate);
    bench!(Upon);
}

/// Benchmarks the time taken to compile a template.
pub fn bench_compile(c: &mut Criterion) {
    macro_rules! bench {
        ($g:ident, $E:ty, $source:literal) => {{
            $g.bench_function(<$E as Engine>::name(), |b| {
                let source = repeat(include_str!(concat!("../benchdata/", $source)), 50);
                let mut engine = <$E as Engine>::new();
                b.iter(|| engine.add_template("bench", &source));
            });
        }};
    }

    macro_rules! bench_with_syntax {
        ($g:ident, $E:ty, $source:literal) => {{
            $g.bench_function(<$E as Engine>::name(), |b| {
                let source = repeat(include_str!(concat!("../benchdata/", $source)), 50);
                let mut engine =
                    <$E as Engine>::with_syntax(("{", "}"), ("<%", "%>"), ("<#", "#>"));
                b.iter(|| engine.add_template("bench", &source));
            });
        }};
    }

    {
        let mut g = c.benchmark_group("compile/basic");
        // 8 times slower than the next slowest, leave out so the chart looks reasonable:
        // bench!(g, Handlebars, "basic/handlebars.html");
        bench!(g, Liquid, "basic/liquid.html");
        bench!(g, Minijinja, "basic/jinja.html");
        bench!(g, Tera, "basic/jinja.html");
        bench!(g, TinyTemplate, "basic/tinytemplate.html");
        bench!(g, Upon, "basic/jinja.html");
    }

    {
        let mut g = c.benchmark_group("compile/syntax");
        bench_with_syntax!(g, Minijinja, "syntax/jinja.html");
        bench_with_syntax!(g, Upon, "syntax/jinja.html");
    }

    {
        let mut g = c.benchmark_group("compile/literals");
        bench!(g, Minijinja, "literals/jinja.html");
        bench!(g, Upon, "literals/jinja.html");
    }
}

/// Benchmarks the time taken to render a template as a string.
pub fn bench_render(c: &mut Criterion) {
    let ctx = context::random(150);

    macro_rules! bench {
        ($g:ident, $E:ty, $source:literal) => {{
            $g.bench_function(<$E as Engine>::name(), |b| {
                let source = repeat(include_str!(concat!("../benchdata/", $source)), 50);
                let mut engine = <$E as Engine>::new();
                <$E as Engine>::add_template(&mut engine, "bench", &source);
                b.iter(|| <$E as Engine>::render(&engine, "bench", &ctx));
            });
        }};
    }

    {
        let mut g = c.benchmark_group("render/basic");
        bench!(g, Handlebars, "basic/handlebars.html");
        bench!(g, Liquid, "basic/liquid.html");
        bench!(g, Minijinja, "basic/jinja.html");
        bench!(g, Tera, "basic/jinja.html");
        bench!(g, TinyTemplate, "basic/tinytemplate.html");
        bench!(g, Upon, "basic/jinja.html");
    }

    {
        let mut g = c.benchmark_group("render/filters");
        bench!(g, Handlebars, "filters/handlebars.html");
        bench!(g, Minijinja, "filters/jinja.html");
        bench!(g, Tera, "filters/jinja.html");
        bench!(g, Upon, "filters/jinja.html");
    }
}

fn repeat(source: &str, n: usize) -> String {
    let mut s = String::new();
    for _ in 0..n {
        s.push_str(source);
    }
    s
}
