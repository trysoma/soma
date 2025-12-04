import { createFileRoute, Outlet } from "@tanstack/react-router";
import { SubNavigation } from "@/components/layout/sub-navigation";
import { A2aProvider } from "@/context/a2a";
import { LINKS } from "@/lib/links";

export const Route = createFileRoute("/a2a/agent/$projectId/$agentId")({
	component: AgentLayoutComponent,
});

function AgentLayoutComponent() {
	const { projectId, agentId } = Route.useParams();

	return (
		<A2aProvider projectId={projectId} agentId={agentId}>
			<SubNavigation
				items={[
					{
						label: "Overview",
						href: LINKS.A2A_AGENT(projectId, agentId),
					},
					{
						label: "Chat",
						href: LINKS.A2A_AGENT_CHAT(projectId, agentId),
					},
				]}
				nestLevel="second"
			/>
			<Outlet />
		</A2aProvider>
	);
}
