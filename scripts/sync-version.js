// Syncs the version from package.json → Cargo.toml + tauri.conf.json
import { readFileSync, writeFileSync } from 'fs';

export function syncVersion(version) {
  // Update Cargo.toml
  const cargoPath = './src-tauri/Cargo.toml';
  let cargo = readFileSync(cargoPath, 'utf-8');
  cargo = cargo.replace(/^version\s*=\s*".*"/m, `version = "${version}"`);
  writeFileSync(cargoPath, cargo);

  // Update tauri.conf.json
  const tauriPath = './src-tauri/tauri.conf.json';
  let tauri;
  try {
    tauri = JSON.parse(readFileSync(tauriPath, 'utf-8'));
  } catch (e) {
    throw new Error(`Failed to parse tauri.conf.json: ${e.message}`);
  }
  if (tauri.version !== version) {
    tauri.version = version;
    writeFileSync(tauriPath, JSON.stringify(tauri, null, 2) + '\n');
  }
}

// CLI entrypoint: runs when invoked directly via `node scripts/sync-version.js`
import { fileURLToPath } from 'url';
import { resolve } from 'path';
if (process.argv[1] && resolve(fileURLToPath(import.meta.url)) === resolve(process.argv[1])) {
  let pkg;
  try {
    pkg = JSON.parse(readFileSync('./package.json', 'utf-8'));
  } catch (e) {
    console.error(`Failed to read package.json: ${e.message}`);
    process.exit(1);
  }
  syncVersion(pkg.version);
  console.log(`Synced version ${pkg.version} → Cargo.toml, tauri.conf.json`);
}
