import { createFileRoute, Outlet } from "@tanstack/react-router";
import { SubNavigation } from "@/components/layout/sub-navigation";
import { A2aProvider } from "@/context/a2a";
import { LINKS } from "@/lib/links";

export const Route = createFileRoute("/agent/$projectId/$agentId")({
	component: AgentLayoutComponent,
});

function AgentLayoutComponent() {
	const { projectId, agentId } = Route.useParams();

	return (
		<A2aProvider projectId={projectId} agentId={agentId}>
			<SubNavigation
				items={[
					{
						label: "A2A Overview",
						href: LINKS.AGENT_A2A_OVERVIEW(projectId, agentId),
					},
					{
						label: "A2A Chat Debugger",
						href: LINKS.AGENT_A2A_CHAT_DEBUGGER(projectId, agentId),
					},
				]}
				nestLevel="third"
			/>
			<Outlet />
		</A2aProvider>
	);
}
