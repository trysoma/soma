import Form from "@rjsf/shadcn";
import validator from "@rjsf/validator-ajv8";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import "@/styles/rjsf-overrides.css";
import { Check, Link, X } from "lucide-react";

// Default DEK alias to use for encrypting credentials
const DEFAULT_DEK_ALIAS = "default";

import type { components } from "@/@types/openapi";
import {
	Accordion,
	AccordionContent,
	AccordionItem,
	AccordionTrigger,
} from "@/components/ui/accordion";
import {
	Table,
	TableBody,
	TableCell,
	TableContainer,
	TableHead,
	TableHeader,
	TableRow,
	TableTitle,
	TableWrapper,
} from "@/components/ui/table";
import $api from "@/lib/api-client.client";
import {
	invalidateDataByControllerTypeId,
	invalidateDataByProviderInstanceId,
	invalidateFunctionInstancesData,
} from "@/lib/query-cache";
import { cn } from "@/lib/utils";
import { queryClient } from "@/main";
import { MarkdownDocumentation } from "./markdown-documentation";

type ProviderController = components["schemas"]["ProviderControllerSerialized"];
type CredentialController =
	components["schemas"]["ProviderCredentialControllerSerialized"];
type ResourceServerCredential =
	components["schemas"]["ResourceServerCredentialSerialized"];
type JsonSchema = components["schemas"]["JsonSchema"];

interface ConfigurationFormProps {
	provider: ProviderController;
	existingProviderInstance?: {
		id: string;
		credential_controller_type_id: string;
		display_name: string;
	} | null;
	existingResourceServerCredential?: ResourceServerCredential | null;
	existingUserCredential?: UserCredential | null;
	functionControllerId?: string; // If provided, will create function instance after provider instance creation
	onSuccess?: () => void;
}

type UserCredential = components["schemas"]["UserCredentialSerialized"];

// Main ConfigurationForm component - renders the credential form and accordion
export const ConfigurationForm = ({
	provider,
	existingProviderInstance,
	existingResourceServerCredential,
	existingUserCredential,
	functionControllerId,
	onSuccess,
}: ConfigurationFormProps) => {
	return (
		<div className="space-y-4">
			<AddNewProviderInstance
				provider={provider}
				existingProviderInstance={existingProviderInstance}
				existingResourceServerCredential={existingResourceServerCredential}
				existingUserCredential={existingUserCredential}
				functionControllerId={functionControllerId}
				onSuccess={onSuccess}
			/>
		</div>
	);
};

// Existing Provider Instances Table Component - exported for composition
export interface LinkProviderInstancesTableProps {
	instances: any[];
	provider: ProviderController;
	functionTypeId: string;
	onSuccess?: () => void;
}

