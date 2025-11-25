// plugins/routes-json.ts

import fs from "node:fs";
import path from "node:path";
import type { OutputBundle } from "rollup";
import type { Plugin } from "vite";

function toAxumPath(routePath: string): string {
	return routePath.replace(/\$([A-Za-z0-9_]+)/g, "{$1}");
}

const OUTPUT_FILE_NAME = ".vite-rs/routes.json";

function getRoutes(): string[] {
	// biome-ignore lint/suspicious/noExplicitAny: globalThis doesn't have type definitions for TSR_ROUTES_BY_ID_MAP
	const routes = (globalThis as any).TSR_ROUTES_BY_ID_MAP as Map<
		string,
		{ routePath: string }
	>;

	if (!routes) return [];

	return Array.from(routes.values())
		.filter((p) => !p.routePath.includes("__"))
		.map(({ routePath }) => toAxumPath(routePath));
}

function makeJson(paths: string[], assets: string[] = []): string {
	return JSON.stringify({ paths, assets }, null, 2);
}

function walkDir(dir: string, baseDir: string): string[] {
	const out: string[] = [];
	for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
		const abs = path.join(dir, entry.name);
		const rel = path.relative(baseDir, abs).replace(/\\/g, "/");
		if (entry.isDirectory()) {
			out.push(...walkDir(abs, baseDir));
		} else {
			out.push(rel);
		}
	}
	return out;
}

export function RoutesJsonPlugin(): Plugin {
	let outDir: string;

	return {
		name: "routes-json",

		configResolved(config) {
			outDir = config.build.outDir || "dist";
		},

		buildStart() {
			this.addWatchFile(path.resolve("src/routes"));
		},

		handleHotUpdate(ctx) {
			if (ctx.file.includes("/routes/")) {
				const paths = getRoutes();
				const source = makeJson(paths);

				const outPath = path.resolve(
					ctx.server.config.root,
					outDir,
					OUTPUT_FILE_NAME,
				);
				fs.mkdirSync(path.dirname(outPath), { recursive: true });
				fs.writeFileSync(outPath, source, "utf-8");
			}
		},

		generateBundle(_options, bundle: OutputBundle) {
			// Still emit into Rollup’s bundle (in-memory) for consistency
			const paths = getRoutes();
			const assets = Object.keys(bundle).filter((f) => !f.endsWith(".map"));
			const source = makeJson(paths, assets);

			this.emitFile({
				type: "asset",
				fileName: OUTPUT_FILE_NAME,
				source,
			});
		},

		closeBundle() {
			// Final pass: scan the entire dist directory
			const distPath = path.resolve(process.cwd(), outDir);
			if (!fs.existsSync(distPath)) return;

			const paths = getRoutes();
			const allFiles = walkDir(distPath, distPath);
			const source = makeJson(paths, allFiles);

			const outPath = path.join(distPath, OUTPUT_FILE_NAME);
			fs.mkdirSync(path.dirname(outPath), { recursive: true });
			fs.writeFileSync(outPath, source, "utf-8");
		},
	};
}
