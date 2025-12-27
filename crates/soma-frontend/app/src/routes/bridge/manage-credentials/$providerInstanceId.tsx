"use client";
import {
	createFileRoute,
	Link,
	useLocation,
	useNavigate,
	useParams,
} from "@tanstack/react-router";
import { Package } from "lucide-react";
import { SlideOutPanel } from "@/components/ui/slide-out-panel";
import { SlideOutTabs } from "@/components/ui/slide-out-tabs";
import $api from "@/lib/api-client.client";
import { LINKS } from "@/lib/links";

export const Route = createFileRoute(
	"/bridge/manage-credentials/$providerInstanceId",
)({
	component: RouteComponent,
});

function RouteComponent() {
	const { providerInstanceId } = useParams({
		from: "/bridge/manage-credentials/$providerInstanceId",
	});
	const navigate = useNavigate();
	const location = useLocation();

	// Query the specific provider instance with all its details
	const { data: providerInstanceData, isLoading: isLoadingInstance } =
		$api.useQuery("get", "/api/mcp/v1/provider/{provider_instance_id}", {
			params: {
				path: {
					provider_instance_id: providerInstanceId,
				},
			},
		});

	const instance = providerInstanceData?.provider_instance;
	const providerController = providerInstanceData?.controller;

	const handleClose = () => {
		navigate({ to: LINKS.BRIDGE_MANAGE_CREDENTIALS() });
	};

	// Determine current tab from pathname
	const getCurrentTab = () => {
		if (location.pathname.includes("/documentation")) return "documentation";
		if (location.pathname.includes("/configuration")) return "configuration";
		if (location.pathname.includes("/functions")) return "functions";
		if (location.pathname.includes("/delete")) return "delete";
		return "documentation";
	};

	if (isLoadingInstance || !instance || !providerController) {
		return null; // Don't show anything while loading
	}

	const tabs = [
		{
			value: "documentation",
			label: "Provider Documentation",
			pathPattern: "/documentation",
			component: (
				<Link
					to={LINKS.BRIDGE_MANAGE_CREDENTIALS_DOCUMENTATION(providerInstanceId)}
				>
					Provider Documentation
				</Link>
			),
		},
		{
			value: "configuration",
			label: "Modify Configuration",
			pathPattern: "/configuration",
			component: (
				<Link
					to={LINKS.BRIDGE_MANAGE_CREDENTIALS_CONFIGURATION(providerInstanceId)}
				>
					Modify Configuration
				</Link>
			),
		},
		{
			value: "functions",
			label: "Enabled Functions",
			pathPattern: "/functions",
			component: (
				<Link
					to={LINKS.BRIDGE_MANAGE_CREDENTIALS_FUNCTIONS(providerInstanceId)}
				>
					Enabled Functions
				</Link>
			),
		},
		{
			value: "delete",
			label: "Delete",
			pathPattern: "/delete",
			component: (
				<Link to={LINKS.BRIDGE_MANAGE_CREDENTIALS_DELETE(providerInstanceId)}>
					Delete
				</Link>
			),
		},
	];

	return (
		<SlideOutPanel
			onClose={handleClose}
			title={instance.display_name}
			subtitle={providerController.name}
			icon={<Package className="h-5 w-5" />}
		>
			<SlideOutTabs
				tabs={tabs}
				getCurrentTab={getCurrentTab}
				className="grid-cols-4"
			/>
		</SlideOutPanel>
	);
}
