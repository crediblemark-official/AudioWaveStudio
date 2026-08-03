import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

// Standalone config for the export-parity harness (tests/compareExport.test.ts).
// The harness lives outside src/ and needs node built-ins, so it is kept out
// of the default suite (vitest.config.ts includes src/**/*.test.ts).
export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'node',
    include: ['tests/**/*.test.ts'],
  },
});
