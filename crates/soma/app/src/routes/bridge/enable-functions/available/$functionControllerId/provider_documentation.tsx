"use client";
import { createFileRoute, useParams } from "@tanstack/react-router";
import { useMemo } from "react";
import { MarkdownDocumentation } from "@/components/bridge/markdown-documentation";
import $api from "@/lib/api-client.client";

export const Route = createFileRoute(
	"/bridge/enable-functions/available/$functionControllerId/provider_documentation",
)({
	component: RouteComponent,
});

function RouteComponent() {
	const { functionControllerId } = useParams({
		from: "/bridge/enable-functions/available/$functionControllerId/provider_documentation",
	});

	// Query available providers
	const { data: availableProviders } = $api.useQuery(
		"get",
		"/api/bridge/v1/available-providers",
		{
			params: {
				query: {
					page_size: 1000,
				},
			},
		},
	);

	// Find the provider for this function
	const provider = useMemo(() => {
		if (!availableProviders?.items) return null;

		for (const prov of availableProviders.items) {
			const fn = prov.functions.find((f) => f.type_id === functionControllerId);
			if (fn) {
				return prov;
			}
		}
		return null;
	}, [availableProviders, functionControllerId]);

	if (!provider) {
		return null;
	}

	return (
		<div className="p-6 mt-0">
			<div className="space-y-4">
				<div>
					<h3 className="font-semibold mb-2">Provider Documentation</h3>
					<MarkdownDocumentation content={provider.documentation} />
				</div>
				<div>
					<h3 className="font-semibold mb-2">Available Credentials</h3>
					<div className="space-y-2">
						{provider.credential_controllers.map((cred) => (
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
