
"use client";
import { createFileRoute } from '@tanstack/react-router'

import { AnimatePresence, motion } from "framer-motion";
import {
	Package,
	X,
} from "lucide-react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import Form from "@rjsf/shadcn";
import validator from "@rjsf/validator-ajv8";
import { Button } from "@/components/ui/button";
import { PageLayout } from "@/components/ui/page-layout";
import { ScrollArea } from "@/components/ui/scroll-area";
import { PageHeaderWithAction } from "@/components/layout/page-header-with-action";
import {
	SearchableSelect,
	type SelectOption,
} from "@/components/ui/searchable-select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
	Accordion,
	AccordionContent,
	AccordionItem,
	AccordionTrigger,
} from "@/components/ui/accordion";
import { cn } from "@/lib/utils";
import $api from '@/lib/api-client.client';
import type { components } from "@/@types/openapi";

export const Route = createFileRoute('/bridge/')({
  component: RouteComponent,
})

function RouteComponent() {
  return <MCPToolsPage />
}

// Type aliases for better readability
type ProviderController = components["schemas"]["ProviderControllerSerialized"];
type CredentialController = components["schemas"]["ProviderCredentialControllerSerialized"];
type JsonSchema = components["schemas"]["JsonSchema"];

// Type for available functions derived from providers
interface AvailableFunction {
	id: string; // function type_id
	providerTypeId: string;
	providerName: string;
	functionName: string;
	documentation: string;
	parametersSchema: JsonSchema;
	outputSchema: JsonSchema;
	categories: string[]; // Provider categories
}

