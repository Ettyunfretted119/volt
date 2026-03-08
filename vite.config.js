import { defineConfig } from 'vite';
import { cpSync } from 'fs';

export default defineConfig({
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
