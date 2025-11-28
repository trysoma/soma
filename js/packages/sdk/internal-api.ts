/**
 * Internal API utilities for SDK-to-API-server communication
 */

export interface ResyncSdkResponse {
	message: string;
	providers_synced: number;
	agents_synced: number;
	secrets_synced: number;
	env_vars_synced: number;
}

/**
 * Calls the internal resync endpoint on the Soma API server.
 * This triggers the API server to:
 * - Fetch metadata from the SDK (providers, agents)
 * - Sync providers to the bridge registry
 * - Register Restate deployments for agents
 * - Sync secrets to the SDK
 * - Sync environment variables to the SDK
 *
 * @param baseUrl - The base URL of the Soma API server (defaults to SOMA_SERVER_BASE_URL env var or http://localhost:3000)
 * @returns The resync response from the server
 */
export async function resyncSdk(baseUrl?: string): Promise<ResyncSdkResponse> {
	const apiBaseUrl =
		baseUrl || process.env["SOMA_SERVER_BASE_URL"] || "http://localhost:3000";
	const url = `${apiBaseUrl}/_internal/v1/resync_sdk`;

	console.log(`[SDK] Calling resync endpoint: ${url}`);

	try {
		const response = await fetch(url, {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
			},
		});

		if (!response.ok) {
			const errorText = await response.text();
			throw new Error(
				`Resync failed with status ${response.status}: ${errorText}`,
			);
		}

		const result: ResyncSdkResponse = await response.json();
		console.log(
			`[SDK] Resync complete: ${result.providers_synced} providers, ${result.agents_synced} agents, ${result.secrets_synced} secrets, ${result.env_vars_synced} env vars`,
		);

		return result;
	} catch (error) {
		const message = error instanceof Error ? error.message : String(error);
		console.error(`[SDK] Resync failed: ${message}`);
		throw error;
	}
}
