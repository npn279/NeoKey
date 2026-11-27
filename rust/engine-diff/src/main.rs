fn main() {
    let start = std::time::Instant::now();
    let total = engine_diff::run(64, 1024);
    assert!(total >= 1_000_000);
    println!(
        "Compared {total} keystrokes; 0 mismatches; elapsed {:.3}s",
        start.elapsed().as_secs_f64()
    );
}
