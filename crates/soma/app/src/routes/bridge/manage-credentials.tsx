"use client";
import { createFileRoute, useNavigate, Outlet } from '@tanstack/react-router'
import { PageLayout } from "@/components/ui/page-layout";
import { PageHeaderWithAction } from "@/components/layout/page-header-with-action";
import { cn } from "@/lib/utils";
import { LINKS } from "@/lib/links";
import $api from '@/lib/api-client.client';
import type { components } from "@/@types/openapi";
import {
	Table,
	TableWrapper,
	TableTitle,
	TableContainer,
	TableEmpty,
	TableHeader,
	TableBody,
	TableRow,
	TableHead,
	TableCell,
} from "@/components/ui/table";

export const Route = createFileRoute('/bridge/manage-credentials')({
  component: RouteComponent,
})

function RouteComponent() {
  return (
    <>
      <ManageCredentialsPage />
      <Outlet />
    </>
  )
}

// Type aliases for better readability
type ProviderController = components["schemas"]["ProviderControllerSerialized"];
type ProviderInstance = components["schemas"]["ProviderInstanceSerialized"];

interface ProviderInstanceWithMetadata extends ProviderInstance {
	providerController?: ProviderController;
	credentialControllerName?: string;
}

// Provider Instances table component
const ProviderInstancesTable = ({
	instances,
	onRowClick,
}: {
	instances: ProviderInstanceWithMetadata[];
	onRowClick: (instance: ProviderInstanceWithMetadata) => void;
}) => {
	return (
		<TableWrapper>
			<TableTitle>Enabled Provider Credentials</TableTitle>
			<TableContainer>
				{instances.length > 0 ? (
					<Table>
						<TableHeader sticky>
							<TableRow>
								<TableHead>Display Name</TableHead>
								<TableHead>Provider</TableHead>
								<TableHead>Credential Type</TableHead>
								<TableHead>Status</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{instances.map((instance, index) => (
								<TableRow
									key={instance.id}
									index={index}
									onClick={() => onRowClick(instance)}
								>
									<TableCell bold>{instance.display_name}</TableCell>
									<TableCell>
										{instance.providerController?.name || instance.provider_controller_type_id}
									</TableCell>
									<TableCell>
										{instance.credentialControllerName || instance.credential_controller_type_id}
									</TableCell>
									<TableCell>
										<span className={cn(
											"px-2 py-1 rounded text-xs",
											instance.status === "active" ? "bg-green-100 text-green-800" : "bg-yellow-100 text-yellow-800"
										)}>
											{instance.status}
										</span>
									</TableCell>
								</TableRow>
							))}
						</TableBody>
					</Table>
				) : (
					<TableEmpty>No credentials configured yet</TableEmpty>
				)}
			</TableContainer>
		</TableWrapper>
	)
};

function ManageCredentialsPage() {
	const navigate = useNavigate();

	// Query provider instances
	const {
		data: providerInstancesData,
		isLoading: isLoadingInstances,
	} = $api.useQuery("get", "/api/bridge/v1/provider", {
		params: {
			query: {
				page_size: 1000,
			},
		},
	})

	// Query available providers to get metadata
	const {
		data: availableProvidersData,
	} = $api.useQuery("get", "/api/bridge/v1/available-providers", {
		params: {
			query: {
				page_size: 1000,
			},
		},
	})

	// Enrich provider instances with metadata
	const providerInstances: ProviderInstanceWithMetadata[] = (providerInstancesData?.items || []).map((instance) => {
		const providerController = availableProvidersData?.items.find(
			(p) => p.type_id === instance.provider_controller_type_id
		)
		const credentialControllerName = providerController?.credential_controllers.find(
			(c) => c.type_id === instance.credential_controller_type_id
		)?.name;

		return {
			...instance,
			providerController,
			credentialControllerName,
		}
	})

	const handleRowClick = (instance: ProviderInstanceWithMetadata) => {
		navigate({ to: LINKS.BRIDGE_MANAGE_CREDENTIALS_DOCUMENTATION(instance.id) })
	}

	if (isLoadingInstances) {
		return (
			<PageLayout>
				<div className="flex items-center justify-center p-8">
					<p className="text-muted-foreground">Loading...</p>
				</div>
			</PageLayout>
		)
	}

	return (
		<PageLayout>
			<PageHeaderWithAction
				title="Manage Credentials"
				description="View and manage your configured provider credentials"
			/>
			<div className="p-6">
				<ProviderInstancesTable
					instances={providerInstances}
					onRowClick={handleRowClick}
				/>
			</div>
		</PageLayout>
	)
}
