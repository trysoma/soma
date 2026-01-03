"use client";
import { createFileRoute, useParams } from "@tanstack/react-router";
import { useMemo } from "react";
import { MarkdownDocumentation } from "@/components/bridge/markdown-documentation";
import $api from "@/lib/api-client.client";

export const Route = createFileRoute(
	"/bridge/enable-functions/available/$functionControllerId/function_documentation",
)({
	component: RouteComponent,
});

function RouteComponent() {
	const { functionControllerId } = useParams({
		from: "/bridge/enable-functions/available/$functionControllerId/function_documentation",
	});

	// Query available providers
	const { data: availableProviders } = $api.useQuery(
		"get",
		"/api/mcp/v1/available-providers",
		{
			params: {
				query: {
					page_size: 1000,
				},
			},
		},
	);

	// Find the function
	const func = useMemo(() => {
		if (!availableProviders?.items) return null;

		for (const prov of availableProviders.items) {
			const fn = prov.functions.find((f) => f.type_id === functionControllerId);
			if (fn) {
				return {
					id: fn.type_id,
					name: fn.name,
					documentation: fn.documentation,
					parameters: fn.parameters,
					output: fn.output,
				};
			}
		}
		return null;
	}, [availableProviders, functionControllerId]);

	if (!func) {
		return null;
	}

	return (
		<div className="p-6 mt-0">
			<div className="space-y-4">
				<div>
					<h3 className="font-semibold mb-2">Documentation</h3>
					<MarkdownDocumentation content={func.documentation} />
				</div>
				<div>
					<h3 className="font-semibold mb-2">Parameters Schema</h3>
					<pre className="bg-gray-100 p-4 rounded text-sm overflow-auto">
						{JSON.stringify(func.parameters, null, 2)}
					</pre>
				</div>
				<div>
					<h3 className="font-semibold mb-2">Output Schema</h3>
					<pre className="bg-gray-100 p-4 rounded text-sm overflow-auto">
						{JSON.stringify(func.output, null, 2)}
					</pre>
				</div>
			</div>
		</div>
	);
}
