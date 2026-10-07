// node scripts/benchmark-resampler.mjs [baseline git ref]
// Requires rustc. Uses a temporary directory; does not modify either checkout.
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const baseline = process.argv[2] ?? 'bc2bc4ce99a48484e435dbedcc35667cd02476b8';
const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'musique-resample-'));
const oldSource = path.join(directory, 'baseline.rs');
const source = path.join(directory, 'benchmark.rs');
const executable = path.join(directory, process.platform === 'win32' ? 'benchmark.exe' : 'benchmark');
const rustPath = value => JSON.stringify(value.replaceAll('\\', '/'));
fs.writeFileSync(oldSource, execFileSync('git', ['show', `${baseline}:src-tauri/src/resample.rs`], { cwd: root }));
fs.writeFileSync(source, `
#[path = ${rustPath(oldSource)}]
mod baseline;
#[path = ${rustPath(path.join(root, 'src-tauri/src/resample.rs'))}]
mod optimized;
use std::{hint::black_box, time::Instant};

fn input(channels: usize) -> Vec<f32> {
    let mut seed = 7u32;
    (0..44100 * channels).map(|_| {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (seed as i32 as f32) / (i32::MAX as f32)
    }).collect()
}

fn time_before(data: &[f32]) -> f64 {
    let start = Instant::now();
    let mut resampler = baseline::Resampler::new(44100, 48000, 2).unwrap();
    for _ in 0..12 { for chunk in data.chunks(2048) { black_box(resampler.process(black_box(chunk))); } }
    start.elapsed().as_secs_f64() * 1000.0
}

fn time_after(data: &[f32]) -> f64 {
    let start = Instant::now();
    let mut resampler = optimized::Resampler::new(44100, 48000, 2).unwrap();
    for _ in 0..12 { for chunk in data.chunks(2048) { black_box(resampler.process(black_box(chunk))); } }
    start.elapsed().as_secs_f64() * 1000.0
}

fn main() {
    for channels in [1, 2, 4] {
        for (from, to) in [(44100, 48000), (48000, 44100), (44100, 96000)] {
            let data = input(channels);
            let mut before = baseline::Resampler::new(from, to, channels).unwrap();
            let mut after = optimized::Resampler::new(from, to, channels).unwrap();
            for chunk in data.chunks(1024 * channels) {
                let expected = before.process(chunk);
                let actual = after.process(chunk);
                assert_eq!(expected.len(), actual.len());
                for (a, b) in expected.iter().zip(&actual) { assert_eq!(a.to_bits(), b.to_bits(), "audio sample changed"); }
            }
            println!("bit-exact channels={channels} rates={from}/{to}");
        }
    }
    let data = input(2);
    black_box(time_before(&data));
    black_box(time_after(&data));
    let mut before = Vec::new();
    let mut after = Vec::new();
    for round in 0..9 {
        let (a, b) = if round % 2 == 0 { (time_before(&data), time_after(&data)) }
            else { let b = time_after(&data); (time_before(&data), b) };
        before.push(a); after.push(b);
        println!("round={round} baseline_ms={a:.3} optimized_ms={b:.3}");
    }
    before.sort_by(f64::total_cmp); after.sort_by(f64::total_cmp);
    let (a, b) = (before[4], after[4]);
    println!("median baseline_ms={a:.3} optimized_ms={b:.3} reduction_percent={:.2}", (1.0 - b / a) * 100.0);
}
`);
execFileSync('rustc', ['--edition=2021', '-C', 'opt-level=3', source, '-o', executable], { cwd: root, stdio: 'inherit' });
execFileSync(executable, [], { cwd: root, stdio: 'inherit' });
console.log(`Benchmark source and binary: ${directory}`);
