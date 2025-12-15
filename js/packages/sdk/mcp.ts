/**
 * Durable MCP (Model Context Protocol) middleware for Restate.
 * Provides a middleware that makes all MCP fetch operations replayable via Restate.
 */

import { Client, type ClientOptions } from "@modelcontextprotocol/sdk/client";
import { StreamableHTTPClientTransport } from "@modelcontextprotocol/sdk/client/streamableHttp.js";
import type { RequestOptions } from "@modelcontextprotocol/sdk/shared/protocol.js";
import type { Transport } from "@modelcontextprotocol/sdk/shared/transport.js";
import type {
	CallToolRequest,
	CallToolResultSchema,
	CompatibilityCallToolResultSchema,
	GetPromptRequest,
	Implementation,
	ListPromptsRequest,
	ListResourcesRequest,
	ListResourceTemplatesRequest,
	ListToolsRequest,
	Notification,
	ReadResourceRequest,
	Result,
	SubscribeRequest,
	UnsubscribeRequest,
} from "@modelcontextprotocol/sdk/types.js";
import type { Context, ObjectContext } from "@restatedev/restate-sdk";

/**
 * Configuration options for creating a Soma MCP client.
 */
export interface SomaMcpClientConfig {
	/**
	 * The base URL of the Soma server.
	 * Defaults to SOMA_SERVER_BASE_URL env var or 'http://localhost:3000'.
	 */
	somaBaseUrl?: string;
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
export async function createSomaMcpClient<RequestT extends Request = Request, NotificationT extends Notification = Notification, ResultT extends Result = Result>(
	ctx: ObjectContext,
	mcpServerInstanceId: string,
	config?: SomaMcpClientConfig,
): Promise<SomaMcpClient<RequestT, NotificationT, ResultT>> {


	const client = new SomaMcpClient<RequestT, NotificationT, ResultT>(
		ctx,
		{
			name: mcpServerInstanceId,
			version: "1.0.0",
		},
	);

	const { transport, sessionId: _sessionId } = createSomaMcpTransport(
		ctx,
		mcpServerInstanceId,
		config,
	);

	// Set up error handler before connecting (matches frontend pattern)
	client.onerror = (error) => {
		ctx.console.error("MCP client error:", error);
	};

	// Set up close handler before connecting (matches frontend pattern)
	client.onclose = () => {
		ctx.console.debug("MCP client connection closed");
	};

	// Create fresh connection - don't wrap in ctx.run to avoid session resumption issues
	// The connection is ephemeral and will be recreated on each invocation/replay
	// If connection fails due to session resumption, the error handler will catch it
	try {
		await client.connect(transport);
	} catch (error) {
		// If connection fails (e.g., due to session resumption issues), log and rethrow
		// The error handler above will also catch it
		ctx.console.error("Failed to connect MCP client:", error);
		throw error;
	}
	return client;
}

export function createSomaMcpTransport(
	_ctx: Context,
	mcpServerInstanceId: string,
	config?: SomaMcpClientConfig,
): { transport: StreamableHTTPClientTransport; sessionId: string | undefined } {
	const baseUrl =
		config?.somaBaseUrl ||
		// biome-ignore lint/complexity/useLiteralKeys: TypeScript requires bracket notation for process.env
		process.env["SOMA_SERVER_BASE_URL"] ||
		"http://localhost:3000";

	const mcpUrl = new URL(
		`/api/bridge/v1/mcp-instance/${mcpServerInstanceId}/mcp`,
		baseUrl,
	);

	let sessionId: string | undefined;
	const transport = new StreamableHTTPClientTransport(mcpUrl);

	return {
		transport,
		sessionId,
	};
}

export class SomaMcpClient<
	RequestT extends Request = Request,
	NotificationT extends Notification = Notification,
	ResultT extends Result = Result,
> extends Client<RequestT, NotificationT, ResultT> {
	private restate: ObjectContext;
	private clientInfo: Implementation;

	public constructor(
		restate: ObjectContext,
		clientInfo: Implementation,
		options?: ClientOptions,
	) {
		super(clientInfo, options);
		this.restate = restate;
		this.clientInfo = clientInfo;
	}

	override async connect(
		transport: Transport,
		options?: RequestOptions,
	): Promise<void> {
		return this.restate.run(`mcp-${this.clientInfo.name}-connect`, async () => {
			return super.connect(transport, options);
		});
	}

	override async ping(options?: RequestOptions): Promise<{
		_meta?: Record<string, unknown> | undefined;
	}> {
		return this.restate.run(`mcp-${this.clientInfo.name}-ping`, async () => {
			return super.ping(options);
		});
	}

	override async getPrompt(
		params: GetPromptRequest["params"],
		options?: RequestOptions,
	): Promise<{
		[x: string]: unknown;
		messages: {
			role: "user" | "assistant";
			content:
				| {
						type: "text";
						text: string;
						_meta?: Record<string, unknown> | undefined;
				  }
				| {
						type: "image";
						data: string;
						mimeType: string;
						_meta?: Record<string, unknown> | undefined;
				  }
				| {
						type: "audio";
						data: string;
						mimeType: string;
						_meta?: Record<string, unknown> | undefined;
				  }
				| {
						type: "resource";
						resource:
							| {
									uri: string;
									text: string;
									mimeType?: string | undefined;
									_meta?: Record<string, unknown> | undefined;
							  }
							| {
									uri: string;
									blob: string;
									mimeType?: string | undefined;
									_meta?: Record<string, unknown> | undefined;
							  };
						_meta?: Record<string, unknown> | undefined;
				  }
				| {
						uri: string;
						name: string;
						type: "resource_link";
						description?: string | undefined;
						mimeType?: string | undefined;
						_meta?:
							| {
									[x: string]: unknown;
							  }
							| undefined;
						icons?:
							| {
									src: string;
									mimeType?: string | undefined;
									sizes?: string[] | undefined;
							  }[]
							| undefined;
						title?: string | undefined;
				  };
		}[];
		_meta?: Record<string, unknown> | undefined;
		description?: string | undefined;
	}> {
		return this.restate.run(
			`mcp-${this.clientInfo.name}-getPrompt`,
			async () => {
				return super.getPrompt(params, options);
			},
		);
	}
	override async listPrompts(
		params?: ListPromptsRequest["params"],
		options?: RequestOptions,
	): Promise<{
		[x: string]: unknown;
		prompts: {
			name: string;
			description?: string | undefined;
			arguments?:
				| {
						name: string;
						description?: string | undefined;
						required?: boolean | undefined;
				  }[]
				| undefined;
			_meta?:
				| {
						[x: string]: unknown;
				  }
				| undefined;
			icons?:
				| {
						src: string;
						mimeType?: string | undefined;
						sizes?: string[] | undefined;
				  }[]
				| undefined;
			title?: string | undefined;
		}[];
		_meta?: Record<string, unknown> | undefined;
		nextCursor?: string | undefined;
	}> {
		return this.restate.run(
			`mcp-${this.clientInfo.name}-listPrompts`,
			async () => {
				return super.listPrompts(params, options);
			},
		);
	}
	override async listResources(
		params?: ListResourcesRequest["params"],
		options?: RequestOptions,
	): Promise<{
		[x: string]: unknown;
		resources: {
			uri: string;
			name: string;
			description?: string | undefined;
			mimeType?: string | undefined;
			_meta?:
				| {
						[x: string]: unknown;
				  }
				| undefined;
			icons?:
				| {
						src: string;
						mimeType?: string | undefined;
						sizes?: string[] | undefined;
				  }[]
				| undefined;
			title?: string | undefined;
		}[];
		_meta?: Record<string, unknown> | undefined;
		nextCursor?: string | undefined;
	}> {
		return this.restate.run(
			`mcp-${this.clientInfo.name}-listResources`,
			async () => {
				return super.listResources(params, options);
			},
		);
	}
	override async listResourceTemplates(
		params?: ListResourceTemplatesRequest["params"],
		options?: RequestOptions,
	): Promise<{
		[x: string]: unknown;
		resourceTemplates: {
			uriTemplate: string;
			name: string;
			description?: string | undefined;
			mimeType?: string | undefined;
			_meta?:
				| {
						[x: string]: unknown;
				  }
				| undefined;
			icons?:
				| {
						src: string;
						mimeType?: string | undefined;
						sizes?: string[] | undefined;
				  }[]
				| undefined;
			title?: string | undefined;
		}[];
		_meta?: Record<string, unknown> | undefined;
		nextCursor?: string | undefined;
	}> {
		return this.restate.run(
			`mcp-${this.clientInfo.name}-listResourceTemplates`,
			async () => {
				return super.listResourceTemplates(params, options);
			},
		);
	}
	override async readResource(
		params: ReadResourceRequest["params"],
		options?: RequestOptions,
	): Promise<{
		[x: string]: unknown;
		contents: (
			| {
					uri: string;
					text: string;
					mimeType?: string | undefined;
					_meta?: Record<string, unknown> | undefined;
			  }
			| {
					uri: string;
					blob: string;
					mimeType?: string | undefined;
					_meta?: Record<string, unknown> | undefined;
			  }
		)[];
		_meta?: Record<string, unknown> | undefined;
	}> {
		return this.restate.run(
			`mcp-${this.clientInfo.name}-readResource`,
			async () => {
				return super.readResource(params, options);
			},
		);
	}
	override async subscribeResource(
		params: SubscribeRequest["params"],
		options?: RequestOptions,
	): Promise<{
		_meta?: Record<string, unknown> | undefined;
	}> {
		return this.restate.run(
			`mcp-${this.clientInfo.name}-subscribeResource`,
			async () => {
				return super.subscribeResource(params, options);
			},
		);
	}
	override async unsubscribeResource(
		params: UnsubscribeRequest["params"],
		options?: RequestOptions,
	): Promise<{
		_meta?: Record<string, unknown> | undefined;
	}> {
		return this.restate.run(
			`mcp-${this.clientInfo.name}-unsubscribeResource`,
			async () => {
				return super.unsubscribeResource(params, options);
			},
		);
	}
	override async callTool(
		params: CallToolRequest["params"],
		resultSchema?:
			| typeof CallToolResultSchema
			| typeof CompatibilityCallToolResultSchema,
		options?: RequestOptions,
	): Promise<
		| {
				[x: string]: unknown;
				content: (
					| {
							type: "text";
							text: string;
							_meta?: Record<string, unknown> | undefined;
					  }
					| {
							type: "image";
							data: string;
							mimeType: string;
							_meta?: Record<string, unknown> | undefined;
					  }
					| {
							type: "audio";
							data: string;
							mimeType: string;
							_meta?: Record<string, unknown> | undefined;
					  }
					| {
							type: "resource";
							resource:
								| {
										uri: string;
										text: string;
										mimeType?: string | undefined;
										_meta?: Record<string, unknown> | undefined;
								  }
								| {
										uri: string;
										blob: string;
										mimeType?: string | undefined;
										_meta?: Record<string, unknown> | undefined;
								  };
							_meta?: Record<string, unknown> | undefined;
					  }
					| {
							uri: string;
							name: string;
							type: "resource_link";
							description?: string | undefined;
							mimeType?: string | undefined;
							_meta?:
								| {
										[x: string]: unknown;
								  }
								| undefined;
							icons?:
								| {
										src: string;
										mimeType?: string | undefined;
										sizes?: string[] | undefined;
								  }[]
								| undefined;
							title?: string | undefined;
					  }
				)[];
				_meta?: Record<string, unknown> | undefined;
				structuredContent?: Record<string, unknown> | undefined;
				isError?: boolean | undefined;
		  }
		| {
				[x: string]: unknown;
				toolResult: unknown;
				_meta?: Record<string, unknown> | undefined;
		  }
	> {
		return this.restate.run(
			`mcp-${this.clientInfo.name}-callTool`,
			async () => {
				return super.callTool(params, resultSchema, options);
			},
		);
	}

	override async listTools(
		params?: ListToolsRequest["params"],
		options?: RequestOptions,
	): Promise<{
		[x: string]: unknown;
		tools: {
			inputSchema: {
				[x: string]: unknown;
				type: "object";
				properties?: Record<string, object> | undefined;
				required?: string[] | undefined;
			};
			name: string;
			description?: string | undefined;
			outputSchema?:
				| {
						[x: string]: unknown;
						type: "object";
						properties?: Record<string, object> | undefined;
						required?: string[] | undefined;
				  }
				| undefined;
			annotations?:
				| {
						title?: string | undefined;
						readOnlyHint?: boolean | undefined;
						destructiveHint?: boolean | undefined;
						idempotentHint?: boolean | undefined;
						openWorldHint?: boolean | undefined;
				  }
				| undefined;
			_meta?: Record<string, unknown> | undefined;
			icons?:
				| {
						src: string;
						mimeType?: string | undefined;
						sizes?: string[] | undefined;
				  }[]
				| undefined;
			title?: string | undefined;
		}[];
		_meta?: Record<string, unknown> | undefined;
		nextCursor?: string | undefined;
	}> {
		return this.restate.run(
			`mcp-${this.clientInfo.name}-listTools`,
			async () => {
				return super.listTools(params, options);
			},
		);
	}
	override async sendRootsListChanged(): Promise<void> {
		return this.restate.run(
			`mcp-${this.clientInfo.name}-sendRootsListChanged`,
			async () => {
				return super.sendRootsListChanged();
			},
		);
	}
}
