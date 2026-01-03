"use client";
import { useEffect, useState } from "react";
import type { components } from "@/@types/openapi";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { SlideOutPanel } from "@/components/ui/slide-out-panel";
import { Textarea } from "@/components/ui/textarea";
import $api from "@/lib/api-client.client";

type McpServerInstanceFunction =
	components["schemas"]["McpServerInstanceFunctionSerialized"];

interface UpdateFunctionSlideoutProps {
	isOpen: boolean;
	onClose: () => void;
	onSuccess: () => void;
	mcpServerInstanceId: string;
	functionData: McpServerInstanceFunction;
}

export function UpdateFunctionSlideout({
	isOpen,
	onClose,
	onSuccess,
	mcpServerInstanceId,
	functionData,
}: UpdateFunctionSlideoutProps) {
	const [functionName, setFunctionName] = useState(functionData.function_name);
	const [functionDescription, setFunctionDescription] = useState(
		functionData.function_description || "",
	);
	const [error, setError] = useState<string | null>(null);

	// Reset form when function data changes
	useEffect(() => {
		setFunctionName(functionData.function_name);
		setFunctionDescription(functionData.function_description || "");
		setError(null);
	}, [functionData]);

	// Query available providers for metadata
	const { data: availableProvidersData } = $api.useQuery(
		"get",
		"/api/mcp/v1/available-providers",
		{
			params: {
				query: {
					page_size: 1000,
				},
			},
		},
		{
			enabled: isOpen,
		},
	);

	// Query provider instances for credential info
	const { data: providerInstancesData } = $api.useQuery(
		"get",
		"/api/mcp/v1/provider",
		{
			params: {
				query: {
					page_size: 1000,
				},
			},
		},
		{
			enabled: isOpen,
		},
	);

	// Update function mutation
	const updateFunction = $api.useMutation(
		"patch",
		"/api/mcp/v1/mcp-server/{mcp_server_instance_id}/function/{function_controller_type_id}/{provider_controller_type_id}/{provider_instance_id}",
	);

	// Get metadata helpers
	const getProviderControllerName = (providerControllerTypeId: string) => {
		const provider = availableProvidersData?.items?.find(
			(p) => p.type_id === providerControllerTypeId,
		);
		return provider?.name || providerControllerTypeId;
	};

	const getFunctionControllerName = (
		functionControllerTypeId: string,
		providerControllerTypeId: string,
	) => {
		const provider = availableProvidersData?.items?.find(
			(p) => p.type_id === providerControllerTypeId,
		);
		const func = provider?.functions?.find(
			(f) => f.type_id === functionControllerTypeId,
		);
		return func?.name || functionControllerTypeId;
	};

	const getProviderInstanceName = (providerInstanceId: string) => {
		const instance = providerInstancesData?.items?.find(
			(p) => p.id === providerInstanceId,
		);
		return instance?.display_name || providerInstanceId;
	};

	const handleSubmit = async () => {
		if (!functionName.trim()) {
			setError("Function name is required");
			return;
		}

		// Validate function name format (snake_case)
		if (!/^[a-z][a-z0-9_]*$/.test(functionName)) {
			setError(
				"Function name must be in snake_case format (lowercase letters, numbers, underscores)",
			);
			return;
		}

		try {
			await updateFunction.mutateAsync({
				params: {
					path: {
						mcp_server_instance_id: mcpServerInstanceId,
						function_controller_type_id:
							functionData.function_controller_type_id,
						provider_controller_type_id:
							functionData.provider_controller_type_id,
						provider_instance_id: functionData.provider_instance_id,
					},
				},
				body: {
					function_name: functionName.trim(),
					function_description: functionDescription.trim() || null,
				},
			});
			onSuccess();
		} catch (err) {
			setError(
				"Failed to update function. The name may already be in use by another function.",
			);
			console.error("Failed to update function:", err);
		}
	};

	const handleClose = () => {
		setError(null);
		onClose();
	};

	return (
		<SlideOutPanel
			isOpen={isOpen}
			onClose={handleClose}
			title="Update function"
			subtitle="Modify the function name and description"
		>
			<div className="flex flex-col h-full">
				<ScrollArea className="flex-1 p-4">
					<div className="space-y-6">
						{/* Function info (read-only) */}
						<div className="p-4 bg-muted rounded-lg">
							<span className="font-medium text-sm text-muted-foreground">
								Function details
							</span>
							<div className="grid grid-cols-2 gap-2 text-sm mt-2">
								<div>
									<span className="text-muted-foreground">Provider:</span>{" "}
									{getProviderControllerName(
										functionData.provider_controller_type_id,
									)}
								</div>
								<div>
									<span className="text-muted-foreground">Function:</span>{" "}
									{getFunctionControllerName(
										functionData.function_controller_type_id,
										functionData.provider_controller_type_id,
									)}
								</div>
								<div>
									<span className="text-muted-foreground">Credentials:</span>{" "}
									{getProviderInstanceName(functionData.provider_instance_id)}
								</div>
							</div>
						</div>

						{/* Function name */}
						<div className="space-y-2">
							<Label htmlFor="functionName">MCP Function Name</Label>
							<Input
								id="functionName"
								placeholder="send_email"
								value={functionName}
								onChange={(e) => setFunctionName(e.target.value)}
							/>
							<p className="text-xs text-muted-foreground">
								The name that will appear in the MCP tool list. Use snake_case
								format.
							</p>
						</div>

						{/* Function description */}
						<div className="space-y-2">
							<Label htmlFor="functionDescription">
								Description (optional)
							</Label>
							<Textarea
								id="functionDescription"
								placeholder="Send an email using the configured email provider"
								value={functionDescription}
								onChange={(e) => setFunctionDescription(e.target.value)}
								rows={3}
								className="bg-white"
							/>
							<p className="text-xs text-muted-foreground">
								A description that helps users understand what this function
								does.
							</p>
						</div>

						{error && <p className="text-sm text-destructive">{error}</p>}
					</div>
				</ScrollArea>

				{/* Footer */}
				<div className="border-t p-4 flex justify-end gap-2">
					<Button variant="outline" onClick={handleClose}>
						Cancel
					</Button>
					<Button onClick={handleSubmit} disabled={updateFunction.isPending}>
						{updateFunction.isPending ? "Saving..." : "Save changes"}
					</Button>
				</div>
			</div>
		</SlideOutPanel>
	);
}
