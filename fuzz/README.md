# fuzz

First install `cargo-fuzz`

```sh
cargo install cargo-fuzz
```

Then make sure you have the latest nightly

```sh
rustup update nightly
```

Then copy the seeds to the `corpus` directory

```sh
mkdir -p fuzz/corpus/compile fuzz/corpus/render
cp fuzz/seeds/compile/* fuzz/corpus/compile/
cp fuzz/seeds/compile/* fuzz/corpus/render/
```

Then run the fuzzer for each target

```sh
cargo +nightly fuzz run compile -- -max_len=100 -jobs=8
```

and

```sh
cargo +nightly fuzz run render -- -max_len=100 -jobs=8
```
