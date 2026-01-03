import { useMemo, useState } from "react";
import type { components } from "@/@types/openapi";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import $api from "@/lib/api-client.client";
import {
	ConfigurationForm,
	LinkProviderInstancesTable,
} from "./configuration-form";

type ProviderController = components["schemas"]["ProviderControllerSerialized"];

interface LinkProviderOrCreateProps {
	provider: ProviderController;
	functionTypeId: string;
	onSuccess?: () => void;
}

// Component that shows tabs to either link to existing provider or create new one
export const LinkProviderOrCreate = ({
	provider,
	functionTypeId,
	onSuccess,
}: LinkProviderOrCreateProps) => {
	const [configMode, setConfigMode] = useState<"existing" | "new">("existing");

	// Query existing provider instances for this provider type (status=active)
	const { data: providerInstancesData, isLoading: isLoadingInstances } =
		$api.useQuery("get", "/api/mcp/v1/provider", {
			params: {
				query: {
					page_size: 1000,
					status: "active",
				},
			},
		});

	// Filter instances by provider controller type
	const existingProviderInstances = useMemo(() => {
		if (!providerInstancesData?.items) return [];
		return providerInstancesData.items.filter(
			(instance) => instance.provider_controller_type_id === provider.type_id,
		);
	}, [providerInstancesData, provider.type_id]);

	const hasExistingCredentials = existingProviderInstances.length > 0;

	// If loading, show loading state
	if (isLoadingInstances) {
		return (
			<div className="flex items-center justify-center p-8">
				<p className="text-muted-foreground">Loading...</p>
			</div>
		);
	}

	// If no existing credentials, show only the "Add New" form
	if (!hasExistingCredentials) {
		return (
			<div className="space-y-4">
				<ConfigurationForm provider={provider} onSuccess={onSuccess} />
			</div>
		);
	}

	// If we have existing credentials, show tabs for "Use Existing" and "Add New"
	return (
		<div className="space-y-4">
			<Tabs
				value={configMode}
				onValueChange={(v) => setConfigMode(v as "existing" | "new")}
			>
				<TabsList className="grid w-fit grid-cols-2">
					<TabsTrigger value="existing">Use existing credentials</TabsTrigger>
					<TabsTrigger value="new">Add new credentials</TabsTrigger>
				</TabsList>

				<TabsContent value="existing" className="mt-4">
					<LinkProviderInstancesTable
						instances={existingProviderInstances}
						provider={provider}
						functionTypeId={functionTypeId}
						onSuccess={onSuccess}
					/>
				</TabsContent>

				<TabsContent value="new" className="mt-4">
					<ConfigurationForm provider={provider} onSuccess={onSuccess} />
				</TabsContent>
			</Tabs>
		</div>
	);
};
