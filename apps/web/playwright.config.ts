import { defineConfig } from '@playwright/test';

// Dedicated port, never reuse a stray server: a smoke test must hit THIS build.
const port = 4391;

export default defineConfig({
	testDir: 'tests',
	webServer: { command: `vite preview --port ${port} --strictPort`, port, reuseExistingServer: false },
	use: { baseURL: `http://localhost:${port}` }
});
