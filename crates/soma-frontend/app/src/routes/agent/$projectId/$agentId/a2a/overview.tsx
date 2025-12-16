import { createFileRoute } from "@tanstack/react-router";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { PageHeader } from "@/components/ui/page-header";
import { PageLayout } from "@/components/ui/page-layout";
import { getAgentA2APath, getAgentCardPath } from "@/context/a2a";

export const Route = createFileRoute("/agent/$projectId/$agentId/a2a/overview")(
	{
		component: RouteComponent,
	},
);

function RouteComponent() {
	const { projectId, agentId } = Route.useParams();
	const baseUrl = window.location.origin;

	const agentCardPath = getAgentCardPath(projectId, agentId);
	const agentA2APath = getAgentA2APath(projectId, agentId);

	return (
		<PageLayout className="py-6">
			<div className="space-y-6">
				<PageHeader
					title={`Agent: ${agentId}`}
					description={`Project: ${projectId}`}
				/>
				<Card>
					<CardHeader>
						<CardTitle>Connectivity information</CardTitle>
						<CardDescription>
							Use the below A2A compliant endpoints to connect to this agent.
						</CardDescription>
					</CardHeader>
					<CardContent>
						<div className="space-y-4">
							<div className="flex flex-col gap-2 max-w-lg">
								<Label>Agent Card endpoint:</Label>
								<Input
									disabled
									type="text"
									value={`${baseUrl}${agentCardPath}`}
								/>
							</div>
							<div className="flex flex-col gap-2 max-w-lg">
								<Label>Agent A2A endpoint:</Label>
								<Input
									disabled
									type="text"
									value={`${baseUrl}${agentA2APath}`}
								/>
							</div>
						</div>
					</CardContent>
				</Card>
			</div>
		</PageLayout>
	);
}
