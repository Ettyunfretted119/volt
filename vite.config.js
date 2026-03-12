import { defineConfig } from 'vite';
import { cpSync, readFileSync } from 'fs';
import { syncVersion } from './scripts/sync-version.js';

// Sync version from package.json → Cargo.toml + tauri.conf.json
let pkg;
try {
  pkg = JSON.parse(readFileSync('./package.json', 'utf-8'));
} catch (e) {
  throw new Error(`Failed to read package.json: ${e.message}`);
}
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
      try {
        cpSync(
          'node_modules/material-icon-theme/icons',
          'dist/material-icons',
          { recursive: true }
        );
      } catch (e) {
        console.warn('Warning: could not copy material icons:', e.message);
      }
    },
  }],
});
