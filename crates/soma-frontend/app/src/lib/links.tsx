export const LINKS = {
	// Agents list page
	AGENTS: () => `/agent`,
	// Agent-specific A2A routes: /agent/{projectId}/{agentId}/a2a/...
	AGENT_A2A_OVERVIEW: (projectId: string, agentId: string) =>
		`/agent/${projectId}/${agentId}/a2a/overview`,
	AGENT_A2A_CHAT_DEBUGGER: (projectId: string, agentId: string) =>
		`/agent/${projectId}/${agentId}/a2a/chat-debugger`,
	BRIDGE: () => `/bridge`,
	BRIDGE_ENABLE_FUNCTIONS: () => `/bridge/enable-functions`,
	BRIDGE_MANAGE_CREDENTIALS: () => `/bridge/manage-credentials`,
	BRIDGE_MCP_SERVERS: () => `/bridge/mcp-servers`,
	// Legacy - kept for backwards compatibility, redirects to MCP Servers tab
	BRIDGE_MCP_INSPECTOR: () => `/bridge/mcp-servers`,

	// Enable Functions - Dynamic links with functionControllerId parameter
	BRIDGE_ENABLE_FUNCTIONS_FUNCTION: (functionControllerId: string) =>
		`/bridge/enable-functions/available/${functionControllerId}/function_documentation`,
	BRIDGE_ENABLE_FUNCTIONS_PROVIDER: (functionControllerId: string) =>
		`/bridge/enable-functions/available/${functionControllerId}/provider_documentation`,
	BRIDGE_ENABLE_FUNCTIONS_CONFIGURE: (functionControllerId: string) =>
		`/bridge/enable-functions/available/${functionControllerId}/configure`,
	BRIDGE_ENABLE_FUNCTIONS_CONFIGURE_NEW: (functionControllerId: string) =>
		`/bridge/enable-functions/available/${functionControllerId}/configure/new`,
	BRIDGE_ENABLE_FUNCTIONS_CONFIGURE_EXISTING: (functionControllerId: string) =>
		`/bridge/enable-functions/available/${functionControllerId}/configure/existing`,
	BRIDGE_ENABLE_FUNCTIONS_TEST: (functionControllerId: string) =>
		`/bridge/enable-functions/available/${functionControllerId}/test`,

	// Manage Credentials - Dynamic links with providerInstanceId parameter
	BRIDGE_MANAGE_CREDENTIALS_DOCUMENTATION: (providerInstanceId: string) =>
		`/bridge/manage-credentials/${providerInstanceId}/documentation`,
	BRIDGE_MANAGE_CREDENTIALS_CONFIGURATION: (providerInstanceId: string) =>
		`/bridge/manage-credentials/${providerInstanceId}/configuration`,
	BRIDGE_MANAGE_CREDENTIALS_FUNCTIONS: (providerInstanceId: string) =>
		`/bridge/manage-credentials/${providerInstanceId}/functions`,
	BRIDGE_MANAGE_CREDENTIALS_DELETE: (providerInstanceId: string) =>
		`/bridge/manage-credentials/${providerInstanceId}/delete`,

	// MCP Servers - Dynamic links with mcpServerInstanceId parameter
	BRIDGE_MCP_SERVER_CONFIGURE: (mcpServerInstanceId: string) =>
		`/bridge/mcp-servers/${mcpServerInstanceId}/configure`,
	BRIDGE_MCP_SERVER_INSPECTOR: (mcpServerInstanceId: string) =>
		`/bridge/mcp-servers/${mcpServerInstanceId}/inspector`,
};
