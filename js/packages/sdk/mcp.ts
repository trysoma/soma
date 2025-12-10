/**
 * Durable MCP (Model Context Protocol) middleware for Restate.
 * Provides a middleware that makes all MCP fetch operations replayable via Restate.
 */

import { Client } from "@modelcontextprotocol/sdk/client";
import { applyMiddlewares, type Middleware } from "@modelcontextprotocol/sdk/client/middleware.js";
import { StreamableHTTPClientTransport } from "@modelcontextprotocol/sdk/client/streamableHttp.js";
import { Transport } from "@modelcontextprotocol/sdk/shared/transport.js";
import type { Context, RunOptions, Serde } from "@restatedev/restate-sdk";

/**
 * Internal structure for serializing Response objects.
 */
interface SerializedResponse {
	status: number;
	statusText: string;
	headers: Record<string, string>;
	body: string;
}

/**
 * Serde for serialized response objects.
 */
class SerializedResponseSerde implements Serde<SerializedResponse> {
	contentType = "application/json";

	serialize(value: SerializedResponse): Uint8Array {
		return new TextEncoder().encode(JSON.stringify(value));
	}

	deserialize(data: Uint8Array): SerializedResponse {
		return JSON.parse(new TextDecoder().decode(data));
	}
}

const serializedResponseSerde = new SerializedResponseSerde();

/**
 * Options for the durable MCP middleware.
 */
export interface DurableMCPOptions {
	/**
	 * Optional prefix for the durable run names.
	 * Useful for distinguishing between different MCP server connections.
	 * @default "mcp"
	 */
	prefix?: string;

	/**
	 * Optional run options to pass to ctx.run().
	 */
	runOptions?: Omit<RunOptions<SerializedResponse>, "serde">;
}

/**
 * Creates a middleware that provides durability to MCP fetch operations via Restate.
 *
 * This middleware wraps each fetch call with `ctx.run()`, making the operation
 * replayable and durable. The response is serialized and stored, allowing
 * Restate to replay the operation if needed.
 *
 * @param ctx - The Restate context used to capture the execution of fetch operations.
 * @param opts - Optional configuration for the middleware.
 * @returns A Middleware function that can be passed to MCP client transports.
 *
 * @example
 * ```typescript
 * import { durableMCP } from '@trysoma/sdk/mcp';
 * import { Client } from '@modelcontextprotocol/sdk/client';
 * import { StreamableHTTPClientTransport } from '@modelcontextprotocol/sdk/client/streamableHttp';
 * import { applyMiddlewares } from '@modelcontextprotocol/sdk/client/middleware';
 *
 * // In a Restate handler
 * const handler = async (ctx: ObjectContext) => {
 *   // Create the durable middleware
 *   const durableMiddleware = durableMCP(ctx, { prefix: 'my-mcp-server' });
 *
 *   // Apply middleware to fetch
 *   const durableFetch = applyMiddlewares(durableMiddleware)(fetch);
 *
 *   // Create transport with durable fetch
 *   const transport = new StreamableHTTPClientTransport(
 *     new URL('http://localhost:3000/mcp'),
 *     { fetch: durableFetch }
 *   );
 *
 *   // Create client and connect
 *   const client = new Client({ name: 'my-client', version: '1.0.0' });
 *   await client.connect(transport);
 *
 *   // All MCP operations are now durable
 *   const tools = await client.listTools();
 *   const result = await client.callTool({ name: 'my-tool', arguments: {} });
 * };
 * ```
 */
