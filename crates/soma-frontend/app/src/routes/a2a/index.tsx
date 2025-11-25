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
import { DEFAULT_AGENT_CARD_PATH, DEFAULT_AGENT_SSE_PATH } from "@/context/a2a";

export const Route = createFileRoute("/a2a/")({
	component: RouteComponent,
});

function RouteComponent() {
	const baseUrl = window.location.origin;

	return (
		<PageLayout className="py-6">
			<div className="space-y-6">
				<PageHeader
					title="Agent 2 Agent"
					description="Below is information about how to access your agent via the Agent 2 Agent protocol."
				/>
				<Card>
					<CardHeader>
						<CardTitle>Connectivity information</CardTitle>
						<CardDescription>
							Use the below A2A compliant endpoints to connect to your agent.
						</CardDescription>
					</CardHeader>
					<CardContent>
						<div className="space-y-4">
							<div className="flex flex-col gap-2 max-w-lg">
								<Label>Agent Card endpoint:</Label>
								<Input
									disabled
									type="text"
									value={`${baseUrl}${DEFAULT_AGENT_CARD_PATH}`}
								/>
							</div>
							<div className="flex flex-col gap-2 max-w-lg">
								<Label>Agent SSE endpoint:</Label>
								<Input
									disabled
									type="text"
									value={`${baseUrl}${DEFAULT_AGENT_SSE_PATH}`}
								/>
							</div>
						</div>
					</CardContent>
				</Card>
			</div>
		</PageLayout>
	);
}
