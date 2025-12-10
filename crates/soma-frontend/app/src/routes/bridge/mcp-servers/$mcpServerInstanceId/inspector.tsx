"use client";
import { createFileRoute, useParams } from "@tanstack/react-router";
import { MCPInspector } from "@/components/mcp-inspector/mcp-inspector";

export const Route = createFileRoute(
	"/bridge/mcp-servers/$mcpServerInstanceId/inspector",
)({
	component: McpServerInspectorPage,
});

function McpServerInspectorPage() {
	const { mcpServerInstanceId } = useParams({
		from: "/bridge/mcp-servers/$mcpServerInstanceId/inspector",
	});

	return <MCPInspector mcpServerInstanceId={mcpServerInstanceId} />;
}
