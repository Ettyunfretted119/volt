import { defineConfig } from 'vite';
import { cpSync, readFileSync, writeFileSync } from 'fs';

// Sync version from package.json → Cargo.toml + tauri.conf.json
const pkg = JSON.parse(readFileSync('./package.json', 'utf-8'));
const syncVersion = (v) => {
  const cargoPath = './src-tauri/Cargo.toml';
  let cargo = readFileSync(cargoPath, 'utf-8');
  cargo = cargo.replace(/^version\s*=\s*".*"/m, `version = "${v}"`);
  writeFileSync(cargoPath, cargo);
  const tauriPath = './src-tauri/tauri.conf.json';
  const tauri = JSON.parse(readFileSync(tauriPath, 'utf-8'));
  if (tauri.version !== v) { tauri.version = v; writeFileSync(tauriPath, JSON.stringify(tauri, null, 2) + '\n'); }
};
syncVersion(pkg.version);

export default defineConfig({
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  plugins: [{
    name: 'copy-material-icons',
    writeBundle() {
      cpSync(
        'node_modules/material-icon-theme/icons',
        'dist/material-icons',
        { recursive: true }
      );
    },
  }],
});
