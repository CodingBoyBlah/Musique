// node scripts/benchmark-time-stretch.mjs [baseline git ref]
// Requires rustc. Compares actual production code without Cargo dependencies.
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const baseline = process.argv[2] ?? "b98500679e4e0b2549987b965af432ccd583dc26";
const directory = fs.mkdtempSync(path.join(os.tmpdir(), "musique-stretch-"));
const oldSource = path.join(directory, "baseline.rs");
const source = path.join(directory, "benchmark.rs");
const executable = path.join(directory, process.platform === "win32" ? "benchmark.exe" : "benchmark");
const rustPath = value => JSON.stringify(value.replaceAll("\\", "/"));
fs.writeFileSync(oldSource, execFileSync("git", ["show", `${baseline}:src-tauri/src/stretch.rs`], { cwd: root }));
fs.writeFileSync(source, `
#![allow(dead_code)]
#[path = ${rustPath(oldSource)}]
mod baseline;
#[path = ${rustPath(path.join(root, "src-tauri/src/stretch.rs"))}]
mod optimized;
use std::{hint::black_box, time::Instant};

fn input(kind: usize) -> Vec<f32> {
    let mut seed = 7u32;
    (0..44100 * 2).map(|i| match kind {
        0 => 0.0,
        1 => (i as f32 * 440.0 * std::f32::consts::PI / 44100.0).sin() * 0.5,
        _ => { seed = seed.wrapping_mul(1664525).wrapping_add(1013904223); seed as i32 as f32 / i32::MAX as f32 }
    }).collect()
}

fn verify() {
    let mut comparisons = 0;
    for kind in 0..3 {
        let data = input(kind);
        for speed in [0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0] {
            for chunk_samples in [2, 14, 254, 2048, 8192, 88200] {
                let mut before = baseline::TimeStretch::new(speed);
                let mut after = optimized::TimeStretch::new(speed);
                for chunk in data.chunks(chunk_samples) {
                    let expected = before.process(chunk);
                    let actual = after.process(chunk);
                    assert_eq!(expected.len(), actual.len(), "kind={kind} speed={speed} chunk={chunk_samples}");
                    for (a, b) in expected.iter().zip(&actual) {
                        assert_eq!(a.to_bits(), b.to_bits(), "audio sample changed, kind={kind} speed={speed} chunk={chunk_samples}");
                    }
                }
                comparisons += 1;
            }
        }
    }
    println!("bit-exact time-stretch combinations: {comparisons}");
}
fn time_before(data: &[f32]) -> f64 {
    let start = Instant::now();
    let mut s = baseline::TimeStretch::new(1.5);
    for _ in 0..12 { for chunk in data.chunks(2048) { black_box(s.process(black_box(chunk))); } }
    start.elapsed().as_secs_f64() * 1000.0
}
fn time_after(data: &[f32]) -> f64 {
    let start = Instant::now();
    let mut s = optimized::TimeStretch::new(1.5);
    for _ in 0..12 { for chunk in data.chunks(2048) { black_box(s.process(black_box(chunk))); } }
    start.elapsed().as_secs_f64() * 1000.0
}
fn main() {
    verify();
    let data = input(2);
    black_box(time_before(&data)); black_box(time_after(&data));
    let mut before = Vec::new(); let mut after = Vec::new();
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
execFileSync("rustc", ["--edition=2021", "-C", "opt-level=3", source, "-o", executable], { cwd: root, stdio: "inherit" });
execFileSync(executable, [], { cwd: root, stdio: "inherit" });
console.log(`Benchmark source and binary: ${directory}`);
