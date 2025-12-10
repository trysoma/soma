import {
	createFileRoute,
	Outlet,
	useNavigate,
	useParams,
} from "@tanstack/react-router";
import { Bot } from "lucide-react";
import { useEffect } from "react";
import { SubNavigationDropdown } from "@/components/layout/sub-navigation-dropdown";
import { PageLayout } from "@/components/ui/page-layout";
import { useAgentList } from "@/hooks/useAgentList";
import { LINKS } from "@/lib/links";

export const Route = createFileRoute("/agent")({
	component: LayoutComponent,
});

function LayoutComponent() {
	const { agents, isLoading } = useAgentList();
	const navigate = useNavigate();
	const params = useParams({ strict: false });
	const agentId = (params as { agentId?: string }).agentId;
	const projectId = (params as { projectId?: string }).projectId;

	// Create agent items for dropdown
	const agentItems = agents.map((agent) => ({
		label: agent.agent_id,
		value: agent.agent_id,
		href: LINKS.AGENT_A2A_OVERVIEW(agent.project_id, agent.agent_id),
	}));

	// Navigation items (only shown when an agent is selected)
	const navItems =
		agentId && projectId
			? [
					{
						label: "Overview",
						href: LINKS.AGENT_A2A_OVERVIEW(projectId, agentId),
					},
					{
						label: "Chat Debugger",
						href: LINKS.AGENT_A2A_CHAT_DEBUGGER(projectId, agentId),
					},
				]
			: undefined;

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

	// Show empty state when no agent is selected
	const showEmptyState = !isLoading && !agentId && agents.length === 0;
	const showSelectPrompt = !isLoading && !agentId && agents.length > 0;

	return (
		<>
			{/* Second-level navigation: Agent dropdown with nav links */}
			<SubNavigationDropdown
				items={agentItems}
				selectedValue={agentId}
				placeholder="Select agent"
				isLoading={isLoading}
				emptyMessage="No agents registered"
				nestLevel="second"
				navItems={navItems}
			/>

			{showEmptyState ? (
				<PageLayout>
					<div className="flex flex-col items-center justify-center py-16 text-center">
						<Bot className="h-12 w-12 text-muted-foreground mb-4" />
						<h3 className="text-lg font-semibold mb-2">No agents registered</h3>
						<p className="text-muted-foreground max-w-md">
							Agents will appear here once they connect to Soma. Start your
							agent application to see it listed.
						</p>
					</div>
				</PageLayout>
			) : showSelectPrompt ? (
				<PageLayout>
					<div className="flex flex-col items-center justify-center py-16 text-center">
						<Bot className="h-12 w-12 text-muted-foreground mb-4" />
						<h3 className="text-lg font-semibold mb-2">Select an agent</h3>
						<p className="text-muted-foreground max-w-md">
							Choose an agent from the dropdown above to view its details and
							debug conversations.
						</p>
					</div>
				</PageLayout>
			) : (
				<Outlet />
			)}
		</>
	);
}
