import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import path from 'path';

export default defineConfig({
	plugins: [
		svelte({
			compilerOptions: {
				runes: ({ filename }) =>
					filename?.split(/[/\\]/).includes('node_modules') ? undefined : true
			}
		})
	],
	resolve: {
		alias: {
			$lib: path.resolve('./src/lib')
		},
		conditions: ['browser']
	},
	test: {
		environment: 'jsdom',
		include: ['src/**/*.test.ts'],
		setupFiles: []
	}
});
