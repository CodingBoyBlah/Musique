// node scripts/benchmark-lyrics.mjs [baseline git ref]
// Requires rustc. Compiles the actual alignment code and shared lyric types.
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const baseline = process.argv[2] ?? "b98500679e4e0b2549987b965af432ccd583dc26";
const directory = fs.mkdtempSync(path.join(os.tmpdir(), "musique-lyrics-"));
const oldSource = path.join(directory, "baseline.rs");
const typeSource = path.join(directory, "types.rs");
const source = path.join(directory, "benchmark.rs");
const executable = path.join(directory, process.platform === "win32" ? "benchmark.exe" : "benchmark");
const rustPath = value => JSON.stringify(value.replaceAll("\\", "/"));
fs.writeFileSync(oldSource, execFileSync("git", ["show", `${baseline}:src-tauri/src/lyrics/align.rs`], { cwd: root }));
// Serde affects IPC serialization, not the alignment algorithm. Remove only
// its derives and attributes so this benchmark needs no Cargo dependencies.
const types = fs.readFileSync(path.join(root, "src-tauri/src/lyrics/types.rs"), "utf8")
  .replace(/\r\n/g, "\n")
  .replace(/^use serde::\{Deserialize, Serialize\};\r?\n/gm, "")
  .replace(/,\s*(?:Serialize|Deserialize)/g, "")
  .replace(/^[ \t]*#\[serde\([^\n]*\)\]\r?\n/gm, "");
fs.writeFileSync(typeSource, types);
fs.writeFileSync(source, `
#![allow(dead_code)]
#[path = ${rustPath(typeSource)}]
mod types;
#[path = ${rustPath(oldSource)}]
mod baseline;
#[path = ${rustPath(path.join(root, "src-tauri/src/lyrics/align.rs"))}]
mod optimized;
use std::{hint::black_box, time::Instant};
use types::{Candidate, LyricLine, LyricWord};

fn document(n: usize, variant: usize, offset: i64) -> Candidate {
    let texts = ["Don't stop me now!", "HELLO, world (ad-lib)", "今日は晴れ", "İSTANBUL forever", "Привет мир", "한글 가사", "♪", "", "chorus alpha", "chorus alphabet"];
    let mut lines = Vec::new();
    for i in 0..n {
        let text = if variant % 7 == 0 { texts[(i + variant) % texts.len()].to_string() }
            else { format!("{} line {}", texts[(i + variant) % texts.len()], i) };
        let time = 10_000 + i as i64 * 2500 + offset;
        let line = if (i + variant) % 3 == 0 {
            LyricLine::worded(time, Some(text.clone()), vec![LyricWord {time_ms: time, end_ms: time + 500, text}])
        } else { LyricLine::line(time, text) };
        lines.push(line);
    }
    let mut c = Candidate::new(if variant % 2 == 0 { "amll" } else { "musixmatch" }, lines);
    c.exact = variant % 4 == 0;
    c
}

fn verify() {
    let mut comparisons = 0;
    for variant in 0..80 {
        for count in [0, 1, 3, 4, 10, 40, 120, 400] {
            let reference = document(count, variant, 0);
            let mut c = document(count, variant, (variant as i64 - 40) * 300);
            if variant % 3 == 0 { c.lines.retain(|l| l.time_ms.rem_euclid(3) != 0); }
            if variant % 5 == 0 { c.lines.reverse(); }
            if variant % 11 == 0 {
                for (i, line) in c.lines.iter_mut().enumerate() { line.time_ms += (i as i64 * 7919 % 5001) - 2500; }
            }
            let before = baseline::align_to_reference(&c, &reference);
            let after = optimized::align_to_reference(&c, &reference);
            assert_eq!((before.offset_ms, before.residual_ms, before.matched, before.aligned),
                (after.offset_ms, after.residual_ms, after.matched, after.aligned), "variant={variant} lines={count}");
            let before = baseline::cross_agrees(&c, &reference);
            let after = optimized::cross_agrees(&c, &reference);
            assert_eq!((before.offset_ms, before.residual_ms, before.matched, before.aligned),
                (after.offset_ms, after.residual_ms, after.matched, after.aligned));
            let other = document(count, variant + 1, 100);
            for reference in [None, Some(&reference)] {
                let before = baseline::choose(reference, vec![c.clone(), other.clone()]);
                let after = optimized::choose(reference, vec![c.clone(), other.clone()]);
                assert_eq!(format!("{before:?}"), format!("{after:?}"), "selection changed");
            }
            comparisons += 4;
        }
    }
    println!("equivalent alignment and selection results: {comparisons} comparisons");
}

fn time_before(c: &Candidate, r: &Candidate) -> f64 {
    let start = Instant::now();
    for _ in 0..100 { black_box(baseline::align_to_reference(black_box(c), black_box(r))); }
    start.elapsed().as_secs_f64() * 1000.0
}
fn time_after(c: &Candidate, r: &Candidate) -> f64 {
    let start = Instant::now();
    for _ in 0..100 { black_box(optimized::align_to_reference(black_box(c), black_box(r))); }
    start.elapsed().as_secs_f64() * 1000.0
}
fn main() {
    verify();
    let reference = document(120, 1, 0);
    let c = document(120, 1, 450);
    black_box(time_before(&c, &reference)); black_box(time_after(&c, &reference));
    let mut before = Vec::new(); let mut after = Vec::new();
    for round in 0..9 {
        let (a, b) = if round % 2 == 0 { (time_before(&c, &reference), time_after(&c, &reference)) }
            else { let b = time_after(&c, &reference); (time_before(&c, &reference), b) };
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