export const LinkProviderInstancesTable = ({
	instances,
	provider,
	functionTypeId,
	onSuccess,
}: LinkProviderInstancesTableProps) => {
	const [linkingInstanceId, setLinkingInstanceId] = useState<string | null>(
		null,
	);
	const [unlinkingInstanceId, setUnlinkingInstanceId] = useState<string | null>(
		null,
	);

	// Helper function to get credential controller display name
	const getCredentialControllerName = (credentialControllerTypeId: string) => {
		const credController = provider.credential_controllers.find(
			(cc) => cc.type_id === credentialControllerTypeId,
		);
		return credController?.name || credentialControllerTypeId;
	};

	// Query function instances to check which provider instances already have this function enabled
	const { data: functionInstancesData, isLoading: isLoadingFunctionInstances } =
		$api.useQuery("get", "/api/bridge/v1/function-instances", {
			params: {
				query: {
					page_size: 1000,
				},
			},
		});

	// Helper function to check if credentials already have this function enabled
	const isAlreadyLinked = (providerInstanceId: string) => {
		if (!functionTypeId || !functionInstancesData?.items) return false;
		return functionInstancesData.items.some(
			(fi) =>
				fi.provider_instance_id === providerInstanceId &&
				fi.function_controller_type_id === functionTypeId,
		);
	};

	// Enable function mutation
	const enableFunctionMutation = $api.useMutation(
		"post",
		"/api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/enable",
	);

	// Disable function mutation
	const disableFunctionMutation = $api.useMutation(
		"post",
		"/api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/disable",
	);

	const handleLinkInstance = async (providerInstanceId: string) => {
		setLinkingInstanceId(providerInstanceId);

		try {
			await enableFunctionMutation.mutateAsync({
				params: {
					path: {
						provider_instance_id: providerInstanceId,
						function_controller_type_id: functionTypeId,
					},
				},
				body: {},
			});
			invalidateDataByControllerTypeId(queryClient, provider.type_id);
			invalidateDataByProviderInstanceId(queryClient, providerInstanceId);
			invalidateFunctionInstancesData(queryClient);
			onSuccess?.();
		} catch (error) {
			console.error("Failed to enable function:", error);
			alert("Failed to enable function. Please try again.");
		} finally {
			setLinkingInstanceId(null);
		}
	};

	const handleUnlinkInstance = async (providerInstanceId: string) => {
		setUnlinkingInstanceId(providerInstanceId);

		try {
			await disableFunctionMutation.mutateAsync({
				params: {
					path: {
						provider_instance_id: providerInstanceId,
						function_controller_type_id: functionTypeId,
					},
				},
				body: undefined,
			});
			invalidateDataByControllerTypeId(queryClient, provider.type_id);
			invalidateDataByProviderInstanceId(queryClient, providerInstanceId);
			invalidateFunctionInstancesData(queryClient);
			onSuccess?.();
		} catch (error) {
			console.error("Failed to disable function:", error);
			alert("Failed to disable function. Please try again.");
		} finally {
			setUnlinkingInstanceId(null);
		}
	};

	if (isLoadingFunctionInstances) {
		return (
			<div className="flex items-center justify-center p-8">
				<p className="text-muted-foreground">Loading...</p>
			</div>
		);
	}

	return (
		<div className="space-y-4">
			<div>
				<h4 className="font-semibold mb-2">Select Existing Credentials</h4>
				<p className="text-sm text-muted-foreground mb-4">
					Link this function to existing configured credentials.
				</p>
			</div>

			<TableWrapper>
				<TableTitle>Available Credentials</TableTitle>
				<TableContainer maxHeight="max-h-[350px]">
					<Table>
						<TableHeader sticky>
							<TableRow>
								<TableHead>Display Name</TableHead>
								<TableHead>Credential Type</TableHead>
								<TableHead>Status</TableHead>
								<TableHead className="w-40 min-w-40"></TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{instances.map((instance, index) => {
								const alreadyLinked = isAlreadyLinked(instance.id);
								return (
									<TableRow key={instance.id} index={index}>
										<TableCell bold>{instance.display_name}</TableCell>
										<TableCell>
											{getCredentialControllerName(
												instance.credential_controller_type_id,
											)}
										</TableCell>
										<TableCell>
											<span
												className={cn(
													"px-2 py-1 rounded text-xs",
													instance.status === "active"
														? "bg-green-100 text-green-800"
														: "bg-yellow-100 text-yellow-800",
												)}
											>
												{instance.status}
											</span>
										</TableCell>
										<TableCell className="text-right min-w-40">
											{alreadyLinked ? (
												<Button
													variant="outline"
													size="sm"
													onClick={() => handleUnlinkInstance(instance.id)}
													disabled={unlinkingInstanceId === instance.id}
													className="hover:bg-red-50 hover:text-red-600"
												>
													<X className="h-4 w-4 mr-1" />
													{unlinkingInstanceId === instance.id
														? "Unlinking..."
														: "Unlink"}
												</Button>
											) : (
												<Button
													variant="outline"
													size="sm"
													onClick={() => handleLinkInstance(instance.id)}
													disabled={linkingInstanceId === instance.id}
												>
													<Link className="h-4 w-4 mr-1" />
													{linkingInstanceId === instance.id
														? "Linking..."
														: "Link"}
												</Button>
											)}
										</TableCell>
									</TableRow>
								);
							})}
						</TableBody>
					</Table>
				</TableContainer>
			</TableWrapper>
		</div>
	);
};

