import { createFileRoute, Outlet } from "@tanstack/react-router";
import { A2aProvider } from "@/context/a2a";

export const Route = createFileRoute("/agent/$projectId/$agentId")({
	component: AgentLayoutComponent,
});

function AgentLayoutComponent() {
	const { projectId, agentId } = Route.useParams();

	return (
		<A2aProvider projectId={projectId} agentId={agentId}>
			<Outlet />
		</A2aProvider>
	);
}
