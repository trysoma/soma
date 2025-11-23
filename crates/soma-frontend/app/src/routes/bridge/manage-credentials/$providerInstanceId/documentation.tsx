"use client";
import { createFileRoute, useParams } from "@tanstack/react-router";
import { MarkdownDocumentation } from "@/components/bridge/markdown-documentation";
import $api from "@/lib/api-client.client";

export const Route = createFileRoute(
	"/bridge/manage-credentials/$providerInstanceId/documentation",
)({
	component: RouteComponent,
});

function RouteComponent() {
	const { providerInstanceId } = useParams({
		from: "/bridge/manage-credentials/$providerInstanceId/documentation",
	});

	// Query the specific provider instance with all its details
	const { data: providerInstanceData } = $api.useQuery(
		"get",
		"/api/bridge/v1/provider/{provider_instance_id}",
		{
			params: {
				path: {
					provider_instance_id: providerInstanceId,
				},
			},
		},
	);

	const instance = providerInstanceData?.provider_instance;
	const providerController = providerInstanceData?.controller;

	if (!instance || !providerController) {
		return null;
	}

	return (
		<div className="p-6 mt-0">
			<div className="space-y-4">
				<div>
					<h3 className="font-semibold mb-2">Provider Documentation</h3>
					<MarkdownDocumentation content={providerController.documentation} />
				</div>
				<div>
					<h3 className="font-semibold mb-2">Available Credentials</h3>
					<div className="space-y-2">
						{providerController.credential_controllers.map((cred) => (
							<div key={cred.type_id} className="border rounded p-3">
								<p className="font-medium">{cred.name}</p>
								<p className="text-sm text-muted-foreground">
									{cred.documentation}
								</p>
							</div>
						))}
					</div>
				</div>
			</div>
		</div>
	);
}
