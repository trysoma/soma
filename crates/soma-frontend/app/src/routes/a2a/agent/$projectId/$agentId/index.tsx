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

export const Route = createFileRoute("/a2a/agent/$projectId/$agentId/")({
	component: RouteComponent,
});

function RouteComponent() {
	const { projectId, agentId } = Route.useParams();
	const baseUrl = window.location.origin;

	const agentCardPath = `/api/a2a/v1/${projectId}/${agentId}/.well-known/agent.json`;
	const agentSsePath = `/api/a2a/v1/${projectId}/${agentId}`;

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
								<Label>Agent SSE endpoint:</Label>
								<Input
									disabled
									type="text"
									value={`${baseUrl}${agentSsePath}`}
								/>
							</div>
						</div>
					</CardContent>
				</Card>
			</div>
		</PageLayout>
	);
}
