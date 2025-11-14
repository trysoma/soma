import { defineConfig } from 'vitest/config'

export default defineConfig({
	test: {
		globals: true,
		environment: 'node',
		passWithNoTests: true,
		coverage: {
			provider: 'v8',
			reporter: ['text', 'json', 'html', 'lcov'],
			reportsDirectory: './coverage',
			include: [
				'src/**/*.ts',
				'src/**/*.tsx',
				'src/**/*.js',
				'src/**/*.jsx',
				'*.ts',
				'*.tsx',
				'*.js',
				'*.jsx',
			],
			exclude: [
				'**/*.test.ts',
				'**/*.test.tsx',
				'**/*.spec.ts',
				'**/*.spec.tsx',
				'**/*.d.ts',
				'**/node_modules/**',
				'**/dist/**',
				'**/.{git,cache}/**',
				'**/vitest.config.*',
				'**/vite.config.*',
				'**/*.config.*',
			],
		},
	},
})
