import { createFileRoute, Outlet } from "@tanstack/react-router";
import { SubNavigation } from "@/components/layout/sub-navigation";
import { LINKS } from "@/lib/links";

export const Route = createFileRoute("/bridge")({
	component: LayoutComponent,
});

function LayoutComponent() {
	return (
		<>
			<SubNavigation
				items={[
					{
						label: "Enable functions",
						href: LINKS.BRIDGE_ENABLE_FUNCTIONS(),
					},
					{
						label: "Manage credentials",
						href: LINKS.BRIDGE_MANAGE_CREDENTIALS(),
					},
					{
						label: "MCP Servers",
						href: LINKS.BRIDGE_MCP_SERVERS(),
					},
				]}
				nestLevel="second"
			/>
			<Outlet />
		</>
	);
}