// Add New Provider Instance Component
const AddNewProviderInstance = ({
	provider,
	existingProviderInstance,
	existingResourceServerCredential,
	existingUserCredential,
	functionControllerId,
	onSuccess,
}: {
	provider: ProviderController;
	existingProviderInstance?: {
		id: string;
		credential_controller_type_id: string;
		display_name: string;
	} | null;
	existingResourceServerCredential?: ResourceServerCredential | null;
	existingUserCredential?: UserCredential | null;
	functionControllerId?: string;
	onSuccess?: () => void;
}) => {
	const credentialControllers = provider.credential_controllers;
	const currentCredentialTypeId =
		existingProviderInstance?.credential_controller_type_id;

	if (credentialControllers.length === 0) {
		return (
			<div className="text-sm text-muted-foreground">
				No credential controllers available for this provider
			</div>
		);
	}

	// If there's only one credential controller, show it directly
	if (credentialControllers.length === 1) {
		return (
			<div className="space-y-6">
				<ResourceServerConfigurationForm
					credentialController={credentialControllers[0]}
					providerTypeId={provider.type_id}
					existingProviderInstance={existingProviderInstance}
					existingResourceServerCredential={existingResourceServerCredential}
					existingUserCredential={existingUserCredential}
					functionControllerId={functionControllerId}
					onSuccess={onSuccess}
				/>
			</div>
		);
	}

	// Multiple credential controllers - show accordion
	// Default to expanding the current one if editing, otherwise none
	const defaultValue = currentCredentialTypeId || undefined;

	return (
		<div className="space-y-4">
			<div>
				<h4 className="font-semibold mb-2">
					Which authentication flow would you like to use?
				</h4>
				<p className="text-sm text-muted-foreground">
					This provider supports multiple authentication methods.{" "}
					{existingProviderInstance
						? "The current authentication method is marked with a checkmark."
						: "Expand each of them below to find out which one suits your needs."}
				</p>
			</div>
			<Accordion
				type="single"
				collapsible
				className="w-full"
				defaultValue={defaultValue}
			>
				{credentialControllers.map((credController) => {
					const isCurrentCredential =
						currentCredentialTypeId === credController.type_id;
					return (
						<AccordionItem
							key={credController.type_id}
							value={credController.type_id}
						>
							<AccordionTrigger className="text-left">
								<div className="flex items-center gap-2">
									<div>
										<div className="font-medium flex items-center gap-2">
											{credController.name}
											{isCurrentCredential && (
												<Check className="h-4 w-4 text-green-600" />
											)}
										</div>
										{credController.documentation && (
											<div className="text-sm text-muted-foreground mt-1 line-clamp-2">
												{credController.documentation.substring(0, 100)}...
											</div>
										)}
									</div>
								</div>
							</AccordionTrigger>
							<AccordionContent>
								<div className="pt-4">
									<ResourceServerConfigurationForm
										credentialController={credController}
										providerTypeId={provider.type_id}
										existingProviderInstance={existingProviderInstance}
										existingResourceServerCredential={
											existingResourceServerCredential
										}
										existingUserCredential={existingUserCredential}
										functionControllerId={functionControllerId}
										onSuccess={onSuccess}
									/>
								</div>
							</AccordionContent>
						</AccordionItem>
					);
				})}
			</Accordion>
		</div>
	);
};

// Resource Server Configuration Form Component
interface ResourceServerConfigurationFormProps {
	credentialController: CredentialController;
	providerTypeId: string;
	existingProviderInstance?: {
		id: string;
		credential_controller_type_id: string;
		display_name: string;
	} | null;
	existingResourceServerCredential?: ResourceServerCredential | null;
	existingUserCredential?: UserCredential | null;
	functionControllerId?: string;
	onSuccess?: () => void;
}

