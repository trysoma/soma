/**
 * AI SDK MCP client wrapper that uses our durable MCP client.
 * Provides AI SDK compatible tools from Soma MCP servers.
 */

import type { JSONSchema7 } from '@ai-sdk/provider';
// Import from 'ai' package to ensure version compatibility with the ai SDK's internal validation
import {
	dynamicTool,
	jsonSchema,
	tool,
	type Tool,
	type ToolExecutionOptions,
} from "ai";
import type {
	ListResourcesResult,
	ListResourceTemplatesResult,
	ListPromptsResult,
	GetPromptResult,
	Notification,
	Result,
} from "@modelcontextprotocol/sdk/types.js";
import { ObjectContext } from "@restatedev/restate-sdk";
import { createSomaMcpClient, SomaMcpClient } from "./mcp.js";
import { z } from "zod";

/**
 * Tool metadata from MCP server.
 */
export interface ToolMeta {
	[key: string]: unknown;
}

/**
 * MCP tool result content types.
 */
export interface CallToolResult {
	content: Array<
		| { type: "text"; text: string; _meta?: Record<string, unknown> }
		| { type: "image"; data: string; mimeType: string; _meta?: Record<string, unknown> }
		| { type: "audio"; data: string; mimeType: string; _meta?: Record<string, unknown> }
		| {
			type: "resource";
			resource:
				| { uri: string; text: string; mimeType?: string; _meta?: Record<string, unknown> }
				| { uri: string; blob: string; mimeType?: string; _meta?: Record<string, unknown> };
			_meta?: Record<string, unknown>;
		}
		| {
			type: "resource_link";
			uri: string;
			name: string;
			description?: string;
			mimeType?: string;
			_meta?: Record<string, unknown>;
		}
	>;
	isError?: boolean;
	structuredContent?: Record<string, unknown>;
	_meta?: Record<string, unknown>;
}

/**
 * Tool schemas configuration - either 'automatic' for JSON schema inference
 * or a record of tool names to their Zod schemas.
 */
export type ToolSchemas = 'automatic' | Record<string, { inputSchema: z.ZodType }>;

/**
 * AI SDK tool with optional metadata.
 */
export type McpTool = Tool & { _meta?: ToolMeta };

/**
 * Tool set type based on schema configuration.
 * When schemas is 'automatic', tools use dynamicTool with JSON schema.
 * When schemas is a record, tools use the provided Zod schemas.
 */
export type McpToolSet<TOOL_SCHEMAS extends ToolSchemas = 'automatic'> =
	TOOL_SCHEMAS extends 'automatic'
		? Record<string, McpTool>
		: { [K in keyof TOOL_SCHEMAS & string]: McpTool };

/**
 * Paginated request parameters.
 */
export interface PaginatedRequest {
	params?: {
		cursor?: string;
	};
}

/**
 * Request options for MCP operations.
 */
export interface RequestOptions {
	signal?: AbortSignal;
}

/**
 * Configuration options for creating a SomaVercelAiSdkMcpClient.
 */
export interface SomaMcpToolsConfig<RequestT extends Request = Request, NotificationT extends Notification = Notification, ResultT extends Result = Result> {
	/**
	 * The base MCP client - either an existing SomaMcpClient instance
	 * or configuration to create one.
	 */
	baseMcpClient: SomaMcpClient<RequestT, NotificationT, ResultT> | {
		somaBaseUrl?: string;
		mcpServerInstanceId: string;
	};
}

/**
 * AI SDK MCP client wrapper that uses Soma's durable MCP client.
 * Provides AI SDK compatible tools from Soma MCP servers.
 *
 * This client is modeled after the Vercel AI SDK's MCPClient implementation
 * but uses SomaMcpClient as the underlying transport.
 */
export class SomaVercelAiSdkMcpClient<RequestT extends Request = Request, NotificationT extends Notification = Notification, ResultT extends Result = Result> {
	private mcpClient: SomaMcpClient<RequestT, NotificationT, ResultT>;

	/**
	 * Create a new SomaVercelAiSdkMcpClient instance.
	 *
	 * @param mcpClient - A SomaMcpClient instance
	 */
	constructor(mcpClient: SomaMcpClient<RequestT, NotificationT, ResultT>) {
		this.mcpClient = mcpClient;
	}

