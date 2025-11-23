"use client";
import { createFileRoute, useParams } from "@tanstack/react-router";
import { Plus, X } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
	Command,
	CommandEmpty,
	CommandGroup,
	CommandInput,
	CommandItem,
} from "@/components/ui/command";
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from "@/components/ui/popover";
import { ScrollArea } from "@/components/ui/scroll-area";
import $api from "@/lib/api-client.client";

export const Route = createFileRoute(
	"/bridge/manage-credentials/$providerInstanceId/functions",
)({
	component: RouteComponent,
});

function RouteComponent() {
	const { providerInstanceId } = useParams({
		from: "/bridge/manage-credentials/$providerInstanceId/functions",
	});

	// Query the specific provider instance with all its details
	const { data: providerInstanceData } = $api.useQuery(
		"get",
		"/api/bridge/v1/provider/{provider_instance_id}",
		{
			params: {
				path: {
					provider_instance_id: providerInstanceId,
				},
			},
		},
	);

	const instance = providerInstanceData?.provider_instance;
	const providerController = providerInstanceData?.controller;

	if (!instance || !providerController) {
		return null;
	}

	return (
		<div className="p-6 mt-0">
			<EnabledFunctionsTab
				providerInstance={instance}
				providerController={providerController}
			/>
		</div>
	);
}

// Enabled Functions Tab Component
const EnabledFunctionsTab = ({
	providerInstance,
	providerController,
}: {
	providerInstance: any;
	providerController: any;
}) => {
	const [open, setOpen] = useState(false);
	const [searchValue, setSearchValue] = useState("");

	// Query function instances for this provider instance
	const {
		data: functionInstancesData,
		isLoading: isLoadingFunctionInstances,
		refetch: refetchFunctionInstances,
	} = $api.useQuery("get", "/api/bridge/v1/function-instances", {
		params: {
			query: {
				page_size: 1000,
				provider_instance_id: providerInstance.id,
			},
		},
	});

	// Enable function mutation (creates a function instance)
	const enableFunctionMutation = $api.useMutation(
		"post",
		"/api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/enable",
	);

	// Disable function mutation (deletes a function instance)
	const disableFunctionMutation = $api.useMutation(
		"post",
		"/api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/disable",
	);

	const enabledFunctionInstances = functionInstancesData?.items || [];

	// Get available functions from provider controller
	const availableFunctions = providerController?.functions || [];

	// Filter out already enabled functions
	const unenabledFunctions = availableFunctions.filter(
		(func: any) =>
			!enabledFunctionInstances.some(
				(enabled) => enabled.function_controller_type_id === func.type_id,
			),
	);

	const handleEnableFunction = async (functionTypeId: string) => {
		try {
			await enableFunctionMutation.mutateAsync({
				params: {
					path: {
						provider_instance_id: providerInstance.id,
						function_controller_type_id: functionTypeId,
					},
				},
				body: {},
			});
			setOpen(false);
			setSearchValue("");
			await refetchFunctionInstances();
		} catch (error) {
			console.error("Failed to enable function:", error);
			alert("Failed to enable function. Please try again.");
		}
	};

	const handleDisableFunction = async (functionControllerTypeId: string) => {
		try {
			await disableFunctionMutation.mutateAsync({
				params: {
					path: {
						provider_instance_id: providerInstance.id,
						function_controller_type_id: functionControllerTypeId,
					},
				},
			});
			await refetchFunctionInstances();
		} catch (error) {
			console.error("Failed to disable function:", error);
			alert("Failed to disable function. Please try again.");
		}
	};

	if (isLoadingFunctionInstances) {
		return (
			<div className="flex items-center justify-center p-8">
				<p className="text-muted-foreground">Loading functions...</p>
			</div>
		);
	}

	// Find function name from provider controller
	const getFunctionName = (functionTypeId: string) => {
		const func = availableFunctions.find(
			(f: any) => f.type_id === functionTypeId,
		);
		return func?.name || functionTypeId;
	};

	return (
		<div className="space-y-4">
			{/* Add Function Dropdown */}
			<div className="space-y-2">
				<Popover open={open} onOpenChange={setOpen}>
					<PopoverTrigger asChild>
						<Button
							type="button"
							variant="outline"
							className="justify-start text-left font-normal"
							disabled={unenabledFunctions.length === 0}
						>
							<Plus className="mr-2 h-4 w-4" />
							Enable a new function
						</Button>
					</PopoverTrigger>
					<PopoverContent className="w-full p-0" align="start">
						<Command>
							<CommandInput
								placeholder="Search functions..."
								value={searchValue}
								onValueChange={setSearchValue}
							/>
							<CommandEmpty>No function found.</CommandEmpty>
							<CommandGroup>
								<ScrollArea className="h-[200px]">
									{unenabledFunctions.map((func: any) => (
										<CommandItem
											key={func.type_id}
											onSelect={() => {
												handleEnableFunction(func.type_id);
											}}
										>
											{func.name}
										</CommandItem>
									))}
								</ScrollArea>
							</CommandGroup>
						</Command>
					</PopoverContent>
				</Popover>
			</div>

			{/* Enabled Functions Table */}
			<div className="border rounded-lg bg-white">
				<div className="px-4 py-3 border-b">
					<h3 className="font-semibold text-sm">Enabled Functions</h3>
				</div>
				<div className="overflow-auto max-h-[350px]">
					{enabledFunctionInstances.length > 0 ? (
						<table className="w-full">
							<thead className="sticky top-0 z-10">
								<tr className="border-b bg-gray-50">
									<th className="px-3 py-2 text-left text-xs font-medium">
										Function Name
									</th>
									<th className="w-50 px-3 py-2"></th>
								</tr>
							</thead>
							<tbody>
								{enabledFunctionInstances.map((functionInstance, index) => (
									<tr
										key={`${functionInstance.function_controller_type_id}-${functionInstance.provider_controller_type_id}-${functionInstance.provider_instance_id}`}
										className={`${
											index % 2 === 1 ? "bg-gray-50/50" : ""
										} hover:bg-gray-50 transition-colors`}
									>
										<td className="px-3 py-2 text-sm font-medium">
											{getFunctionName(
												functionInstance.function_controller_type_id,
											)}
										</td>
										<td className="px-3 py-2 text-right w-50">
											<Button
												type="button"
												variant="ghost"
												size="sm"
												onClick={() =>
													handleDisableFunction(
														functionInstance.function_controller_type_id,
													)
												}
												className="hover:bg-red-50 hover:text-red-600"
												disabled={disableFunctionMutation.isPending}
											>
												<X className="h-4 w-4 mr-1" />
												Disable
											</Button>
										</td>
									</tr>
								))}
							</tbody>
						</table>
					) : (
						<div className="p-8 text-center text-sm text-muted-foreground">
							No functions enabled for these credentials yet
						</div>
					)}
				</div>
			</div>
		</div>
	);
};
