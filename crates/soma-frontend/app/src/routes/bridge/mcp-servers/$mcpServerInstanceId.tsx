import { createFileRoute, Outlet } from "@tanstack/react-router";

export const Route = createFileRoute(
	"/bridge/mcp-servers/$mcpServerInstanceId",
)({
	component: McpServerInstanceLayout,
});

function McpServerInstanceLayout() {
	return <Outlet />;
}
