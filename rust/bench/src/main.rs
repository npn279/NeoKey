mod callback;

use engine_diff::{benchmark, vietnamese_keystrokes, Key, CORPUS};
use neokey_engine::{
    data::{vReplaceMaro, vRestoreAndStartNewSession},
    Engine, Settings,
};
use std::{
    fs::{self, File},
    hint::black_box,
    io::{BufWriter, Write},
    path::PathBuf,
    time::Instant,
};

const WARMUP: usize = 12;
const BATCH_TRIALS: usize = 101;
const KEY_TRIALS: usize = 21;

fn add_macros(engine: &mut Engine) {
    for (key, value) in benchmark::MACROS {
        assert!(engine.add_macro(key, value));
    }
}

fn rust_key(engine: &mut Engine, key: Key) -> u8 {
    let hook = black_box(engine.handle_event(key.event, key.state, key.data, key.caps, key.ctrl));
    let code = hook.code;
    if i64::from(code) == vRestoreAndStartNewSession {
        engine.start_new_session();
    }
    code
}

#[derive(Clone, Copy)]
enum Sample {
    Warmup,
    Batch,
    Key,
}

fn sample<T, R>(keys: &[T], kind: Sample, mut process: impl FnMut(&T) -> R) -> Vec<u128> {
    match kind {
        Sample::Warmup | Sample::Batch => {
            let start = Instant::now();
            for key in black_box(keys) {
                black_box(process(key));
            }
            vec![start.elapsed().as_nanos()]
        }
        Sample::Key => {
            let mut times = Vec::with_capacity(keys.len());
            for key in black_box(keys) {
                let start = Instant::now();
                black_box(process(key));
                times.push(start.elapsed().as_nanos());
            }
            times
        }
    }
}

fn measure(
    out: &mut impl Write,
    scope: &str,
    case: &str,
    keys: usize,
    mut run: impl FnMut(&str, Sample) -> Vec<u128>,
) -> std::io::Result<()> {
    eprintln!("{scope} {case} ({keys} keys per trial)");
    let mut trial = 0;
    for (count, sample, label, denominator) in [
        (WARMUP, Sample::Warmup, "warmup", keys),
        (BATCH_TRIALS, Sample::Batch, "batch", keys),
        (KEY_TRIALS, Sample::Key, "key", 1),
    ] {
        for local_trial in 0..count {
            let order = if trial % 2 == 0 {
                ["cpp", "rust"]
            } else {
                ["rust", "cpp"]
            };
            trial += 1;
            // Time both sides before formatting or writing the raw rows.
            let results = order.map(|side| (side, run(side, sample)));
            if matches!(sample, Sample::Warmup) {
                continue;
            }
            for (side, times) in results {
                for (index, ns) in times.into_iter().enumerate() {
                    writeln!(
                        out,
                        "{scope},{case},{side},{label},{local_trial},{index},{denominator},{ns}"
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut args = std::env::args_os().skip(1);
    let dir = PathBuf::from(args.next().expect("results directory"));
    let private_source = args.next().is_some_and(|arg| arg == "--private-source");
    callback::configure(private_source);
    callback::verify_screen_model();
    fs::create_dir_all(&dir)?;
    fs::write(
        dir.join("callback_source.txt"),
        if private_source {
            "private\n"
        } else {
            "null\n"
        },
    )?;
    let mut raw = BufWriter::new(File::create(dir.join("timings.csv"))?);
    let mut checks = BufWriter::new(File::create(dir.join("checks.csv"))?);
    writeln!(raw, "scope,case,side,sample,trial,index,keys,ns")?;
    writeln!(checks, "scope,case,keys,macro_hits,mismatched_callback_snapshots,cpp_posted,rust_posted,final_screen_equal")?;
    let mut timer = BufWriter::new(File::create(dir.join("timer.csv"))?);
    writeln!(timer, "ns")?;
    let clock: Vec<_> = (0..50_000)
        .map(|_| {
            let start = Instant::now();
            black_box(());
            start.elapsed().as_nanos()
        })
        .collect();
    for ns in clock {
        writeln!(timer, "{ns}")?;
    }

    let random = benchmark::random(2048);
    for (input, mode) in [(0, "telex"), (1, "vni")] {
        let real: Vec<_> = vietnamese_keystrokes(CORPUS, input)
            .chars()
            .map(Key::character)
            .collect();
        for (corpus, keys) in [("text", &real), ("random", &random)] {
            for (spell, macros) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let settings = Settings {
                    vInputType: input,
                    vFreeMark: 0,
                    vCheckSpelling: spell,
                    vUseMacro: macros,
                    ..Settings::default()
                };
                let case = format!("{mode}-{corpus}-spell{spell}-macro{macros}");
                assert_eq!(benchmark::verify(settings.clone(), keys), keys.len());
                let mut engine = Engine::with_settings(settings.clone());
                add_macros(&mut engine);
                let hits = keys
                    .iter()
                    .filter(|&&key| i64::from(rust_key(&mut engine, key)) == vReplaceMaro)
                    .count();
                writeln!(checks, "engine,{case},{},{hits},,,,", keys.len())?;
                measure(&mut raw, "engine", &case, keys.len(), |side, kind| {
                    if side == "cpp" {
                        benchmark::reference_setup(&settings);
                        sample(keys, kind, |key| benchmark::reference_key(*key))
                    } else {
                        let mut engine = Engine::with_settings(settings.clone());
                        add_macros(&mut engine);
                        sample(keys, kind, |key| rust_key(&mut engine, *key))
                    }
                })?;
            }

            let settings = Settings {
                vInputType: input,
                vFreeMark: 0,
                vUseMacro: 1,
                vSwitchKeyStatus: 0xfe00_00fe,
                ..Settings::default()
            };
            eprintln!("Preparing synthetic events for {mode}-{corpus}");
            let events = callback::input_events(keys);
            for (step, output) in [(false, "batch"), (true, "step")] {
                let case = format!("{mode}-{corpus}-{output}");
                eprintln!("Validating callback {case}");
                let (diffs, cpp, rust, equal) = callback::verify(&events, &settings, step);
                writeln!(
                    checks,
                    "callback,{case},{},,{diffs},{cpp},{rust},{equal}",
                    events.len()
                )?;
                measure(&mut raw, "callback", &case, events.len(), |side, kind| {
                    if side == "cpp" {
                        callback::cpp_setup(&settings, step);
                        sample(&events, kind, |event| callback::cpp_key(event))
                    } else {
                        let mut state = callback::RustCallback::new(&settings, step);
                        sample(&events, kind, |event| state.key(event))
                    }
                })?;
            }
        }
    }
    checks.flush()?;
    raw.flush()?;
    timer.flush()
}
