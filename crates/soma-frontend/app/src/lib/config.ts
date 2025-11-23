import { useMemo } from "react";
import type { components } from "@/@types/openapi";
import $api from "@/lib/api-client";

let runtimeConfig!: components["schemas"]["RuntimeConfig"];

export async function loadConfig() {
	if (runtimeConfig) return runtimeConfig;

	try {
		const res = await $api.GET("/_internal/v1/runtime_config");
		if (res.error) throw new Error(`Failed to fetch config`);
		runtimeConfig = res.data;
		(window as any).__RUNTIME_CONFIG__ = runtimeConfig;
		return runtimeConfig;
	} catch (err) {
		console.error("Failed to fetch config", err);
		throw err;
	}
}

export function getConfig() {
	if (!runtimeConfig) {
		throw new Error("Runtime config not loaded yet. Call loadConfig() first.");
	}
	return runtimeConfig;
}

export function useConfig() {
	const config = useMemo(() => getConfig(), []);
	return config;
}
