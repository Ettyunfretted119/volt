import { defineConfig } from 'vite';
import { cpSync, readFileSync } from 'fs';

const pkg = JSON.parse(readFileSync('./package.json', 'utf-8'));

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
