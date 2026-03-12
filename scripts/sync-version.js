// Syncs the version from package.json → Cargo.toml + tauri.conf.json
import { readFileSync, writeFileSync } from 'fs';

const pkg = JSON.parse(readFileSync('./package.json', 'utf-8'));
const version = pkg.version;

// Update Cargo.toml
const cargoPath = './src-tauri/Cargo.toml';
let cargo = readFileSync(cargoPath, 'utf-8');
cargo = cargo.replace(/^version\s*=\s*".*"/m, `version = "${version}"`);
writeFileSync(cargoPath, cargo);

// Update tauri.conf.json
const tauriPath = './src-tauri/tauri.conf.json';
const tauri = JSON.parse(readFileSync(tauriPath, 'utf-8'));
tauri.version = version;
writeFileSync(tauriPath, JSON.stringify(tauri, null, 2) + '\n');

console.log(`Synced version ${version} → Cargo.toml, tauri.conf.json`);
