import { readFile, appendFile } from "node:fs/promises";
import { randomUUID } from "node:crypto";

const read = (path) => readFile(path, "utf8");
const version = JSON.parse(await read("package.json")).version;
if (!/^\d+\.\d+\.\d+$/.test(version)) {
  throw new Error(`Invalid release version: ${version}`);
}

const versions = {
  "src-tauri/tauri.conf.json": JSON.parse(await read("src-tauri/tauri.conf.json")).version,
  "src-tauri/Cargo.toml": (await read("src-tauri/Cargo.toml"))
    .match(/^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m)?.[1],
  "src-tauri/Cargo.lock": (await read("src-tauri/Cargo.lock"))
    .match(/^name = "spotify"\r?\nversion = "([^"]+)"/m)?.[1],
};
for (const [path, actual] of Object.entries(versions)) {
  if (actual !== version) {
    throw new Error(`${path} has version ${actual}; expected ${version}`);
  }
}

const tag = process.env.RELEASE_TAG || `v${version}`;
if (tag !== `v${version}`) {
  throw new Error(`Release tag ${tag} does not match app version ${version}`);
}
const notesTag = tag === "v1.3.1" ? "v1.3.0" : tag;
const path = `changelogs/${notesTag}.md`;
const body = (await read(path)).trim();
if (!body) throw new Error(`Release notes are empty: ${path}`);

if (process.env.GITHUB_OUTPUT) {
  const delimiter = `notes_${randomUUID()}`;
  await appendFile(process.env.GITHUB_OUTPUT,
    `tag=${tag}\nbody<<${delimiter}\n${body}\n${delimiter}\n`);
}
console.log(`Loaded release notes for ${tag} from ${path}`);
