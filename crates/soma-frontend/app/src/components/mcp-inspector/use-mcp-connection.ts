import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StreamableHTTPClientTransport } from "@modelcontextprotocol/sdk/client/streamableHttp.js";
import type { Transport } from "@modelcontextprotocol/sdk/shared/transport.js";
import type {
	ClientRequest,
	ServerCapabilities,
	ServerNotification,
} from "@modelcontextprotocol/sdk/types.js";
import { useCallback, useEffect, useRef, useState } from "react";
import { z } from "zod";

type ConnectionStatus = "disconnected" | "connecting" | "connected" | "error";

interface UseMCPConnectionOptions {
	serverUrl: string;
}

interface RequestHistoryItem {
	request: string;
	response?: string;
}

export function useMCPConnection({ serverUrl }: UseMCPConnectionOptions) {
	const [connectionStatus, setConnectionStatus] =
		useState<ConnectionStatus>("disconnected");
	const [serverCapabilities, setServerCapabilities] =
		useState<ServerCapabilities | null>(null);
	const [requestHistory, setRequestHistory] = useState<RequestHistoryItem[]>(
		[],
	);
	const [notifications, setNotifications] = useState<ServerNotification[]>([]);

	// Use refs to store client and transport to avoid stale closure issues
	const mcpClientRef = useRef<Client | null>(null);
	const transportRef = useRef<Transport | null>(null);
	// Track if we're currently connecting to prevent race conditions
	const isConnectingRef = useRef(false);

	const pushHistory = useCallback((request: object, response?: object) => {
		setRequestHistory((prev) => [
			...prev,
			{
				request: JSON.stringify(request),
				response: response !== undefined ? JSON.stringify(response) : undefined,
			},
		]);
	}, []);

	const disconnect = useCallback(async () => {
		const client = mcpClientRef.current;
		if (client) {
			try {
				await client.close();
			} catch (error) {
				// Ignore close errors - connection may already be closed
				console.debug("Error closing MCP client:", error);
			}
		}
		mcpClientRef.current = null;
		transportRef.current = null;
		setConnectionStatus("disconnected");
		setServerCapabilities(null);
	}, []);

	const connect = useCallback(async () => {
		// Prevent concurrent connection attempts
		if (isConnectingRef.current) {
			return;
		}

		// Close any existing connection first
		if (mcpClientRef.current) {
			await disconnect();
		}

		isConnectingRef.current = true;

		try {
			setConnectionStatus("connecting");

			const clientCapabilities = {
				capabilities: {
					sampling: {},
					roots: {
						listChanged: true,
					},
				},
			};

			const client = new Client(
				{ name: "mcp-inspector", version: "1.0.0" },
				clientCapabilities,
			);

			const transport = new StreamableHTTPClientTransport(new URL(serverUrl));

			// Set up close handler before connecting
			client.onclose = () => {
				console.debug("MCP client connection closed");
				mcpClientRef.current = null;
				transportRef.current = null;
				setConnectionStatus("disconnected");
				setServerCapabilities(null);
			};

			// Set up error handler
			client.onerror = (error) => {
				console.error("MCP client error:", error);
				setConnectionStatus("error");
			};

			await client.connect(transport as Transport);

			mcpClientRef.current = client;
			transportRef.current = transport;

			const capabilities = client.getServerCapabilities();
			setServerCapabilities(capabilities ?? null);

			pushHistory(
				{ method: "initialize" },
				{
					capabilities,
					serverInfo: client.getServerVersion(),
				},
			);

			// Set up notification handler
			client.fallbackNotificationHandler = (
				notification: any,
			): Promise<void> => {
				setNotifications((prev) => [
					...prev,
					notification as ServerNotification,
				]);
				return Promise.resolve();
			};

			setConnectionStatus("connected");
		} catch (error) {
			console.error("Failed to connect to MCP server:", error);
			setConnectionStatus("error");
			mcpClientRef.current = null;
			transportRef.current = null;
		} finally {
			isConnectingRef.current = false;
		}
	}, [serverUrl, pushHistory, disconnect]);

	// Cleanup on unmount
	useEffect(() => {
		return () => {
			const client = mcpClientRef.current;
			if (client) {
				client.close().catch((error) => {
					console.debug("Error closing MCP client on unmount:", error);
				});
			}
		};
	}, []);

	const makeRequest = useCallback(
		async (request: ClientRequest, schema?: any): Promise<any> => {
			const client = mcpClientRef.current;
			if (!client) {
				throw new Error("MCP client not connected");
			}

			try {
				// If no schema provided, use a passthrough object schema
				const responseSchema = schema || z.object({}).catchall(z.any());
				const response = await client.request(request as any, responseSchema);
				pushHistory(request, response);
				return response;
			} catch (error) {
				const errorMessage =
					error instanceof Error ? error.message : String(error);
				pushHistory(request, { error: errorMessage });
				throw error;
			}
		},
		[pushHistory],
	);

	const clearRequestHistory = useCallback(() => {
		setRequestHistory([]);
	}, []);

	const clearNotifications = useCallback(() => {
		setNotifications([]);
	}, []);

	return {
		connectionStatus,
		serverCapabilities,
		requestHistory,
		notifications,
		connect,
		disconnect,
		makeRequest,
		clearRequestHistory,
		clearNotifications,
	};
}