export const durableMCP = (
	ctx: Context,
	opts?: DurableMCPOptions,
): Middleware => {
	const prefix = opts?.prefix ?? "mcp";
	let callIndex = 0;

	return (next) => async (input, init) => {
		const currentIndex = callIndex++;
		const method = init?.method || "GET";
		const url = typeof input === "string" ? input : input.toString();

		// Check if this is requesting an SSE stream (Accept: text/event-stream)
		const acceptHeader = init?.headers instanceof Headers
			? init.headers.get("accept")
			: typeof init?.headers === "object" && init.headers !== null
				? (init.headers as Record<string, string>)["accept"] || (init.headers as Record<string, string>)["Accept"]
				: undefined;

		const isSSERequest = acceptHeader?.includes("text/event-stream");

		// For SSE streaming requests, we need to handle the stream specially
		if (isSSERequest) {
			const runName = `${prefix}-fetch-${currentIndex}-${method}-${url.split("/").pop() || "request"}`;

			// Make the HTTP request (this could be wrapped in ctx.run for durability)
			const response = await next(input, init);

			// Check if we got an SSE response
			const contentType = response.headers.get("content-type");
			if (!contentType?.includes("text/event-stream")) {
				// Not actually SSE, return as-is
				return response;
			}

			// Create a TransformStream to process SSE events
			const { readable, writable } = new TransformStream<Uint8Array, Uint8Array>();

			// Process the SSE stream in the background
			(async () => {
				const reader = response.body?.getReader();
				const writer = writable.getWriter();
				const decoder = new TextDecoder();

				if (!reader) {
					await writer.close();
					return;
				}

				let buffer = "";

				try {
					while (true) {
						const { done, value } = await reader.read();

						if (done) {
							break;
						}

						// Decode the chunk and add to buffer
						buffer += decoder.decode(value, { stream: true });

						// Process complete events (events are separated by double newlines)
						const events = buffer.split("\n\n");
						// Keep the last incomplete event in buffer
						buffer = events.pop() || "";

						for (const event of events) {
							if (event.trim()) {
								// TODO: This is where you can make each event durable
								// e.g., await ctx.run(`${runName}-event-${eventIndex}`, async () => event);
								ctx.console.log(`SSE event: ${event.substring(0, 100)}...`);

								// Write the event back to the stream (with double newline delimiter)
								const encoded = new TextEncoder().encode(event + "\n\n");
								await writer.write(encoded);
							}
						}
					}

					// Write any remaining buffer content
					if (buffer.trim()) {
						const encoded = new TextEncoder().encode(buffer);
						await writer.write(encoded);
					}
				} catch (error) {
					ctx.console.error(`SSE stream error: ${error}`);
				} finally {
					await writer.close();
				}
			})();

			// Return a new response with the transformed stream
			return new Response(readable, {
				status: response.status,
				statusText: response.statusText,
				headers: response.headers,
			});
		}

		// Create a unique name for this operation
		const runName = `${prefix}-fetch-${currentIndex}-${method}-${url.split("/").pop() || "request"}`;

		// Execute the fetch durably (only for non-streaming requests)
		const serializedResponse = await ctx.run(
			runName,
			async () => {
				const response = await next(input, init);

				// Serialize the response for storage
				const body = await response.text();
				const headers: Record<string, string> = {};
				response.headers.forEach((value, key) => {
					headers[key] = value;
				});

				return {
					status: response.status,
					statusText: response.statusText,
					headers,
					body,
				};
			},
			{
				...opts?.runOptions,
				serde: serializedResponseSerde,
			},
		);

		// Reconstruct the Response object
		return new Response(serializedResponse.body, {
			status: serializedResponse.status,
			statusText: serializedResponse.statusText,
			headers: serializedResponse.headers,
		});
	};
};

/**
 * Configuration options for creating a Soma MCP client.
 */
export interface SomaMcpClientConfig {
	/**
	 * The base URL of the Soma server.
	 * Defaults to SOMA_SERVER_BASE_URL env var or 'http://localhost:3000'.
	 */
	SOMA_BASE_URL?: string;
}

/**
 * Creates an MCP client connected to a Soma MCP server instance.
 *
 * @param mcpServerInstanceId - The ID of the MCP server instance to connect to.
 * @param config - Optional configuration including base URL.
 * @returns A connected MCP Client instance.
 *
 * @example
 * ```typescript
 * import { createSomaMcpClient } from '@trysoma/sdk/mcp';
 *
 * const client = await createSomaMcpClient('my-mcp-instance-id');
 *
 * // List available tools
 * const tools = await client.listTools();
 *
 * // Call a tool
 * const result = await client.callTool({ name: 'my-tool', arguments: {} });
 * ```
 */
export async function createSomaMcpClient(
	ctx: Context,
	mcpServerInstanceId: string,
	config?: SomaMcpClientConfig,
): Promise<Client> {
	
	
	const client = new Client({
		name: mcpServerInstanceId,
		version: "1.0.0",
	});

	let {
		transport,
		sessionId,
	} = createSomaMcpTransport(ctx, mcpServerInstanceId, config);
	await client.connect(transport);
	sessionId = transport.sessionId;
	return client;
}


export function createSomaMcpTransport(ctx: Context, mcpServerInstanceId: string, config?: SomaMcpClientConfig): { transport: StreamableHTTPClientTransport, sessionId: string | undefined } {	
	const baseUrl =
	config?.SOMA_BASE_URL ||
	// biome-ignore lint/complexity/useLiteralKeys: TypeScript requires bracket notation for process.env
		process.env["SOMA_SERVER_BASE_URL"] ||
		"http://localhost:3000";

	const mcpUrl = new URL(
		`/api/bridge/v1/mcp-instance/${mcpServerInstanceId}/mcp`,
		baseUrl,
	);

	const durableMiddleware = durableMCP(ctx, { prefix: mcpServerInstanceId });
	const durableFetch = applyMiddlewares(durableMiddleware)(fetch);
	let sessionId: string | undefined = undefined;
	const transport = new StreamableHTTPClientTransport(mcpUrl, { fetch: durableFetch, sessionId });

	return {
		transport,
		sessionId,
	};
}