// Functions table component
const FunctionsTable = ({
	functions,
	title,
	onRowClick,
	loadMore,
	hasMore,
	selectedProvider,
	selectedCategory,
}: {
	functions: AvailableFunction[];
	title: string;
	onRowClick: (func: AvailableFunction) => void;
	loadMore?: () => void;
	hasMore?: boolean;
	selectedProvider: string;
	selectedCategory: string;
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

	// Filter functions
	const filteredFunctions = useMemo(() => {
		return functions.filter((func) => {
			const matchesProvider =
				selectedProvider === "" || func.providerName === selectedProvider;
			const matchesCategory =
				selectedCategory === "" || func.categories.includes(selectedCategory);
			return matchesProvider && matchesCategory;
		});
	}, [functions, selectedProvider, selectedCategory]);

	return (
		<div className="border rounded-lg bg-white">
			<div className="px-4 py-3 border-b">
				<h3 className="font-semibold text-sm">{title}</h3>
			</div>
			<div className="overflow-auto max-h-[300px]">
				<table className="w-full">
					<thead className="sticky top-0 z-10">
						<tr className="border-b bg-gray-50">
							<th className="px-3 py-2 text-left text-xs font-medium">
								Provider
							</th>
							<th className="px-3 py-2 text-left text-xs font-medium">
								Function
							</th>
						</tr>
					</thead>
					<tbody>
						{filteredFunctions.map((func, index) => (
							<tr
								key={func.id}
								className={cn(
									"cursor-pointer transition-colors hover:bg-gray-50",
									index % 2 === 1 && "bg-gray-50/50",
								)}
								onClick={() => onRowClick(func)}
							>
								<td className="px-3 py-2 text-sm font-medium">
									{func.providerName}
								</td>
								<td className="px-3 py-2 text-sm">{func.functionName}</td>
							</tr>
						))}
					</tbody>
				</table>
				{hasMore && (
					<div
						ref={loadMoreRef}
						className="p-4 text-center text-sm text-muted-foreground"
					>
						Loading more...
					</div>
				)}
			</div>
		</div>
	);
};

// Markdown Documentation Component
const MarkdownDocumentation = ({ content }: { content: string }) => {
	if (!content) {
		return (
			<div className="text-sm text-muted-foreground">
				No documentation available
			</div>
		);
	}

	return (
		<div className="markdown-content">
			<ReactMarkdown 
				remarkPlugins={[remarkGfm]}
				components={{
					h1: ({ node, ...props }) => <h1 className="text-2xl font-bold mb-4 mt-6" {...props} />,
					h2: ({ node, ...props }) => <h2 className="text-xl font-bold mb-3 mt-5" {...props} />,
					h3: ({ node, ...props }) => <h3 className="text-lg font-semibold mb-2 mt-4" {...props} />,
					h4: ({ node, ...props }) => <h4 className="text-base font-semibold mb-2 mt-3" {...props} />,
					h5: ({ node, ...props }) => <h5 className="text-sm font-semibold mb-2 mt-3" {...props} />,
					h6: ({ node, ...props }) => <h6 className="text-xs font-semibold mb-2 mt-3" {...props} />,
					p: ({ node, ...props }) => <p className="mb-4" {...props} />,
					ul: ({ node, ...props }) => <ul className="list-disc pl-6 mb-4 space-y-2" {...props} />,
					ol: ({ node, ...props }) => <ol className="list-decimal pl-6 mb-4 space-y-2" {...props} />,
					li: ({ node, ...props }) => <li className="mb-1" {...props} />,
					a: ({ node, ...props }) => <a className="text-blue-600 hover:text-blue-800 underline" {...props} />,
					code: ({ node, className, children, ...props }) => {
						const isInline = !className;
						return isInline ? (
							<code className="bg-gray-100 dark:bg-gray-800 px-1.5 py-0.5 rounded text-sm font-mono" {...props}>
								{children}
							</code>
						) : (
							<code className={cn("block bg-gray-100 dark:bg-gray-800 p-4 rounded text-sm font-mono overflow-x-auto", className)} {...props}>
								{children}
							</code>
						);
					},
					pre: ({ node, ...props }) => <pre className="mb-4 overflow-x-auto" {...props} />,
					blockquote: ({ node, ...props }) => <blockquote className="border-l-4 border-gray-300 pl-4 italic my-4" {...props} />,
					table: ({ node, ...props }) => <table className="min-w-full border-collapse border border-gray-300 my-4" {...props} />,
					thead: ({ node, ...props }) => <thead className="bg-gray-100 dark:bg-gray-800" {...props} />,
					tbody: ({ node, ...props }) => <tbody {...props} />,
					tr: ({ node, ...props }) => <tr className="border-b border-gray-300" {...props} />,
					th: ({ node, ...props }) => <th className="border border-gray-300 px-4 py-2 text-left font-semibold" {...props} />,
					td: ({ node, ...props }) => <td className="border border-gray-300 px-4 py-2" {...props} />,
					hr: ({ node, ...props }) => <hr className="my-8 border-t border-gray-300" {...props} />,
					strong: ({ node, ...props }) => <strong className="font-bold" {...props} />,
					em: ({ node, ...props }) => <em className="italic" {...props} />,
				}}
			>
				{content}
			</ReactMarkdown>
		</div>
	);
};

// Configuration Tab Component
const ConfigurationTab = ({ provider }: { provider: ProviderController }) => {
	const [configMode, setConfigMode] = useState<"existing" | "new">("new");
	
	// TODO: Fetch provider instances for this provider type
	// const existingProviderInstances = []; // Filter by provider.type_id
	const hasExistingProviders = false; // Change based on API data

	return (
		<div className="space-y-4">
			{hasExistingProviders && (
				<Tabs value={configMode} onValueChange={(v) => setConfigMode(v === "existing" || v === "new" ? v : "new")}>
					<TabsList className="grid w-full grid-cols-2">
						<TabsTrigger value="existing">Use Existing</TabsTrigger>
						<TabsTrigger value="new">Add New</TabsTrigger>
					</TabsList>

					<TabsContent value="existing" className="mt-4">
						<div className="space-y-4">
							<h4 className="font-semibold">Select Existing Provider Instance</h4>
							<SearchableSelect
								options={[]} // TODO: Map existing provider instances to options
								value=""
								onChange={() => {}}
								placeholder="Search for a provider instance..."
								searchPlaceholder="Type to search..."
								emptyText="No provider instances found"
								className="w-full"
							/>
						</div>
					</TabsContent>

					<TabsContent value="new" className="mt-4">
						<AddNewProviderInstance provider={provider} />
					</TabsContent>
				</Tabs>
			)}
			
			{!hasExistingProviders && (
				<AddNewProviderInstance provider={provider} />
			)}
		</div>
	);
};

// Add New Provider Instance Component
const AddNewProviderInstance = ({ provider }: { provider: ProviderController }) => {
	const credentialControllers = provider.credential_controllers;
	
	if (credentialControllers.length === 0) {
		return (
			<div className="text-sm text-muted-foreground">
				No credential controllers available for this provider
			</div>
		);
	}

	if (credentialControllers.length === 1) {
		return (
			<div className="space-y-6">
				{credentialControllers[0].documentation && (
					<div className="p-4 border rounded-lg bg-muted/50">
						<MarkdownDocumentation content={credentialControllers[0].documentation} />
					</div>
				)}
				<ResourceServerConfigurationForm 
					credentialController={credentialControllers[0]}
					providerTypeId={provider.type_id}
				/>
			</div>
		);
	}

	return (
		<div className="space-y-4">
			<div>
				<h4 className="font-semibold mb-2">Which authentication flow would you like to use?</h4>
				<p className="text-sm text-muted-foreground">
					This provider supports multiple authentication methods. Expand each of them below to find out which one suits your needs.
				</p>
			</div>
			<Accordion type="single" collapsible className="w-full">
				{credentialControllers.map((credController) => (
					<AccordionItem key={credController.type_id} value={credController.type_id}>
						<AccordionTrigger className="text-left">
							<div>
								<div className="font-medium">{credController.name}</div>
								{credController.documentation && (
									<div className="text-sm text-muted-foreground mt-1 line-clamp-2">
										{credController.documentation.substring(0, 100)}...
									</div>
								)}
							</div>
					</AccordionTrigger>
					<AccordionContent>
						<div className="pt-4">
							<ResourceServerConfigurationForm 
								credentialController={credController}
								providerTypeId={provider.type_id}
							/>
						</div>
					</AccordionContent>
					</AccordionItem>
				))}
			</Accordion>
		</div>
	);
};

// Resource Server Configuration Form Component
const ResourceServerConfigurationForm = ({ 
	credentialController,
	providerTypeId
}: { 
	credentialController: CredentialController;
	providerTypeId: string;
}) => {
	const [formErrors, setFormErrors] = useState<string[]>([]);
	
	// Parse the JSON schema for resource server configuration
	const schema = credentialController.configuration_schema.resource_server;
	
	if (!schema) {
		return (
			<div className="text-sm text-muted-foreground">
				No configuration schema available
			</div>
		);
	}

	const handleSubmit = async (data: { formData?: unknown }) => {
		console.log("Form submitted:", data.formData);
		setFormErrors([]);
		setIsSubmitting(true);
		
		try {
			// Get the first encryption key
			const encryptionKeyId = dataEncryptionKeys?.items?.[0]?.id;
			if (!encryptionKeyId) {
				setFormErrors(["No encryption key available. Please enable Bridge MCP first."]);
				setIsSubmitting(false);
				return;
			}
			
			// Step 1: Encrypt the resource server configuration
			const encryptedConfig = await encryptConfigMutation.mutateAsync({
				params: {
					path: {
						provider_controller_type_id: providerTypeId,
						credential_controller_type_id: credentialController.type_id,
					},
				},
				body: {
					data_encryption_key_id: encryptionKeyId,
					value: data.formData || {},
				},
			});
			
			// Step 2: Create the resource server credential
			const credential = await createCredentialMutation.mutateAsync({
				params: {
					path: {
						provider_controller_type_id: providerTypeId,
						credential_controller_type_id: credentialController.type_id,
					},
				},
				body: {
					data_encryption_key_id: encryptionKeyId,
					resource_server_configuration: encryptedConfig,
					metadata: null,
				},
			});
			
			console.log("Resource server credential created:", credential);
			
			// Step 3: If requires brokering, start the user credential brokering flow
			if (credentialController.requires_brokering && credential.id) {
				const brokeringResponse = await startBrokeringMutation.mutateAsync({
					params: {
						path: {
							provider_controller_type_id: providerTypeId,
							credential_controller_type_id: credentialController.type_id,
						},
					},
					body: {
						resource_server_cred_id: credential.id,
					},
				});
				
				console.log("Brokering response:", brokeringResponse);
				
				// Check if we need to redirect
				if (brokeringResponse.type === "broker_state") {
					const action = brokeringResponse.action;
					if (typeof action === "object" && action && "Redirect" in action) {
						const redirectUrl = action.Redirect.url;
						console.log("Redirecting to:", redirectUrl);
						window.location.href = redirectUrl;
						return;
					}
				}
			}
			
			// Success - no redirect needed
			alert("Provider instance created successfully!");
			setIsSubmitting(false);
			
		} catch (error) {
			console.error("Error creating provider instance:", error);
			setFormErrors([
				error instanceof Error ? error.message : "Failed to create provider instance. Please try again."
			]);
			setIsSubmitting(false);
		}
	};

	const handleError = (errors: unknown) => {
		console.log("Form errors:", errors);
		const errorMessages = Array.isArray(errors) 
			? errors.map((err) => err.message || "Invalid field")
			: ["Form validation failed"];
		setFormErrors(errorMessages);
	};

	const submitButtonText = credentialController.requires_brokering
		? "Save, and configure user credentials"
		: "Save and enable function";

	// Build UI schema to customize field rendering
	const properties = (schema.properties as Record<string, JsonSchema>) || {};
	const uiSchema: Record<string, unknown> = {
		"ui:submitButtonOptions": {
			submitText: isSubmitting ? "Submitting..." : submitButtonText,
			norender: false,
			props: {
				disabled: isSubmitting
			}
		}
	};

	// Add spacing and better styling for each field
	Object.keys(properties).forEach((key) => {
		const prop = properties[key];
		const propFormat = prop.format as string | undefined;
		
		uiSchema[key] = {
			"ui:classNames": "mb-6",
			...(propFormat === "password" && {
				"ui:widget": "password"
			}),
			...(prop.type === "object" && {
				"ui:widget": "textarea",
				"ui:options": {
					rows: 5
				}
			})
		};
	});

	return (
		<div className="space-y-6">
			{credentialController.documentation && (
				<div className="p-4 border rounded-lg bg-muted/50">
					<MarkdownDocumentation content={credentialController.documentation} />
				</div>
			)}
			
			{formErrors.length > 0 && (
				<div className="p-4 border border-destructive bg-destructive/10 rounded-lg">
					<h4 className="text-sm font-semibold text-destructive mb-2">Form Errors:</h4>
					<ul className="list-disc list-inside space-y-1">
						{formErrors.map((error, idx) => (
							<li key={idx} className="text-sm text-destructive">{error}</li>
						))}
					</ul>
				</div>
			)}
			
			<div className="rjsf-form-wrapper">
				<Form
					schema={schema}
					validator={validator}
					onSubmit={handleSubmit}
					onError={handleError}
					uiSchema={uiSchema}
					showErrorList={false}
				/>
			</div>
		</div>
	);
};

// Documentation panel component
const DocumentationPanel = ({
	func,
	provider,
	onClose,
}: {
	func: AvailableFunction | null;
	provider: ProviderController | null;
	onClose: () => void;
}) => {
	if (!func || !provider) return null;

	return (
		<div className="h-full flex flex-col bg-background">
			<div className="flex items-center justify-between p-4 border-b">
				<div className="flex items-center gap-3">
					<Package className="h-5 w-5" />
					<div>
						<h2 className="font-semibold">{provider.name}</h2>
						<p className="text-sm text-muted-foreground">{func.functionName}</p>
					</div>
				</div>
				<Button variant="ghost" size="icon" onClick={onClose}>
					<X className="h-4 w-4" />
				</Button>
			</div>

			<Tabs defaultValue="function" className="flex-1 flex flex-col">
				<TabsList className="mx-4 mt-4 grid w-fit grid-cols-3">
					<TabsTrigger value="function">Fn Documentation</TabsTrigger>
					<TabsTrigger value="provider">Provider Documentation</TabsTrigger>
					<TabsTrigger value="configuration">Configure</TabsTrigger>
				</TabsList>

				<ScrollArea className="flex-1">
					<TabsContent value="function" className="p-6 mt-0">
						<div className="space-y-4">
							<div>
								<h3 className="font-semibold mb-2">Documentation</h3>
								<MarkdownDocumentation content={func.documentation} />
							</div>
							<div>
								<h3 className="font-semibold mb-2">Parameters Schema</h3>
								<pre className="bg-gray-100 p-4 rounded text-sm overflow-auto">
									{JSON.stringify(func.parametersSchema, null, 2)}
								</pre>
							</div>
							<div>
								<h3 className="font-semibold mb-2">Output Schema</h3>
								<pre className="bg-gray-100 p-4 rounded text-sm overflow-auto">
									{JSON.stringify(func.outputSchema, null, 2)}
								</pre>
							</div>
						</div>
					</TabsContent>

					<TabsContent value="provider" className="p-6 mt-0">
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
										<p className="text-sm text-muted-foreground">{cred.documentation}</p>
									</div>
								))}
							</div>
						</div>
						</div>
					</TabsContent>

				<TabsContent value="configuration" className="p-6 mt-0">
					<ConfigurationTab provider={provider} />
				</TabsContent>
				</ScrollArea>
			</Tabs>
		</div>
	);
};

