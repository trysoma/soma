import type { QueryClient } from "@tanstack/react-query";

export const invalidateDataByControllerTypeId = (
	queryClient: QueryClient,
	providerControllerTypeId: string,
) => {
	queryClient.invalidateQueries({
		queryKey: [
			"get",
			"/api/mcp/v1/provider/grouped-by-function",
			{ provider_controller_type_id: providerControllerTypeId },
		],
		exact: false,
	});
	queryClient.invalidateQueries({
		queryKey: [
			"get",
			"/api/mcp/v1/function-instances",
			{ provider_controller_type_id: providerControllerTypeId },
		],
		exact: false,
	});
	queryClient.invalidateQueries({
		queryKey: [
			"get",
			"/api/mcp/v1/provider",
			{ provider_controller_type_id: providerControllerTypeId },
		],
		exact: false,
	});
};

export const invalidateDataByProviderInstanceId = (
	queryClient: QueryClient,
	providerInstanceId: string,
) => {
	queryClient.invalidateQueries({
		queryKey: [
			"get",
			"/api/mcp/v1/function-instances",
			{ provider_instance_id: providerInstanceId },
		],
		exact: false,
	});
	queryClient.invalidateQueries({
		queryKey: ["get", "/api/mcp/v1/provider"],
		exact: false,
	});
};

export const invalidateFunctionInstancesData = (queryClient: QueryClient) => {
	queryClient.invalidateQueries({
		queryKey: ["get", "/api/mcp/v1/function-instances"],
		exact: false,
	});
	queryClient.invalidateQueries({
		queryKey: ["get", "/api/mcp/v1/provider/grouped-by-function"],
		exact: false,
	});
};

export const invalidateMcpInstancesData = (queryClient: QueryClient) => {
	// Invalidate all MCP instance related queries using predicate to match any query key containing the path
	queryClient.invalidateQueries({
		predicate: (query) => {
			const queryKey = query.queryKey;
			return (
				Array.isArray(queryKey) &&
				queryKey.some(
					(key) =>
						typeof key === "string" && key.includes("/api/mcp/v1/mcp-server"),
				)
			);
		},
	});
};

export const invalidateMcpInstanceById = (
	queryClient: QueryClient,
	_mcpServerInstanceId: string,
) => {
	// Invalidate all MCP instance queries - the predicate approach handles both list and individual queries
	invalidateMcpInstancesData(queryClient);
};
