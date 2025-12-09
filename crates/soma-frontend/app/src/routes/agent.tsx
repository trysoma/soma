import { createFileRoute, Outlet, useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import { SubNavigation } from "@/components/layout/sub-navigation";
import { useAgentList } from "@/hooks/useAgentList";
import { LINKS } from "@/lib/links";

export const Route = createFileRoute("/agent")({
	component: LayoutComponent,
});

function LayoutComponent() {
	const { agents, isLoading } = useAgentList();
	const navigate = useNavigate();

	// Create agent items for second-level navigation
	const agentItems = agents.map((agent) => ({
		label: agent.agent_id,
		href: LINKS.AGENT_A2A_OVERVIEW(agent.project_id, agent.agent_id),
	}));

	// Redirect to first agent when agents are loaded and we're on the base /agent route
	useEffect(() => {
		if (!isLoading && agents.length > 0) {
			const currentPath = window.location.pathname;
			// Only redirect if we're on /agent or /agent/ exactly
			if (currentPath === "/agent" || currentPath === "/agent/") {
				const firstAgent = agents[0];
				navigate({
					to: LINKS.AGENT_A2A_OVERVIEW(
						firstAgent.project_id,
						firstAgent.agent_id,
					),
				});
			}
		}
	}, [isLoading, agents, navigate]);

	return (
		<>
			{/* Second-level navigation: Agent list */}
			{!isLoading && agentItems.length > 0 && (
				<SubNavigation items={agentItems} nestLevel="second" />
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