const ResourceServerConfigurationForm = ({
	credentialController,
	providerTypeId,
	existingProviderInstance,
	existingResourceServerCredential,
	existingUserCredential,
	functionControllerId,
	onSuccess,
}: ResourceServerConfigurationFormProps) => {
	const [formErrors, setFormErrors] = useState<string[]>([]);
	const [isSubmitting, setIsSubmitting] = useState(false);
	const [accountName, setAccountName] = useState(
		existingProviderInstance?.display_name || "",
	);

	// Helper to check if we should populate from existing credentials
	// Only populate if the credential type matches what this controller expects
	const shouldPopulateFromExistingResourceServer =
		existingResourceServerCredential &&
		existingResourceServerCredential.type_id ===
			getExpectedResourceServerTypeId(credentialController.type_id);

	const shouldPopulateFromExistingUserCredential =
		existingUserCredential &&
		existingUserCredential.type_id ===
			getExpectedUserCredentialTypeId(credentialController.type_id);

	// Helper function to determine expected resource server credential type ID based on credential controller type
	function getExpectedResourceServerTypeId(
		credentialControllerTypeId: string,
	): string {
		// Map credential controller type IDs to their expected resource server credential type IDs
		const typeMapping: Record<string, string> = {
			oauth_auth_flow: "oauth2_authorization_code_flow_resource_server",
			oauth2_jwt_bearer_assertion_flow:
				"oauth2_jwt_bearer_assertion_flow_resource_server",
			// Add more mappings as needed
		};
		return (
			typeMapping[credentialControllerTypeId] ||
			`${credentialControllerTypeId}_resource_server`
		);
	}

	// Helper function to determine expected user credential type ID based on credential controller type
	function getExpectedUserCredentialTypeId(
		credentialControllerTypeId: string,
	): string {
		// Map credential controller type IDs to their expected user credential type IDs
		const typeMapping: Record<string, string> = {
			oauth_auth_flow: "oauth2_authorization_code_flow_user",
			oauth2_jwt_bearer_assertion_flow: "oauth2_jwt_bearer_assertion_flow_user",
			api_key: "user_api_key",
			no_auth: "user_no_auth",
			// Add more mappings as needed
		};
		return (
			typeMapping[credentialControllerTypeId] ||
			`${credentialControllerTypeId}_user`
		);
	}

	// Mutations
	const encryptConfigMutation = $api.useMutation(
		"post",
		"/api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/resource-server/encrypt",
	);
	const createCredentialMutation = $api.useMutation(
		"post",
		"/api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/resource-server",
	);
	const createProviderInstanceMutation = $api.useMutation(
		"post",
		"/api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}",
	);
	const startBrokeringMutation = $api.useMutation(
		"post",
		"/api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential/broker",
	);
	const encryptUserCredentialMutation = $api.useMutation(
		"post",
		"/api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential/encrypt",
	);
	const createUserCredentialMutation = $api.useMutation(
		"post",
		"/api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential",
	);
	const createFunctionInstanceMutation = $api.useMutation(
		"post",
		"/api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/enable",
	);

	// Parse the JSON schemas for resource server and user credential configuration
	const rawResourceServerSchema =
		credentialController.configuration_schema.resource_server;
	const rawUserCredentialSchema =
		credentialController.configuration_schema.user_credential;

	if (!rawResourceServerSchema) {
		return (
			<div className="text-sm text-muted-foreground">
				No configuration schema available
			</div>
		);
	}

	// Remove $schema and title properties to avoid validation issues and hide the form header
	const resourceServerSchema = { ...rawResourceServerSchema };
	delete resourceServerSchema.$schema;
	delete resourceServerSchema.title;

	// Prepare user credential schema if available and not requiring brokering
	const userCredentialSchema = rawUserCredentialSchema
		? { ...rawUserCredentialSchema }
		: null;
	if (userCredentialSchema) {
		delete userCredentialSchema.$schema;
		delete userCredentialSchema.title;
	}

	// Determine if we need to show user credential form (non-brokering providers)
	const showUserCredentialForm =
		!credentialController.requires_brokering && userCredentialSchema;

	// Combine schemas if showing user credential form
	const combinedSchema =
		showUserCredentialForm && userCredentialSchema
			? {
					...resourceServerSchema,
					properties: {
						...((resourceServerSchema.properties as object) || {}),
						...((userCredentialSchema.properties as object) || {}),
					},
					required: [
						...(Array.isArray(resourceServerSchema.required)
							? resourceServerSchema.required
							: []),
						...(Array.isArray(userCredentialSchema.required)
							? userCredentialSchema.required
							: []),
					],
				}
			: resourceServerSchema;

	const handleSubmit = async (data: { formData?: unknown }) => {
		console.log("Form submitted:", data.formData);
		setFormErrors([]);

		// Validate account name
		if (!accountName.trim()) {
			setFormErrors(["Account name is required"]);
			return;
		}

		setIsSubmitting(true);

		try {
			// Prepare form data for submission
			const formData = (data.formData as Record<string, any>) || {};

			// Split form data into resource server and user credential parts
			const resourceServerProperties =
				(resourceServerSchema.properties as Record<string, JsonSchema>) || {};
			const userCredentialProperties =
				showUserCredentialForm && userCredentialSchema
					? (userCredentialSchema.properties as Record<string, JsonSchema>) ||
						{}
					: {};

			let resourceServerData: Record<string, any> = {};
			let userCredentialData: Record<string, any> = {};

			// Separate the form data
			Object.keys(formData).forEach((key) => {
				if (key in resourceServerProperties) {
					resourceServerData[key] = formData[key];
				} else if (key in userCredentialProperties) {
					userCredentialData[key] = formData[key];
				}
			});

			// Handle password fields for resource server credentials
			if (
				shouldPopulateFromExistingResourceServer &&
				existingResourceServerCredential?.value
			) {
				const existingValue = existingResourceServerCredential.value as Record<
					string,
					any
				>;
				const updatedData: Record<string, any> = { ...resourceServerData };

				Object.keys(resourceServerProperties).forEach((key) => {
					const prop = resourceServerProperties[key];
					const propFormat = prop.format as string | undefined;
					const isPasswordField =
						propFormat === "password" ||
						key.toLowerCase().includes("secret") ||
						key.toLowerCase().includes("password");

					// If this is a password field and it's empty/undefined, use the original value
					if (
						isPasswordField &&
						!resourceServerData[key] &&
						key in existingValue
					) {
						updatedData[key] = existingValue[key];
					}
				});

				resourceServerData = updatedData;
			}

			// Handle password fields for user credentials
			if (
				showUserCredentialForm &&
				shouldPopulateFromExistingUserCredential &&
				existingUserCredential?.value
			) {
				const existingValue = existingUserCredential.value as Record<
					string,
					any
				>;
				const updatedData: Record<string, any> = { ...userCredentialData };

				Object.keys(userCredentialProperties).forEach((key) => {
					const prop = userCredentialProperties[key];
					const propFormat = prop.format as string | undefined;
					const isPasswordField =
						propFormat === "password" ||
						key.toLowerCase().includes("secret") ||
						key.toLowerCase().includes("password");

					// If this is a password field and it's empty/undefined, use the original value
					if (
						isPasswordField &&
						!userCredentialData[key] &&
						key in existingValue
					) {
						updatedData[key] = existingValue[key];
					}
				});

				userCredentialData = updatedData;
			}

			// Step 1: Encrypt the resource server configuration
			const encryptedResourceServerConfig =
				await encryptConfigMutation.mutateAsync({
					params: {
						path: {
							provider_controller_type_id: providerTypeId,
							credential_controller_type_id: credentialController.type_id,
						},
					},
					body: {
						dek_alias: DEFAULT_DEK_ALIAS,
						value: resourceServerData,
					},
				});

			// Step 2: Create the resource server credential
			const resourceServerCredential =
				await createCredentialMutation.mutateAsync({
					params: {
						path: {
							provider_controller_type_id: providerTypeId,
							credential_controller_type_id: credentialController.type_id,
						},
					},
					body: {
						dek_alias: DEFAULT_DEK_ALIAS,
						resource_server_configuration: encryptedResourceServerConfig,
						metadata: null,
					},
				});

			console.log(
				"Resource server credential created:",
				resourceServerCredential,
			);

			// Step 3: Create user credential if not requiring brokering
			let userCredential = null;
			if (showUserCredentialForm) {
				// Encrypt user credential configuration
				const encryptedUserCredentialConfig =
					await encryptUserCredentialMutation.mutateAsync({
						params: {
							path: {
								provider_controller_type_id: providerTypeId,
								credential_controller_type_id: credentialController.type_id,
							},
						},
						body: {
							dek_alias: DEFAULT_DEK_ALIAS,
							value: userCredentialData,
						},
					});

				// Create user credential
				userCredential = await createUserCredentialMutation.mutateAsync({
					params: {
						path: {
							provider_controller_type_id: providerTypeId,
							credential_controller_type_id: credentialController.type_id,
						},
					},
					body: {
						dek_alias: DEFAULT_DEK_ALIAS,
						user_credential_configuration: encryptedUserCredentialConfig,
						metadata: null,
					},
				});

				console.log("User credential created:", userCredential);
			}

			// Step 4: Create the provider instance
			const providerInstance = await createProviderInstanceMutation.mutateAsync(
				{
					params: {
						path: {
							provider_controller_type_id: providerTypeId,
							credential_controller_type_id: credentialController.type_id,
						},
					},
					body: {
						resource_server_credential_id: resourceServerCredential.id,
						display_name: accountName.trim(),
						...(userCredential && { user_credential_id: userCredential.id }),
						return_on_successful_brokering: {
							type: "url",
							url: window.location.href,
						},
					},
				},
			);

			console.log("Provider instance created:", providerInstance);

			// Step 5: Create function instance if functionControllerId is provided
			if (functionControllerId) {
				try {
					await createFunctionInstanceMutation.mutateAsync({
						params: {
							path: {
								provider_instance_id: providerInstance.id,
								function_controller_type_id: functionControllerId,
							},
						},
						body: {},
					});
					console.log("Function instance created for:", functionControllerId);
				} catch (error) {
					console.error("Error creating function instance:", error);
					// Don't fail the whole flow if function instance creation fails
					// The user can enable it later
				}
			}

			// Step 6: If requires brokering, start the user credential brokering flow
			if (
				credentialController.requires_brokering &&
				resourceServerCredential.id
			) {
				const brokeringResponse = await startBrokeringMutation.mutateAsync({
					params: {
						path: {
							provider_controller_type_id: providerTypeId,
							credential_controller_type_id: credentialController.type_id,
						},
					},
					body: {
						provider_instance_id: providerInstance.id,
					},
				});

				console.log("Brokering response:", brokeringResponse);

				// Handle JSON responses with redirect information
				if (
					brokeringResponse &&
					typeof brokeringResponse === "object" &&
					"type" in brokeringResponse
				) {
					if (
						brokeringResponse.type === "broker_state" &&
						"action" in brokeringResponse
					) {
						const action = brokeringResponse.action as any;
						if (typeof action === "object" && action && "Redirect" in action) {
							const redirectUrl = action.Redirect.url;
							console.log("Redirecting to:", redirectUrl);
							window.location.href = redirectUrl;
							return;
						}
					}

					// If response indicates completion (user_credential type), continue
					if (brokeringResponse.type === "user_credential") {
						console.log("User credential created successfully");
					}
				}
			}

			// Success - invalidate queries to refresh UI
			invalidateDataByControllerTypeId(queryClient, providerTypeId);
			invalidateFunctionInstancesData(queryClient);
			setIsSubmitting(false);
			onSuccess?.();
		} catch (error) {
			console.error("Error creating credentials:", error);
			setFormErrors([
				error instanceof Error
					? error.message
					: "Failed to create credentials. Please try again.",
			]);
			setIsSubmitting(false);
		}
	};

	const handleError = (errors: unknown) => {
		console.log("Form errors:", errors);
		const errorMessages = Array.isArray(errors)
			? errors.map((err: any) => err.message || "Invalid field")
			: ["Form validation failed"];
		setFormErrors(errorMessages);
	};

	const submitButtonText = credentialController.requires_brokering
		? "Save, and configure user credentials"
		: showUserCredentialForm
			? "Save configuration"
			: "Save and enable function";

	// Custom field template for consistent styling across all schema fields
	const CustomFieldTemplate = (props: any) => {
		const {
			id,
			classNames,
			label,
			help,
			required,
			description,
			errors,
			children,
		} = props;

		return (
			<div className={classNames}>
				{label && (
					<label htmlFor={id} className="block text-sm font-medium mb-2">
						{label}
						{required && <span className="text-red-600 ml-1">*</span>}
					</label>
				)}
				{description && (
					<p className="text-sm text-muted-foreground mb-2">{description}</p>
				)}
				{children}
				{errors}
				{help}
			</div>
		);
	};

	// Custom object field template to hide title and divider
	const CustomObjectFieldTemplate = (props: any) => {
		return (
			<div>
				{props.properties.map((element: any) => (
					<div key={element.name}>{element.content}</div>
				))}
			</div>
		);
	};

	// Custom title field template to hide title completely
	const CustomTitleFieldTemplate = () => {
		return null;
	};

	// Build UI schema to customize field rendering
	const properties =
		(combinedSchema.properties as Record<string, JsonSchema>) || {};
	const resourceServerProperties =
		(resourceServerSchema.properties as Record<string, JsonSchema>) || {};
	const userCredentialProperties =
		showUserCredentialForm && userCredentialSchema
			? (userCredentialSchema.properties as Record<string, JsonSchema>) || {}
			: {};

	const uiSchema: Record<string, unknown> = {
		"ui:submitButtonOptions": {
			submitText: isSubmitting ? "Submitting..." : submitButtonText,
			norender: false,
			props: {
				disabled: isSubmitting,
			},
		},
		"ui:FieldTemplate": CustomFieldTemplate,
		"ui:ObjectFieldTemplate": CustomObjectFieldTemplate,
		"ui:TitleFieldTemplate": CustomTitleFieldTemplate,
	};

	// Build initial form data from existing credentials
	const initialFormData: Record<string, any> = {};

	// Populate from existing resource server credential
	if (
		shouldPopulateFromExistingResourceServer &&
		existingResourceServerCredential?.value
	) {
		const existingValue = existingResourceServerCredential.value as Record<
			string,
			any
		>;

		// Copy values, replacing password fields with undefined
		Object.keys(resourceServerProperties).forEach((key) => {
			const prop = resourceServerProperties[key];
			const propFormat = prop.format as string | undefined;
			const isPasswordField =
				propFormat === "password" ||
				key.toLowerCase().includes("secret") ||
				key.toLowerCase().includes("password");

			if (key in existingValue) {
				// For password fields, use undefined as placeholder
				initialFormData[key] = isPasswordField ? undefined : existingValue[key];
			}
		});
	}

	// Populate from existing user credential
	if (
		showUserCredentialForm &&
		shouldPopulateFromExistingUserCredential &&
		existingUserCredential?.value
	) {
		const existingValue = existingUserCredential.value as Record<string, any>;

		// Copy values, replacing password fields with undefined
		Object.keys(userCredentialProperties).forEach((key) => {
			const prop = userCredentialProperties[key];
			const propFormat = prop.format as string | undefined;
			const isPasswordField =
				propFormat === "password" ||
				key.toLowerCase().includes("secret") ||
				key.toLowerCase().includes("password");

			if (key in existingValue) {
				// For password fields, use undefined as placeholder
				initialFormData[key] = isPasswordField ? undefined : existingValue[key];
			}
		});
	}

	// Add spacing and better styling for each field
	Object.keys(properties).forEach((key) => {
		const prop = properties[key];
		const propFormat = prop.format as string | undefined;

		uiSchema[key] = {
			"ui:classNames": "mb-6",
			...(propFormat === "password" && {
				"ui:widget": "password",
				"ui:placeholder": "***",
			}),
			...(prop.type === "object" && {
				"ui:widget": "textarea",
				"ui:options": {
					rows: 5,
				},
			}),
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
					<h4 className="text-sm font-semibold text-destructive mb-2">
						Form Errors:
					</h4>
					<ul className="list-disc list-inside space-y-1">
						{formErrors.map((error, idx) => (
							<li key={idx} className="text-sm text-destructive">
								{error}
							</li>
						))}
					</ul>
				</div>
			)}

			{/* Account Name Input - rendered before schema fields */}
			<div className="mb-6 max-w-2xl">
				<label
					htmlFor="account-name"
					className="block text-sm font-medium mb-2"
				>
					Account Name <span className="text-red-600 ml-1">*</span>
				</label>
				<p className="text-sm text-muted-foreground mb-2">
					This name will help you identify this provider account later
				</p>
				<Input
					id="account-name"
					type="text"
					value={accountName}
					onChange={(e) => setAccountName(e.target.value)}
					placeholder="Enter a name for this account (e.g., Work Gmail, Personal Notion)"
					disabled={isSubmitting}
				/>
			</div>

			{/* Resource Server Configuration Form */}
			<div className="rjsf-form-wrapper">
				<Form
					schema={combinedSchema}
					validator={validator}
					onSubmit={handleSubmit}
					onError={handleError}
					uiSchema={uiSchema}
					formData={initialFormData}
					showErrorList={false}
				/>
			</div>
		</div>
	);
};
