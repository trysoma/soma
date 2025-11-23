"use client";
import { createFileRoute, Outlet, useNavigate } from "@tanstack/react-router";
import { X } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { components } from "@/@types/openapi";
import { PageHeaderWithAction } from "@/components/layout/page-header-with-action";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { PageLayout } from "@/components/ui/page-layout";
import {
	SearchableSelect,
	type SelectOption,
} from "@/components/ui/searchable-select";
import {
	Table,
	TableBody,
	TableCell,
	TableContainer,
	TableHead,
	TableHeader,
	TableLoadMore,
	TableRow,
	TableTitle,
	TableWrapper,
} from "@/components/ui/table";
import {
	Tooltip,
	TooltipContent,
	TooltipTrigger,
} from "@/components/ui/tooltip";
import $api from "@/lib/api-client.client";
import { LINKS } from "@/lib/links";

export const Route = createFileRoute("/bridge/enable-functions")({
	component: RouteComponent,
});

function RouteComponent() {
	return (
		<>
			<EnableFunctionsPage />
			<Outlet />
		</>
	);
}

// Type aliases for better readability
type JsonSchema = components["schemas"]["JsonSchema"];
type ProviderInstance =
	components["schemas"]["ProviderInstanceSerializedWithCredentials"];

// Type for available functions derived from providers
export interface AvailableFunction {
	id: string; // function type_id
	providerTypeId: string;
	providerName: string;
	functionName: string;
	documentation: string;
	parametersSchema: JsonSchema;
	outputSchema: JsonSchema;
	categories: string[]; // Provider categories
	providerInstances: ProviderInstance[]; // Provider instances where this function is enabled
}

// Component to render provider instance badges with truncation and tooltip
const ProviderInstanceBadges = ({
	instances,
}: {
	instances: ProviderInstance[];
}) => {
	const MAX_DISPLAY_LENGTH = 50; // Maximum characters to display before truncating
	const MAX_VISIBLE_BADGES = 3; // Show at most 3 badges before "+ X more"

	if (!instances || instances.length === 0) {
		return <span className="text-xs text-muted-foreground">None</span>;
	}

	// Calculate how many badges we can show before hitting the character limit
	let visibleCount = 0;
	let currentLength = 0;

	for (let i = 0; i < instances.length && i < MAX_VISIBLE_BADGES; i++) {
		const badgeLength = instances[i].provider_instance.display_name.length + 4; // +4 for padding/spacing
		if (currentLength + badgeLength <= MAX_DISPLAY_LENGTH) {
			visibleCount++;
			currentLength += badgeLength;
		} else {
			break;
		}
	}

	// Ensure at least 1 badge is visible
	if (visibleCount === 0 && instances.length > 0) {
		visibleCount = 1;
	}

	const visibleInstances = instances.slice(0, visibleCount);
	const hiddenCount = instances.length - visibleCount;
	const allInstanceNames = instances
		.map((i) => i.provider_instance.display_name)
		.join(", ");

	return (
		<div
			className="flex items-center gap-1 flex-wrap"
			onClick={(e) => e.stopPropagation()}
		>
			{visibleInstances.map((instance) => (
				<Badge
					key={instance.provider_instance.id}
					variant="secondary"
					className="text-xs"
				>
					{instance.provider_instance.display_name}
				</Badge>
			))}
			{hiddenCount > 0 && (
				<Tooltip>
					<TooltipTrigger asChild>
						<Badge variant="outline" className="text-xs cursor-help">
							+ {hiddenCount} more
						</Badge>
					</TooltipTrigger>
					<TooltipContent side="top" className="max-w-md">
						<div className="text-xs">{allInstanceNames}</div>
					</TooltipContent>
				</Tooltip>
			)}
		</div>
	);
};

// Functions table component
const FunctionsTable = ({
	functions,
	title,
	onRowClick,
	loadMore,
	hasMore,
}: {
	functions: AvailableFunction[];
	title: string;
	onRowClick: (func: AvailableFunction) => void;
	loadMore?: () => void;
	hasMore?: boolean;
}) => {
	const observerRef = useRef<IntersectionObserver | null>(null);
	const loadMoreRef = useRef<HTMLDivElement | null>(null);

	useEffect(() => {
		if (!loadMore || !hasMore) return;

		observerRef.current = new IntersectionObserver(
			(entries) => {
				if (entries[0].isIntersecting) {
					loadMore();
				}
			},
			{ threshold: 0.1 },
		);

		if (loadMoreRef.current) {
			observerRef.current.observe(loadMoreRef.current);
		}

		return () => {
			if (observerRef.current) {
				observerRef.current.disconnect();
			}
		};
	}, [loadMore, hasMore]);

	return (
		<TableWrapper>
			<TableTitle>{title}</TableTitle>
			<TableContainer maxHeight="max-h-[300px]">
				<Table>
					<TableHeader sticky>
						<TableRow>
							<TableHead>Provider</TableHead>
							<TableHead>Function</TableHead>
							<TableHead>Enabled Credentials</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{functions.map((func, index) => (
							<TableRow
								key={func.id}
								index={index}
								onClick={() => onRowClick(func)}
							>
								<TableCell bold>{func.providerName}</TableCell>
								<TableCell>{func.functionName}</TableCell>
								<TableCell>
									<ProviderInstanceBadges instances={func.providerInstances} />
								</TableCell>
							</TableRow>
						))}
					</TableBody>
				</Table>
				{hasMore && <TableLoadMore loadMoreRef={loadMoreRef} />}
			</TableContainer>
		</TableWrapper>
	);
};

