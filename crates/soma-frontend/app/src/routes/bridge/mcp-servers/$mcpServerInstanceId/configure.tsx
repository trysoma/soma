"use client";
import {
	createFileRoute,
	useNavigate,
	useParams,
} from "@tanstack/react-router";
import { Pencil, Plus, Trash2 } from "lucide-react";
import { useState } from "react";
import type { components } from "@/@types/openapi";
import { PageHeaderWithAction } from "@/components/layout/page-header-with-action";
import { AddFunctionSlideout } from "@/components/mcp-servers/add-function-slideout";
import { UpdateFunctionSlideout } from "@/components/mcp-servers/update-function-slideout";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { PageLayout } from "@/components/ui/page-layout";
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
import $api from "@/lib/api-client.client";
import { LINKS } from "@/lib/links";
import {
	invalidateMcpInstanceById,
	invalidateMcpInstancesData,
} from "@/lib/query-cache";
import { queryClient } from "@/main";

export const Route = createFileRoute(
	"/bridge/mcp-servers/$mcpServerInstanceId/configure",
)({
	component: ConfigureMcpServerPage,
});

type McpServerInstanceFunction =
	components["schemas"]["McpServerInstanceFunctionSerialized"];

function ConfigureMcpServerPage() {
	const { mcpServerInstanceId } = useParams({
		from: "/bridge/mcp-servers/$mcpServerInstanceId/configure",
	});
	const navigate = useNavigate();

	const [isAddFunctionOpen, setIsAddFunctionOpen] = useState(false);
	const [editingFunction, setEditingFunction] =
		useState<McpServerInstanceFunction | null>(null);
	const [isDeleting, setIsDeleting] = useState(false);

	// Query MCP server instance
	const { data: instanceData, isLoading } = $api.useQuery(
		"get",
		"/api/bridge/v1/mcp-instance/{mcp_server_instance_id}",
		{
			params: {
				path: {
					mcp_server_instance_id: mcpServerInstanceId,
				},
			},
		},
	);

	// Query available providers for metadata
	const { data: availableProvidersData } = $api.useQuery(
		"get",
		"/api/bridge/v1/available-providers",
		{
			params: {
				query: {
					page_size: 1000,
				},
			},
		},
	);

	// Query provider instances for credential info
	const { data: providerInstancesData } = $api.useQuery(
		"get",
		"/api/bridge/v1/provider",
		{
			params: {
				query: {
					page_size: 1000,
				},
			},
		},
	);

	// Delete function mutation
	const deleteFunction = $api.useMutation(
		"delete",
		"/api/bridge/v1/mcp-instance/{mcp_server_instance_id}/function/{function_controller_type_id}/{provider_controller_type_id}/{provider_instance_id}",
	);

	// Delete MCP server mutation
	const deleteMcpServer = $api.useMutation(
		"delete",
		"/api/bridge/v1/mcp-instance/{mcp_server_instance_id}",
	);

	const instance = instanceData;
	const functions = instance?.functions || [];

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

	const handleDelete = async (func: McpServerInstanceFunction) => {
		if (
			!confirm(
				`Are you sure you want to remove "${func.function_name}" from this MCP server?`,
			)
		) {
			return;
		}

		try {
			await deleteFunction.mutateAsync({
				params: {
					path: {
						mcp_server_instance_id: mcpServerInstanceId,
						function_controller_type_id: func.function_controller_type_id,
						provider_controller_type_id: func.provider_controller_type_id,
						provider_instance_id: func.provider_instance_id,
					},
				},
			});
			invalidateMcpInstanceById(queryClient, mcpServerInstanceId);
		} catch (error) {
			console.error("Failed to delete function:", error);
			alert("Failed to delete function. Please try again.");
		}
	};

	const handleAddSuccess = () => {
		setIsAddFunctionOpen(false);
		invalidateMcpInstanceById(queryClient, mcpServerInstanceId);
	};

	const handleUpdateSuccess = () => {
		setEditingFunction(null);
		invalidateMcpInstanceById(queryClient, mcpServerInstanceId);
	};

	const handleDeleteMcpServer = async () => {
		if (!instance) return;

		if (
			!confirm(
				`Are you sure you want to delete the MCP server "${instance.name}"? This will remove all configured functions.`,
			)
		) {
			return;
		}

		setIsDeleting(true);
		try {
			await deleteMcpServer.mutateAsync({
				params: {
					path: {
						mcp_server_instance_id: mcpServerInstanceId,
					},
				},
			});
			invalidateMcpInstancesData(queryClient);
			navigate({ to: LINKS.BRIDGE_MCP_SERVERS() });
		} catch (error) {
			console.error("Failed to delete MCP server:", error);
			alert("Failed to delete MCP server. Please try again.");
			setIsDeleting(false);
		}
	};

	if (isLoading) {
		return (
			<PageLayout>
				<div className="flex items-center justify-center p-8">
					<p className="text-muted-foreground">Loading...</p>
				</div>
			</PageLayout>
		);
	}

	if (!instance) {
		return (
			<PageLayout>
				<div className="flex items-center justify-center p-8">
					<p className="text-muted-foreground">MCP server not found</p>
				</div>
			</PageLayout>
		);
	}

	return (
		<PageLayout>
			<PageHeaderWithAction
				title={instance.name}
				description="Configure the functions available in this MCP server"
				actions={
					<>
						<Button onClick={() => setIsAddFunctionOpen(true)}>
							<Plus className="h-4 w-4 mr-2" />
							Add function
						</Button>
						<Button
							variant="destructive"
							onClick={handleDeleteMcpServer}
							disabled={isDeleting}
						>
							<Trash2 className="h-4 w-4 mr-2" />
							{isDeleting ? "Deleting..." : "Delete MCP Server"}
						</Button>
					</>
				}
			/>
			<div className="p-6">
				{/* Functions table */}
				<TableWrapper>
					<TableTitle>MCP Server Functions</TableTitle>
					<TableContainer>
						{functions.length > 0 ? (
							<Table>
								<TableHeader sticky>
									<TableRow>
										<TableHead>Provider</TableHead>
										<TableHead>Function</TableHead>
										<TableHead>Credentials</TableHead>
										<TableHead>MCP Function Name</TableHead>
										<TableHead>Description</TableHead>
										<TableHead className="w-[100px]">Actions</TableHead>
									</TableRow>
								</TableHeader>
								<TableBody>
									{functions.map((func, index) => (
										<TableRow
											key={`${func.function_controller_type_id}-${func.provider_instance_id}`}
											index={index}
										>
											<TableCell bold>
												{getProviderControllerName(
													func.provider_controller_type_id,
												)}
											</TableCell>
											<TableCell>
												{getFunctionControllerName(
													func.function_controller_type_id,
													func.provider_controller_type_id,
												)}
											</TableCell>
											<TableCell>
												<Badge variant="secondary">
													{getProviderInstanceName(func.provider_instance_id)}
												</Badge>
											</TableCell>
											<TableCell>{func.function_name}</TableCell>
											<TableCell className="max-w-[200px] truncate">
												{func.function_description || (
													<span className="text-muted-foreground">-</span>
												)}
											</TableCell>
											<TableCell>
												<div className="flex items-center gap-1">
													<Button
														variant="ghost"
														size="icon"
														onClick={() => setEditingFunction(func)}
													>
														<Pencil className="h-4 w-4" />
													</Button>
													<Button
														variant="ghost"
														size="icon"
														onClick={() => handleDelete(func)}
														disabled={deleteFunction.isPending}
													>
														<Trash2 className="h-4 w-4 text-destructive" />
													</Button>
												</div>
											</TableCell>
										</TableRow>
									))}
								</TableBody>
							</Table>
						) : (
							<TableEmpty>
								No functions configured yet. Click "Add function" to get
								started.
							</TableEmpty>
						)}
					</TableContainer>
				</TableWrapper>
			</div>

			{/* Add Function Slideout */}
			<AddFunctionSlideout
				isOpen={isAddFunctionOpen}
				onClose={() => setIsAddFunctionOpen(false)}
				onSuccess={handleAddSuccess}
				mcpServerInstanceId={mcpServerInstanceId}
			/>

			{/* Update Function Slideout */}
			{editingFunction && (
				<UpdateFunctionSlideout
					isOpen={!!editingFunction}
					onClose={() => setEditingFunction(null)}
					onSuccess={handleUpdateSuccess}
					mcpServerInstanceId={mcpServerInstanceId}
					functionData={editingFunction}
				/>
			)}
		</PageLayout>
	);
}