	/**
	 * Returns a set of AI SDK tools from the MCP server.
	 *
	 * Tool parameters are automatically inferred from the server's JSON schema
	 * if not explicitly provided in the schemas configuration.
	 *
	 * @param options - Optional configuration with schemas for typed tools
	 * @returns A record of tool names to their AI SDK implementations
	 *
	 * @example
	 * ```typescript
	 * import { createSomaAiSdkMcpClient } from '@trysoma/sdk/vercel-ai-sdk';
	 * import { generateText } from 'ai';
	 *
	 * const mcpClient = await createSomaAiSdkMcpClient(ctx, {
	 *   baseMcpClient: { mcpServerInstanceId: 'my-instance' }
	 * });
	 * const tools = await mcpClient.tools();
	 *
	 * const result = await generateText({
	 *   model: myModel,
	 *   tools,
	 *   prompt: 'Use the available tools...',
	 * });
	 * ```
	 */
	async tools<TOOL_SCHEMAS extends ToolSchemas = 'automatic'>({
		schemas = 'automatic' as TOOL_SCHEMAS,
	}: {
		schemas?: TOOL_SCHEMAS;
	} = {}): Promise<McpToolSet<TOOL_SCHEMAS>> {
		const tools: Record<string, McpTool> = {};
		const listToolsResult = await this.mcpClient.listTools();

		for (const mcpTool of listToolsResult.tools) {
			const { name, description, inputSchema, annotations, _meta } = mcpTool;
			const title = annotations?.title ?? mcpTool.title;

			// Skip tools not in the provided schemas when using typed schemas
			if (schemas !== 'automatic' && !(name in (schemas as object))) {
				continue;
			}

			const self = this;

			const execute = async (
				// biome-ignore lint/suspicious/noExplicitAny: matches AI SDK pattern
				args: any,
				options: ToolExecutionOptions,
			): Promise<CallToolResult> => {
				options?.abortSignal?.throwIfAborted();
				const result = await self.mcpClient.callTool({ name, arguments: args });
				return result as CallToolResult;
			};

			const toolWithExecute =
				schemas === 'automatic'
					? dynamicTool({
						description,
						title,
						inputSchema: jsonSchema({
							...inputSchema,
							properties: inputSchema.properties ?? {},
							additionalProperties: false,
						} as JSONSchema7),
						execute,
					})
					// biome-ignore lint/suspicious/noExplicitAny: matches AI SDK pattern for typed schemas
					: tool({
						description,
						title,
						inputSchema: (schemas as Record<string, { inputSchema: z.ZodType<any> }>)[name]!.inputSchema,
						execute,
					});

			tools[name] = { ...toolWithExecute, _meta };
		}

		return tools as McpToolSet<TOOL_SCHEMAS>;
	}

	/**
	 * List available resources from the MCP server.
	 */
	async listResources(options?: {
		params?: PaginatedRequest['params'];
	}): Promise<ListResourcesResult> {
		return this.mcpClient.listResources(options?.params);
	}

	/**
	 * Read a resource by URI from the MCP server.
	 */
	async readResource(args: {
		uri: string;
	}): Promise<Awaited<ReturnType<typeof this.mcpClient.readResource>>> {
		return this.mcpClient.readResource({ uri: args.uri });
	}

	/**
	 * List available resource templates from the MCP server.
	 */
	async listResourceTemplates(): Promise<ListResourceTemplatesResult> {
		return this.mcpClient.listResourceTemplates();
	}

	/**
	 * List available prompts from the MCP server.
	 */
	async listPrompts(options?: {
		params?: PaginatedRequest['params'];
	}): Promise<ListPromptsResult> {
		return this.mcpClient.listPrompts(options?.params);
	}

	/**
	 * Get a prompt by name from the MCP server.
	 */
	async getPrompt(args: {
		name: string;
		arguments?: Record<string, string>;
	}): Promise<GetPromptResult> {
		return this.mcpClient.getPrompt({ name: args.name, arguments: args.arguments });
	}
}

/**
 * Create a SomaVercelAiSdkMcpClient instance.
 *
 * This factory function creates an AI SDK compatible MCP client that wraps
 * Soma's durable MCP client. It can either use an existing SomaMcpClient
 * or create a new one from configuration.
 *
 * @param restate - The Restate ObjectContext for durable execution
 * @param config - Configuration including the base MCP client or connection details
 * @returns A SomaVercelAiSdkMcpClient instance
 *
 * @example
 * ```typescript
 * import { createSomaAiSdkMcpClient } from '@trysoma/sdk/vercel-ai-sdk';
 * import { generateText } from 'ai';
 *
 * // Using connection config
 * const mcpClient = await createSomaAiSdkMcpClient(ctx, {
 *   baseMcpClient: { mcpServerInstanceId: 'my-instance' }
 * });
 *
 * // Or using an existing SomaMcpClient
 * const somaMcpClient = await createSomaMcpClient(ctx, 'my-instance');
 * const mcpClient = await createSomaAiSdkMcpClient(ctx, {
 *   baseMcpClient: somaMcpClient
 * });
 *
 * const tools = await mcpClient.tools();
 *
 * const result = await generateText({
 *   model: myModel,
 *   tools,
 *   prompt: 'Use the available tools...',
 * });
 * ```
 */
export async function createSomaAiSdkMcpClient<RequestT extends Request = Request, NotificationT extends Notification = Notification, ResultT extends Result = Result>(
	restate: ObjectContext,
	config: SomaMcpToolsConfig<RequestT, NotificationT, ResultT>,
): Promise<SomaVercelAiSdkMcpClient<RequestT, NotificationT, ResultT>> {
	const baseMcpClient = config.baseMcpClient;
	if (baseMcpClient instanceof SomaMcpClient) {
		return new SomaVercelAiSdkMcpClient(baseMcpClient);
	}

	// Create a new client from config with proper generics
	const mcpClient = await createSomaMcpClient<RequestT, NotificationT, ResultT>(
		restate,
		baseMcpClient.mcpServerInstanceId,
		{
			somaBaseUrl: baseMcpClient.somaBaseUrl,
		},
	);
	return new SomaVercelAiSdkMcpClient(mcpClient);
}
