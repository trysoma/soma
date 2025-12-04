import { createFileRoute } from "@tanstack/react-router";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { PageHeader } from "@/components/ui/page-header";
import { PageLayout } from "@/components/ui/page-layout";

export const Route = createFileRoute("/a2a/")({
	component: RouteComponent,
});

function RouteComponent() {
	// This page is shown when there are no agents registered or during loading
	// The a2a.tsx layout will redirect to the first agent when available
	return (
		<PageLayout className="py-6">
			<div className="space-y-6">
				<PageHeader
					title="Agent 2 Agent"
					description="Manage and interact with your registered agents via the Agent 2 Agent protocol."
				/>
				<Card>
					<CardHeader>
						<CardTitle>No agents registered</CardTitle>
						<CardDescription>
							Register an agent in your SDK to get started. Once registered,
							agents will appear in the navigation above.
						</CardDescription>
					</CardHeader>
					<CardContent>
						<p className="text-sm text-muted-foreground">
							To register an agent, use the <code>soma.registerAgent()</code>{" "}
							method in your SDK code.
						</p>
					</CardContent>
				</Card>
			</div>
		</PageLayout>
	);
}
