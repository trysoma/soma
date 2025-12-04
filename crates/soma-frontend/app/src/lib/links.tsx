export const LINKS = {
	A2A: () => `/a2a`,
	A2A_CHAT: () => `/a2a/chat`,
	// Agent-specific A2A routes
	A2A_AGENT: (projectId: string, agentId: string) =>
		`/a2a/agent/${projectId}/${agentId}`,
	A2A_AGENT_CHAT: (projectId: string, agentId: string) =>
		`/a2a/agent/${projectId}/${agentId}/chat`,
	BRIDGE: () => `/bridge`,
	BRIDGE_ENABLE_FUNCTIONS: () => `/bridge/enable-functions`,
	BRIDGE_MANAGE_CREDENTIALS: () => `/bridge/manage-credentials`,
	BRIDGE_MCP_INSPECTOR: () => `/bridge/mcp-inspector`,

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
};
