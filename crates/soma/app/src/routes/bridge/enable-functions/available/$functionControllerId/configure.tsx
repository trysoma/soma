"use client";
import {
	createFileRoute,
	Link,
	Outlet,
	useLocation,
	useNavigate,
	useParams,
} from "@tanstack/react-router";
import { useEffect, useMemo } from "react";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import $api from "@/lib/api-client.client";

export const Route = createFileRoute(
	"/bridge/enable-functions/available/$functionControllerId/configure",
)({
	component: RouteComponent,
});

function RouteComponent() {
	const { functionControllerId } = useParams({
		from: "/bridge/enable-functions/available/$functionControllerId/configure",
	});
	const location = useLocation();
	const navigate = useNavigate();

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

	// Query existing provider instances for this provider type (status=active)
	const { data: providerInstancesData } = $api.useQuery(
		"get",
		"/api/bridge/v1/provider",
		{
			params: {
				query: {
					page_size: 1000,
					status: "active",
				},
			},
		},
		{
			enabled: !!provider,
		},
	);

	// Filter instances by provider controller type
	const existingProviderInstances = useMemo(() => {
		if (!providerInstancesData?.items || !provider) return [];
		return providerInstancesData.items.filter(
			(instance) => instance.provider_controller_type_id === provider.type_id,
		);
	}, [providerInstancesData, provider]);

	const hasExistingProviders = existingProviderInstances.length > 0;

	// Determine current tab from pathname
	const getCurrentTab = () => {
		if (location.pathname.includes("/existing")) return "existing";
		if (location.pathname.includes("/new")) return "new";
		return "existing";
	};

	// If no existing providers, redirect to new tab
	useEffect(() => {
		if (
			location.pathname.includes("/configure") &&
			provider &&
			!hasExistingProviders &&
			!location.pathname.includes("/new")
		) {
			navigate({
				to: "/bridge/enable-functions/available/$functionControllerId/configure/new",
				params: { functionControllerId },
			});
		}
	}, [
		provider,
		hasExistingProviders,
		location.pathname,
		navigate,
		functionControllerId,
	]);

	if (!provider) {
		return null;
	}

	return (
		<div className="p-6 mt-0">
			<Tabs value={getCurrentTab()} className="space-y-4">
				<TabsList className="grid w-fit grid-cols-2">
					<TabsTrigger
						value="existing"
						asChild
						className={
							hasExistingProviders
								? "cursor-pointer"
								: "cursor-not-allowed opacity-50"
						}
						disabled={!hasExistingProviders}
					>
						<Link
							to="/bridge/enable-functions/available/$functionControllerId/configure/existing"
							params={{ functionControllerId }}
						>
							Use existing integration
						</Link>
					</TabsTrigger>
					<TabsTrigger value="new" asChild>
						<Link
							to="/bridge/enable-functions/available/$functionControllerId/configure/new"
							params={{ functionControllerId }}
						>
							Add new credentials
						</Link>
					</TabsTrigger>
				</TabsList>

				<Outlet />
			</Tabs>
		</div>
	);
}
