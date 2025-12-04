import { createFileRoute, Outlet, useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import { SubNavigation } from "@/components/layout/sub-navigation";
import { useAgentList } from "@/hooks/useAgentList";
import { LINKS } from "@/lib/links";

export const Route = createFileRoute("/a2a")({
	component: LayoutComponent,
});

function LayoutComponent() {
	const { agents, isLoading } = useAgentList();
	const navigate = useNavigate();

	// Create agent items for third-level navigation
	const agentItems = agents.map((agent) => ({
		label: agent.agent_id,
		href: LINKS.A2A_AGENT(agent.project_id, agent.agent_id),
	}));

	// Redirect to first agent when agents are loaded and we're on the base /a2a route
	useEffect(() => {
		if (!isLoading && agents.length > 0) {
			const currentPath = window.location.pathname;
			// Only redirect if we're on /a2a or /a2a/ exactly
			if (currentPath === "/a2a" || currentPath === "/a2a/") {
				const firstAgent = agents[0];
				navigate({
					to: LINKS.A2A_AGENT(firstAgent.project_id, firstAgent.agent_id),
				});
			}
		}
	}, [isLoading, agents, navigate]);

	return (
		<>
			{/* Third-level navigation: Agent list */}
			{!isLoading && agentItems.length > 0 && (
				<SubNavigation items={agentItems} nestLevel="third" />
			)}
			{isLoading && (
				<div className="h-[34px] border-b bg-card flex items-center px-4 text-sm text-muted-foreground">
					Loading agents...
				</div>
			)}
			{!isLoading && agentItems.length === 0 && (
				<div className="h-[34px] border-b bg-card flex items-center px-4 text-sm text-muted-foreground">
					No agents registered
				</div>
			)}
			<Outlet />
		</>
	);
}