function MCPToolsPage() {
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
	const hasEncryptionKeys = dataEncryptionKeys?.items && dataEncryptionKeys.items.length > 0;
	
	// Query available providers (only when keys exist)
	const {
		data: availableProviders,
		isLoading: isLoadingProviders,
	} = $api.useQuery("get", "/api/bridge/v1/available-providers", 
		{
			params: {
				query: {
					page_size: 1000,
				},
			},
		},
		{
			enabled: hasEncryptionKeys,
		}
	);

	// Mutation to create data encryption key
	const createKeyMutation = $api.useMutation("post", "/api/bridge/v1/encryption/data-encryption-key");

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

	// Transform provider data into available functions
	const availableFunctions = useMemo(() => {
		if (!availableProviders?.items) return [];
		
		const functions: AvailableFunction[] = [];
		for (const provider of availableProviders.items) {
			for (const func of provider.functions) {
				functions.push({
					id: func.type_id,
					providerTypeId: provider.type_id,
					providerName: provider.name,
					functionName: func.name,
					documentation: func.documentation,
					parametersSchema: func.parameters,
					outputSchema: func.output,
					categories: provider.categories || [],
				});
			}
		}
		return functions;
	}, [availableProviders]);

	// Get unique providers and categories for filters
	const allProviders = useMemo(
		() => Array.from(new Set(availableFunctions.map((f) => f.providerName))).sort(),
		[availableFunctions],
	);

	const allCategories = useMemo(
		() => Array.from(new Set(availableFunctions.flatMap((f) => f.categories))).sort(),
		[availableFunctions],
	);

	const [displayedAvailableFunctions, setDisplayedAvailableFunctions] = useState<
		AvailableFunction[]
	>([]);
	const [selectedFunction, setSelectedFunction] = useState<AvailableFunction | null>(null);
	const [selectedProvider, setSelectedProvider] = useState<ProviderController | null>(null);
	const [isPanelOpen, setIsPanelOpen] = useState(false);
	const [selectedProviderFilter, setSelectedProviderFilter] = useState("");
	const [selectedCategoryFilter, setSelectedCategoryFilter] = useState("");
	const [providerSearchQuery, setProviderSearchQuery] = useState("");
	const [categorySearchQuery, setCategorySearchQuery] = useState("");

	const ITEMS_PER_PAGE = 20;

	// Provider options for filters
	const providerOptions: SelectOption[] = useMemo(() => {
		// If no search query, show first 10 providers
		if (!providerSearchQuery) {
			return allProviders.slice(0, 10).map((p) => ({ value: p, label: p }));
		}

		// Search through all providers
		const filtered = allProviders
			.filter((p) =>
				p.toLowerCase().includes(providerSearchQuery.toLowerCase()),
			)
			.slice(0, 10); // Limit search results to 10

		return filtered.map((p) => ({ value: p, label: p }));
	}, [allProviders, providerSearchQuery]);

	// Category options for filters
	const categoryOptions: SelectOption[] = useMemo(() => {
		// If no search query, show first 10 categories
		if (!categorySearchQuery) {
			return allCategories.slice(0, 10).map((c) => ({ value: c, label: c }));
		}

		// Search through all categories
		const filtered = allCategories
			.filter((c) =>
				c.toLowerCase().includes(categorySearchQuery.toLowerCase()),
			)
			.slice(0, 10); // Limit search results to 10

		return filtered.map((c) => ({ value: c, label: c }));
	}, [allCategories, categorySearchQuery]);

	// Initialize displayed functions
	useEffect(() => {
		setDisplayedAvailableFunctions(availableFunctions.slice(0, ITEMS_PER_PAGE));
	}, [availableFunctions]);

	// Load more functions
	const loadMoreAvailable = useCallback(() => {
		const currentLength = displayedAvailableFunctions.length;
		const nextItems = availableFunctions.slice(
			currentLength,
			currentLength + ITEMS_PER_PAGE,
		);
		setDisplayedAvailableFunctions((prev) => [...prev, ...nextItems]);
	}, [displayedAvailableFunctions, availableFunctions]);

	const handleFunctionClick = (func: AvailableFunction) => {
		setSelectedFunction(func);
		// Find the full provider object
		const provider = availableProviders?.items.find((p) => p.type_id === func.providerTypeId);
		setSelectedProvider(provider || null);
		setIsPanelOpen(true);
	};

	const handleClosePanel = () => {
		setIsPanelOpen(false);
		setTimeout(() => {
			setSelectedFunction(null);
			setSelectedProvider(null);
		}, 300);
	};

	// If loading, show loading state
	if (isLoadingKeys || (hasEncryptionKeys && isLoadingProviders)) {
		return (
			<PageLayout>
				<PageHeaderWithAction
					title="Bridge MCP"
					description="Manage and configure Model Context Protocol tools"
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
					title="Bridge MCP"
					description="Manage and configure Model Context Protocol tools"
					actions={
						<Button 
							onClick={handleEnableBridge} 
							disabled={createKeyMutation.isPending}
						>
							{createKeyMutation.isPending ? "Enabling..." : "Enable Bridge MCP"}
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
		<>
			<PageLayout>
				<div className="h-[calc(100vh-var(--header-height)-46px)] ">
					<div className="h-full flex flex-col">
						<div className="space-y-4">
							<PageHeaderWithAction
								title="Bridge MCP"
								description="Manage and configure Model Context Protocol tools"
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
								<div className="space-y-4">
									{/* <MCPToolsTable
										tools={displayedEnabledTools}
										title="Enabled Functions"
										onRowClick={handleRowClick}
										loadMore={loadMoreEnabled}
										hasMore={displayedEnabledTools.length < enabledTools.length}
										selectedPlatform={selectedProviderFilter}
										selectedCategory=""
									/> */}

									<FunctionsTable
										functions={displayedAvailableFunctions}
										title="Available Functions"
										onRowClick={handleFunctionClick}
										loadMore={loadMoreAvailable}
										hasMore={
											displayedAvailableFunctions.length < availableFunctions.length
										}
										selectedProvider={selectedProviderFilter}
										selectedCategory={selectedCategoryFilter}
									/>
								</div>
							</div>
						</div>
					</div>
				</div>
			</PageLayout>

			{/* Documentation panel - positioned relative to body */}
			<AnimatePresence>
				{isPanelOpen && (
					<motion.div
						className="fixed top-0 right-0 h-screen w-[50vw] border-l bg-background shadow-2xl overflow-y-auto z-50"
						initial={{ x: "100%" }}
						animate={{ x: 0 }}
						exit={{ x: "100%" }}
						transition={{ type: "spring", stiffness: 300, damping: 30 }}
					>
						<DocumentationPanel
							func={selectedFunction}
							provider={selectedProvider}
							onClose={handleClosePanel}
						/>
					</motion.div>
				)}
			</AnimatePresence>
		</>
	);
}
