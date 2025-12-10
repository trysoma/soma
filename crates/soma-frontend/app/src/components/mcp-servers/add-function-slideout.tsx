"use client";
import { Check } from "lucide-react";
import { useMemo, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { SlideOutPanel } from "@/components/ui/slide-out-panel";
import {
	Table,
	TableBody,
	TableCell,
	TableContainer,
	TableEmpty,
	TableHead,
	TableHeader,
	TableRow,
	TableTitle,
	TableWrapper,
} from "@/components/ui/table";
import { Textarea } from "@/components/ui/textarea";
import $api from "@/lib/api-client.client";

interface EnabledFunction {
	functionControllerTypeId: string;
	functionControllerName: string;
	providerControllerTypeId: string;
	providerControllerName: string;
	providerInstanceId: string;
	providerInstanceName: string;
}

interface AddFunctionSlideoutProps {
	isOpen: boolean;
	onClose: () => void;
	onSuccess: () => void;
	mcpServerInstanceId: string;
}

export function AddFunctionSlideout({
	isOpen,
	onClose,
	onSuccess,
	mcpServerInstanceId,
}: AddFunctionSlideoutProps) {
	const [selectedFunction, setSelectedFunction] =
		useState<EnabledFunction | null>(null);
	const [functionName, setFunctionName] = useState("");
	const [functionDescription, setFunctionDescription] = useState("");
	const [error, setError] = useState<string | null>(null);

	// Query enabled functions (provider instances grouped by function)
	const { data: functionInstanceData, isLoading: isLoadingFunctions } =
		$api.useQuery(
			"get",
			"/api/bridge/v1/provider/grouped-by-function",
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

	// Query current MCP server instance to get already added functions
	const { data: mcpInstance } = $api.useQuery(
		"get",
		"/api/bridge/v1/mcp-instance/{mcp_server_instance_id}",
		{
			params: {
				path: {
					mcp_server_instance_id: mcpServerInstanceId,
				},
			},
		},
		{
			enabled: isOpen,
		},
	);

	// Add function mutation
	const addFunction = $api.useMutation(
		"post",
		"/api/bridge/v1/mcp-instance/{mcp_server_instance_id}/function",
	);

	// Transform function instance data into enabled functions
	const enabledFunctions = useMemo(() => {
		if (!functionInstanceData?.items) return [];

		const functions: EnabledFunction[] = [];
		const existingFunctions = new Set(
			mcpInstance?.functions?.map(
				(f) =>
					`${f.function_controller_type_id}:${f.provider_controller_type_id}:${f.provider_instance_id}`,
			) || [],
		);

		for (const item of functionInstanceData.items) {
			for (const providerInstance of item.provider_instances || []) {
				const key = `${item.function_controller.type_id}:${item.provider_controller.type_id}:${providerInstance.provider_instance.id}`;
				// Skip functions that are already added to this MCP server
				if (existingFunctions.has(key)) continue;

				functions.push({
					functionControllerTypeId: item.function_controller.type_id,
					functionControllerName: item.function_controller.name,
					providerControllerTypeId: item.provider_controller.type_id,
					providerControllerName: item.provider_controller.name,
					providerInstanceId: providerInstance.provider_instance.id,
					providerInstanceName: providerInstance.provider_instance.display_name,
				});
			}
		}

		return functions;
	}, [functionInstanceData, mcpInstance]);

	const handleSelectFunction = (func: EnabledFunction) => {
		setSelectedFunction(func);
		// Default function name to the function controller name (snake_case)
		setFunctionName(func.functionControllerTypeId);
		setFunctionDescription("");
		setError(null);
	};

	const handleSubmit = async () => {
		if (!selectedFunction) {
			setError("Please select a function");
			return;
		}

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
			await addFunction.mutateAsync({
				params: {
					path: {
						mcp_server_instance_id: mcpServerInstanceId,
					},
				},
				body: {
					function_controller_type_id:
						selectedFunction.functionControllerTypeId,
					provider_controller_type_id:
						selectedFunction.providerControllerTypeId,
					provider_instance_id: selectedFunction.providerInstanceId,
					function_name: functionName.trim(),
					function_description: functionDescription.trim() || null,
				},
			});
			handleClose();
			onSuccess();
		} catch (err) {
			setError("Failed to add function. It may already exist with that name.");
			console.error("Failed to add function:", err);
		}
	};

	const handleClose = () => {
		setSelectedFunction(null);
		setFunctionName("");
		setFunctionDescription("");
		setError(null);
		onClose();
	};

	return (
		<SlideOutPanel
			isOpen={isOpen}
			onClose={handleClose}
			title="Add function"
			subtitle="Select an enabled function to add to this MCP server"
		>
			<div className="flex flex-col h-full">
				{!selectedFunction ? (
					<>
						{/* Function selection view */}
						<ScrollArea className="flex-1">
							{isLoadingFunctions ? (
								<div className="flex items-center justify-center p-8">
									<p className="text-muted-foreground">
										Loading enabled functions...
									</p>
								</div>
							) : (
								<div className="p-4">
									<TableWrapper>
										<TableTitle>Available Functions</TableTitle>
										<TableContainer maxHeight="max-h-none">
											{enabledFunctions.length === 0 ? (
												<TableEmpty>
													No enabled functions available. Enable functions in
													the "Enable functions" tab first, or all enabled
													functions have already been added to this MCP server.
												</TableEmpty>
											) : (
												<Table>
													<TableHeader sticky>
														<TableRow>
															<TableHead>Provider</TableHead>
															<TableHead>Function</TableHead>
															<TableHead>Credentials</TableHead>
															<TableHead className="w-[80px]"></TableHead>
														</TableRow>
													</TableHeader>
													<TableBody>
														{enabledFunctions.map((func, index) => (
															<TableRow
																key={`${func.functionControllerTypeId}-${func.providerInstanceId}`}
																index={index}
																onClick={() => handleSelectFunction(func)}
															>
																<TableCell bold>
																	{func.providerControllerName}
																</TableCell>
																<TableCell>
																	{func.functionControllerName}
																</TableCell>
																<TableCell>
																	<Badge variant="secondary">
																		{func.providerInstanceName}
																	</Badge>
																</TableCell>
																<TableCell>
																	<Button variant="outline" size="sm">
																		Add to server
																	</Button>
																</TableCell>
															</TableRow>
														))}
													</TableBody>
												</Table>
											)}
										</TableContainer>
									</TableWrapper>
								</div>
							)}
						</ScrollArea>
					</>
				) : (
					<>
						{/* Configuration view */}
						<ScrollArea className="flex-1 p-4">
							<div className="space-y-6">
								{/* Selected function info */}
								<div className="p-4 bg-muted rounded-lg">
									<div className="flex items-center gap-2 mb-2">
										<Check className="h-4 w-4 text-green-600" />
										<span className="font-medium">Selected function</span>
									</div>
									<div className="grid grid-cols-2 gap-2 text-sm">
										<div>
											<span className="text-muted-foreground">Provider:</span>{" "}
											{selectedFunction.providerControllerName}
										</div>
										<div>
											<span className="text-muted-foreground">Function:</span>{" "}
											{selectedFunction.functionControllerName}
										</div>
										<div>
											<span className="text-muted-foreground">
												Credentials:
											</span>{" "}
											{selectedFunction.providerInstanceName}
										</div>
									</div>
									<Button
										variant="link"
										className="p-0 h-auto mt-2"
										onClick={() => setSelectedFunction(null)}
									>
										Change selection
									</Button>
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
										The name that will appear in the MCP tool list. Use
										snake_case format.
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
							<Button onClick={handleSubmit} disabled={addFunction.isPending}>
								{addFunction.isPending ? "Adding..." : "Add function"}
							</Button>
						</div>
					</>
				)}
			</div>
		</SlideOutPanel>
	);
}
