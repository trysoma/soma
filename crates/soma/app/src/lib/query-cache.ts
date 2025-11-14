import type { QueryClient } from "@tanstack/react-query";

export const invalidateDataByControllerTypeId = (
	queryClient: QueryClient,
	providerControllerTypeId: string,
) => {
	queryClient.invalidateQueries({
		queryKey: [
			"get",
			"/api/bridge/v1/provider/grouped-by-function",
			{ provider_controller_type_id: providerControllerTypeId },
		],
		exact: false,
	});
	queryClient.invalidateQueries({
		queryKey: [
			"get",
			"/api/bridge/v1/function-instances",
			{ provider_controller_type_id: providerControllerTypeId },
		],
		exact: false,
	});
	queryClient.invalidateQueries({
		queryKey: [
			"get",
			"/api/bridge/v1/provider",
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
			"/api/bridge/v1/function-instances",
			{ provider_instance_id: providerInstanceId },
		],
		exact: false,
	});
	queryClient.invalidateQueries({
		queryKey: ["get", "/api/bridge/v1/provider"],
		exact: false,
	});
};

export const invalidateFunctionInstancesData = (queryClient: QueryClient) => {
	queryClient.invalidateQueries({
		queryKey: ["get", "/api/bridge/v1/function-instances"],
		exact: false,
	});
	queryClient.invalidateQueries({
		queryKey: ["get", "/api/bridge/v1/provider/grouped-by-function"],
		exact: false,
	});
};
