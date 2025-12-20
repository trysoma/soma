"use client";
import {
	createFileRoute,
	Outlet,
	useNavigate,
	useParams,
} from "@tanstack/react-router";
import { Plus, Server } from "lucide-react";
import { useEffect, useState } from "react";
import type { components } from "@/@types/openapi";
import { SubNavigationDropdown } from "@/components/layout/sub-navigation-dropdown";
import { CreateMcpServerDialog } from "@/components/mcp-servers/create-mcp-server-dialog";
import { Button } from "@/components/ui/button";
import { PageLayout } from "@/components/ui/page-layout";
import $api from "@/lib/api-client.client";
import { LINKS } from "@/lib/links";

export const Route = createFileRoute("/bridge/mcp-servers")({
	component: McpServersLayout,
});

type McpServerInstance =
	components["schemas"]["McpServerInstanceSerializedWithFunctions"];

function McpServersLayout() {
	const navigate = useNavigate();
	const params = useParams({ strict: false });
	const selectedInstanceId = (params as { mcpServerInstanceId?: string })
		.mcpServerInstanceId;

	const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);

	// Query MCP server instances
	const {
		data: mcpInstancesData,
		isLoading,
		refetch,
	} = $api.useQuery("get", "/api/bridge/v1/mcp-server", {
		params: {
			query: {
				page_size: 1000,
			},
		},
	});

	const instances: McpServerInstance[] = mcpInstancesData?.items || [];

	// Create dropdown items
	const instanceItems = instances.map((instance) => ({
		label: `${instance.name} (${instance.functions.length} functions)`,
		value: instance.id,
		href: LINKS.BRIDGE_MCP_SERVER_CONFIGURE(instance.id),
	}));

	// Navigation items (only shown when an instance is selected)
	const navItems = selectedInstanceId
		? [
				{
					label: "Configure",
					href: LINKS.BRIDGE_MCP_SERVER_CONFIGURE(selectedInstanceId),
				},
				{
					label: "MCP Inspector",
					href: LINKS.BRIDGE_MCP_SERVER_INSPECTOR(selectedInstanceId),
				},
			]
		: undefined;

	// Redirect to first instance when loaded and we're on the base route
	useEffect(() => {
		if (!isLoading && instances.length > 0) {
			const currentPath = window.location.pathname;
			// Only redirect if we're on /bridge/mcp-servers exactly
			if (
				currentPath === "/bridge/mcp-servers" ||
				currentPath === "/bridge/mcp-servers/"
			) {
				navigate({
					to: LINKS.BRIDGE_MCP_SERVER_CONFIGURE(instances[0].id),
				});
			}
		}
	}, [isLoading, instances, navigate]);

	const handleCreateSuccess = (newInstance: McpServerInstance) => {
		refetch();
		setIsCreateDialogOpen(false);
		navigate({
			to: LINKS.BRIDGE_MCP_SERVER_CONFIGURE(newInstance.id),
		});
	};

	// Show empty state when no MCP server is selected
	const showEmptyState =
		!isLoading && !selectedInstanceId && instances.length === 0;
	const showSelectPrompt =
		!isLoading && !selectedInstanceId && instances.length > 0;

	return (
		<>
			{/* Second-level navigation: MCP server dropdown with create button and nav links */}
			<SubNavigationDropdown
				items={instanceItems}
				selectedValue={selectedInstanceId}
				placeholder="Select MCP server"
				isLoading={isLoading}
				emptyMessage="No MCP servers configured"
				nestLevel="second"
				onCreateNew={() => setIsCreateDialogOpen(true)}
				createLabel="Create new"
				navItems={navItems}
			/>

			{showEmptyState ? (
				<PageLayout>
					<div className="flex flex-col items-center justify-center py-16 text-center">
						<Server className="h-12 w-12 text-muted-foreground mb-4" />
						<h3 className="text-lg font-semibold mb-2">
							No MCP servers configured
						</h3>
						<p className="text-muted-foreground mb-6 max-w-md">
							Create an MCP server to expose your enabled functions to AI agents
							and tools.
						</p>
						<Button onClick={() => setIsCreateDialogOpen(true)}>
							<Plus className="h-4 w-4 mr-2" />
							Create MCP Server
						</Button>
					</div>
				</PageLayout>
			) : showSelectPrompt ? (
				<PageLayout>
					<div className="flex flex-col items-center justify-center py-16 text-center">
						<Server className="h-12 w-12 text-muted-foreground mb-4" />
						<h3 className="text-lg font-semibold mb-2">Select an MCP server</h3>
						<p className="text-muted-foreground max-w-md">
							Choose an MCP server from the dropdown above to view and configure
							its functions.
						</p>
					</div>
				</PageLayout>
			) : (
				<Outlet />
			)}

			<CreateMcpServerDialog
				isOpen={isCreateDialogOpen}
				onClose={() => setIsCreateDialogOpen(false)}
				onSuccess={handleCreateSuccess}
			/>
		</>
	);
}
