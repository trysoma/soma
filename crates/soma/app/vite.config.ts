import { resolve } from "node:path";
import tailwindcss from "@tailwindcss/vite";
import { TanStackRouterVite } from "@tanstack/router-plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { defineConfig as defineViteConfig } from "vite";
import { mergeConfig } from "vitest/config";
import baseConfig from "../../../vitest.config.base";
import { RoutesJsonPlugin } from "./vite.axum";

const viteConfig = defineViteConfig({
	plugins: [
		TanStackRouterVite({ autoCodeSplitting: true }),
		viteReact(),
		tailwindcss(),
		RoutesJsonPlugin(),
	],
	resolve: {
		alias: {
			"@": resolve(__dirname, "./src"),
		},
	},
	server: {
		// port: 3000,
		// host: '127.0.0.1',
		hmr: {
			port: 21012,
			// clientPort: process.env.HMR_PORT ? parseInt(process.env.HMR_PORT) : 21013,
		},
		allowedHosts: ["localhost", "127.0.0.1"],
	},
	build: {
		target: "esnext",
	},
});
// https://vitejs.dev/config/
export default mergeConfig(
	baseConfig,
	viteConfig,
	
);
