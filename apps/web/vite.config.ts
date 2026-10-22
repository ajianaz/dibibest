import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	// SPA fallback: unknown paths (e.g. /app on a static host) serve the client router.
	plugins: [sveltekit({ adapter: adapter({ fallback: '200.html' }) })]
});
