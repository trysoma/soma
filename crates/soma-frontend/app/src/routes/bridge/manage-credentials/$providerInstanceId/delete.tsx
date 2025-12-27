"use client";
import {
	createFileRoute,
	useNavigate,
	useParams,
} from "@tanstack/react-router";
import { Trash2 } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import $api from "@/lib/api-client.client";
import { LINKS } from "@/lib/links";
import { invalidateDataByProviderInstanceId } from "@/lib/query-cache";
import { queryClient } from "@/main";

export const Route = createFileRoute(
	"/bridge/manage-credentials/$providerInstanceId/delete",
)({
	component: RouteComponent,
});

function RouteComponent() {
	const { providerInstanceId } = useParams({
		from: "/bridge/manage-credentials/$providerInstanceId/delete",
	});
	const navigate = useNavigate();
	const [isDeleting, setIsDeleting] = useState(false);

	// Query the specific provider instance with all its details
	const { data: providerInstanceData } = $api.useQuery(
		"get",
		"/api/mcp/v1/provider/{provider_instance_id}",
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

	// Delete mutation
	const deleteMutation = $api.useMutation(
		"delete",
		"/api/mcp/v1/provider/{provider_instance_id}",
	);

	const handleDelete = async () => {
		if (!instance) return;

		setIsDeleting(true);

		try {
			await deleteMutation.mutateAsync({
				params: {
					path: {
						provider_instance_id: providerInstanceId,
					},
				},
			});
			invalidateDataByProviderInstanceId(queryClient, providerInstanceId);
			// Redirect to manage credentials page on success
			navigate({ to: LINKS.BRIDGE_MANAGE_CREDENTIALS() });
		} catch (error) {
			console.error("Failed to delete credentials:", error);
			alert("Failed to delete credentials. Please try again.");
			setIsDeleting(false);
		}
	};

	if (!instance || !providerController) {
		return null;
	}

	return (
		<div className="p-6 mt-0">
			<div className="space-y-6">
				<div className="space-y-2">
					<h3 className="font-semibold text-lg">Delete Credentials</h3>
					<p className="text-sm text-muted-foreground">
						This will permanently delete these credentials and all associated
						configurations.
					</p>
				</div>

				<div className="border border-destructive/50 rounded-lg p-4 bg-destructive/5">
					<div className="space-y-4">
						<div className="flex items-start gap-3">
							<Trash2 className="h-5 w-5 text-destructive mt-0.5" />
							<div className="space-y-2">
								<h4 className="font-medium text-destructive">Warning</h4>
								<p className="text-sm text-muted-foreground">
									Deleting these credentials will:
								</p>
								<ul className="list-disc list-inside text-sm text-muted-foreground space-y-1 ml-2">
									<li>Permanently remove all credential configurations</li>
									<li>
										Disable all functions currently using these credentials
									</li>
									<li>Remove all stored authentication data</li>
								</ul>
								<p className="text-sm font-medium text-destructive mt-3">
									This action cannot be undone.
								</p>
							</div>
						</div>

						<div className="pt-2">
							<Button
								variant="destructive"
								onClick={handleDelete}
								disabled={isDeleting}
								className="w-full sm:w-auto"
							>
								<Trash2 className="h-4 w-4 mr-2" />
								{isDeleting
									? "Deleting..."
									: `Delete "${instance.display_name}"`}
							</Button>
						</div>
					</div>
				</div>

				<div className="text-xs text-muted-foreground">
					<p>Provider: {providerController.name}</p>
					<p>ID: {instance.id}</p>
					<p>Status: {instance.status}</p>
				</div>
			</div>
		</div>
	);
}