function EnableFunctionsPage() {
	const navigate = useNavigate();

	// Query data encryption keys
	const {
		data: dataEncryptionKeys,
		isLoading: isLoadingKeys,
		refetch: refetchKeys,
	} = $api.useQuery("get", "/api/bridge/v1/encryption/data-encryption-key", {
		params: {
			query: {
				page_size: 100,
			},
		},
	});

	// Check if we have any encryption keys
	const hasEncryptionKeys =
		dataEncryptionKeys?.items && dataEncryptionKeys.items.length > 0;

	const [selectedProviderFilter, setSelectedProviderFilter] = useState("");
	const [selectedCategoryFilter, setSelectedCategoryFilter] = useState("");
	const [providerSearchQuery, setProviderSearchQuery] = useState("");
	const [categorySearchQuery, setCategorySearchQuery] = useState("");

	// Query provider instances grouped by function (only when keys exist)
	// This query uses server-side filtering via provider_controller_type_id and function_category
	const {
		data: functionInstanceData,
		isLoading: isLoadingFunctions,
		isFetching: isFetchingFunctions,
	} = $api.useQuery(
		"get",
		"/api/bridge/v1/provider/grouped-by-function",
		{
			params: {
				query: {
					page_size: 1000,
					...(selectedProviderFilter && {
						provider_controller_type_id: selectedProviderFilter,
					}),
					...(selectedCategoryFilter && {
						function_category: selectedCategoryFilter,
					}),
				},
			},
		},
		{
			enabled: hasEncryptionKeys,
			// Keep previous data while fetching new data to prevent flashing
			placeholderData: (previousData) => previousData,
		},
	);

	// Query available providers to get the list for filter dropdown
	const { data: availableProvidersData } = $api.useQuery(
		"get",
		"/api/bridge/v1/available-providers",
		{
			params: {
				query: {
					page_size: 1000,
				},
			},
		},
		{
			enabled: hasEncryptionKeys,
		},
	);

	// Mutation to create data encryption key
	const createKeyMutation = $api.useMutation(
		"post",
		"/api/bridge/v1/encryption/data-encryption-key",
	);

	// Handler to create default encryption key
	const handleEnableBridge = async () => {
		try {
			await createKeyMutation.mutateAsync({
				body: {
					id: "default",
				},
			});
			// Refetch keys after creation
			await refetchKeys();
		} catch (error) {
			console.error("Failed to create encryption key:", error);
		}
	};

	// Transform function instance data into available functions
	const availableFunctions = useMemo(() => {
		if (!functionInstanceData?.items) return [];

		const functions: AvailableFunction[] = functionInstanceData.items.map(
			(item) => ({
				id: item.function_controller.type_id,
				providerTypeId: item.provider_controller.type_id,
				providerName: item.provider_controller.name,
				functionName: item.function_controller.name,
				documentation: item.function_controller.documentation,
				parametersSchema: item.function_controller.parameters,
				outputSchema: item.function_controller.output,
				categories: item.provider_controller.categories || [],
				providerInstances: item.provider_instances || [],
			}),
		);

		return functions;
	}, [functionInstanceData]);

	// Get unique providers and categories for filters from available providers
	const allProviders = useMemo(() => {
		if (!availableProvidersData?.items) return [];
		return availableProvidersData.items
			.map((p) => ({ value: p.type_id, label: p.name }))
			.sort((a, b) => a.label.localeCompare(b.label));
	}, [availableProvidersData]);

	const allCategories = useMemo(() => {
		if (!availableProvidersData?.items) return [];
		const categories = new Set<string>();
		availableProvidersData.items.forEach((p) => {
			(p.categories || []).forEach((c) => categories.add(c));
		});
		return Array.from(categories).sort();
	}, [availableProvidersData]);

	const [displayedAvailableFunctions, setDisplayedAvailableFunctions] =
		useState<AvailableFunction[]>([]);

	const ITEMS_PER_PAGE = 20;

	// Provider options for filters
	const providerOptions: SelectOption[] = useMemo(() => {
		// If no search query, show first 10 providers
		if (!providerSearchQuery) {
			return allProviders.slice(0, 10);
		}

		// Search through all providers
		const filtered = allProviders
			.filter((p) =>
				p.label.toLowerCase().includes(providerSearchQuery.toLowerCase()),
			)
			.slice(0, 10); // Limit search results to 10

		return filtered;
	}, [allProviders, providerSearchQuery]);

	// Category options for filters
	const categoryOptions: SelectOption[] = useMemo(() => {
		const categorySelectOptions = allCategories.map((c) => ({
			value: c,
			label: c,
		}));

		// If no search query, show first 10 categories
		if (!categorySearchQuery) {
			return categorySelectOptions.slice(0, 10);
		}

		// Search through all categories
		const filtered = categorySelectOptions
			.filter((c) =>
				c.label.toLowerCase().includes(categorySearchQuery.toLowerCase()),
			)
			.slice(0, 10); // Limit search results to 10

		return filtered;
	}, [allCategories, categorySearchQuery]);

	// Initialize displayed functions
	useEffect(() => {
		setDisplayedAvailableFunctions(availableFunctions.slice(0, ITEMS_PER_PAGE));
	}, [availableFunctions]);

	// Load more functions
	const loadMoreAvailable = useCallback(() => {
		setDisplayedAvailableFunctions((prev) => {
			const currentLength = prev.length;
			const nextItems = availableFunctions.slice(
				currentLength,
				currentLength + ITEMS_PER_PAGE,
			);
			return [...prev, ...nextItems];
		});
	}, [availableFunctions]);

	const handleFunctionClick = (func: AvailableFunction) => {
		navigate({ to: LINKS.BRIDGE_ENABLE_FUNCTIONS_FUNCTION(func.id) });
	};

	// If loading, show loading state
	if (isLoadingKeys || (hasEncryptionKeys && isLoadingFunctions)) {
		return (
			<PageLayout>
				<PageHeaderWithAction
					title="Enable Functions"
					description="Browse and enable available MCP functions"
				/>
				<div className="flex items-center justify-center min-h-[400px]">
					<p className="text-muted-foreground">Loading...</p>
				</div>
			</PageLayout>
		);
	}

	// If no encryption keys exist, show setup screen
	if (!hasEncryptionKeys) {
		return (
			<PageLayout>
				<PageHeaderWithAction
					title="Enable Functions"
					description="Browse and enable available MCP functions"
					actions={
						<Button
							onClick={handleEnableBridge}
							disabled={createKeyMutation.isPending}
						>
							{createKeyMutation.isPending
								? "Enabling..."
								: "Enable Bridge MCP"}
						</Button>
					}
				/>
				{createKeyMutation.isError && (
					<div className="p-4">
						<p className="text-sm text-destructive">
							Failed to enable Bridge MCP. Please try again.
						</p>
					</div>
				)}
			</PageLayout>
		);
	}

	return (
		<PageLayout>
			<div className="h-[calc(100vh-var(--header-height)-46px-var(--sub-nav-height))] ">
				<div className="h-full flex flex-col">
					<div className="space-y-4">
						<PageHeaderWithAction
							title="Enable Functions"
							description="Browse and enable available MCP functions"
						/>

						<div className="px-4 space-y-4">
							{/* Global filters */}
							<div className="flex flex-wrap gap-3 items-center">
								<SearchableSelect
									options={providerOptions}
									value={selectedProviderFilter}
									onChange={setSelectedProviderFilter}
									onSearchChange={setProviderSearchQuery}
									placeholder="Filter by provider"
									searchPlaceholder="Type to search providers..."
									emptyText="No providers found"
									className="w-[250px]"
									showAllOption={false}
								/>

								<SearchableSelect
									options={categoryOptions}
									value={selectedCategoryFilter}
									onChange={setSelectedCategoryFilter}
									onSearchChange={setCategorySearchQuery}
									placeholder="Filter by category"
									searchPlaceholder="Type to search categories..."
									emptyText="No categories found"
									className="w-[250px]"
									showAllOption={false}
								/>

								{(selectedProviderFilter || selectedCategoryFilter) && (
									<Button
										variant="ghost"
										size="sm"
										onClick={() => {
											setSelectedProviderFilter("");
											setSelectedCategoryFilter("");
										}}
										className="h-8 px-2 lg:px-3"
									>
										Clear filters
										<X className="ml-2 h-4 w-4" />
									</Button>
								)}
							</div>

							{/* Data tables */}
							<div className="space-y-4 relative">
								{/* Loading overlay - shows when fetching new data */}

								<div
									className={`transition-opacity duration-200 ${isFetchingFunctions && !isLoadingFunctions ? "opacity-60" : "opacity-100"}`}
								>
									<FunctionsTable
										functions={displayedAvailableFunctions}
										title="Available Functions"
										onRowClick={handleFunctionClick}
										loadMore={loadMoreAvailable}
										hasMore={
											displayedAvailableFunctions.length <
											availableFunctions.length
										}
									/>
								</div>
							</div>
						</div>
					</div>
				</div>
			</div>
		</PageLayout>
	);
}
