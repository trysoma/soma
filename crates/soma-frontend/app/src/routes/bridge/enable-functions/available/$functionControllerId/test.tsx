"use client";
import Form from "@rjsf/shadcn";
import validator from "@rjsf/validator-ajv8";
import { createFileRoute, useParams } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import $api from "@/lib/api-client.client";
import "@/styles/rjsf-overrides.css";

export const Route = createFileRoute(
	"/bridge/enable-functions/available/$functionControllerId/test",
)({
	component: RouteComponent,
});

function RouteComponent() {
	const { functionControllerId } = useParams({
		from: "/bridge/enable-functions/available/$functionControllerId/test",
	});

	const [selectedProviderInstanceId, setSelectedProviderInstanceId] =
		useState<string>("");
	const [formData, setFormData] = useState<any>({});
	const [responseData, setResponseData] = useState<any>(null);
	const [isInvoking, setIsInvoking] = useState(false);
	const [error, setError] = useState<string | null>(null);

	// Query available providers to get function schema
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

	// Find the function and provider
	const { func, provider } = useMemo(() => {
		if (!availableProviders?.items) return { func: null, provider: null };

		let foundFunc: any = null;
		let foundProvider: any = null;

		for (const prov of availableProviders.items) {
			const fn = prov.functions.find((f) => f.type_id === functionControllerId);
			if (fn) {
				foundFunc = fn;
				foundProvider = prov;
				break;
			}
		}

		return { func: foundFunc, provider: foundProvider };
	}, [availableProviders, functionControllerId]);

	// Query enabled provider instances for this function
	const { data: enabledInstancesData } = $api.useQuery(
		"get",
		"/api/mcp/v1/provider/grouped-by-function",
		{
			params: {
				query: {
					page_size: 1000,
					provider_controller_type_id: provider?.type_id,
					function_category: null,
				},
			},
		},
		{
			enabled: !!provider,
		},
	);

	// Find this specific function's enabled instances
	const enabledInstances = useMemo(() => {
		if (!enabledInstancesData?.items) return [];
		const functionData = enabledInstancesData.items.find(
			(item) => item.function_controller.type_id === functionControllerId,
		);
		return functionData?.provider_instances || [];
	}, [enabledInstancesData, functionControllerId]);

	// Invoke function mutation
	const invokeMutation = $api.useMutation(
		"post",
		"/api/mcp/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/invoke",
	);

	// Parse the JSON schema for function parameters
	// This MUST be before any early returns to satisfy Rules of Hooks
	const schema = useMemo(() => {
		if (!func) return {};
		const rawSchema = func.parameters;
		const schema = { ...rawSchema };
		delete schema.$schema;
		delete schema.title;
		return schema;
	}, [func]);

	// Build UI schema to customize field rendering
	// This MUST also be before any early returns to satisfy Rules of Hooks
	const uiSchema = useMemo(() => {
		const properties = (schema.properties as Record<string, any>) || {};

		// Custom field template for consistent styling
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

		const uiSchema: Record<string, unknown> = {
			"ui:submitButtonOptions": {
				submitText: isInvoking ? "Invoking..." : "Invoke Function",
				norender: false,
				props: {
					disabled: isInvoking || !selectedProviderInstanceId,
				},
			},
			"ui:FieldTemplate": CustomFieldTemplate,
			"ui:ObjectFieldTemplate": CustomObjectFieldTemplate,
			"ui:TitleFieldTemplate": CustomTitleFieldTemplate,
		};

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

		return uiSchema;
	}, [schema, isInvoking, selectedProviderInstanceId]);

	const handleSubmit = async (data: { formData?: unknown }) => {
		if (!selectedProviderInstanceId) {
			setError("Please select a provider instance");
			return;
		}

		setIsInvoking(true);
		setError(null);
		setResponseData(null);

		try {
			const response = await invokeMutation.mutateAsync({
				params: {
					path: {
						provider_instance_id: selectedProviderInstanceId,
						function_controller_type_id: functionControllerId,
					},
				},
				body: {
					params: data.formData || {},
				},
			});

			setResponseData(response);
		} catch (err) {
			console.error("Function invocation error:", err);
			setError(
				err instanceof Error ? err.message : "Failed to invoke function",
			);
		} finally {
			setIsInvoking(false);
		}
	};

	if (!func) {
		return (
			<div className="p-6">
				<p className="text-muted-foreground">Loading...</p>
			</div>
		);
	}

	if (enabledInstances.length === 0) {
		return (
			<div className="p-6">
				<p className="text-muted-foreground">
					No enabled credentials for this function. Please configure credentials
					first.
				</p>
			</div>
		);
	}

	return (
		<div className="p-6 space-y-6">
			{/* Provider Instance Selector */}
			<div className="mb-6 max-w-2xl">
				<label
					htmlFor="provider-instance"
					className="block text-sm font-medium mb-2"
				>
					Select Credentials <span className="text-red-600 ml-1">*</span>
				</label>
				<p className="text-sm text-muted-foreground mb-2">
					Choose which credentials to use for testing this function
				</p>
				<Select
					value={selectedProviderInstanceId}
					onValueChange={setSelectedProviderInstanceId}
				>
					<SelectTrigger className="w-full bg-white">
						<SelectValue placeholder="Select credentials..." />
					</SelectTrigger>
					<SelectContent>
						{enabledInstances.map((instance) => (
							<SelectItem
								key={instance.provider_instance.id}
								value={instance.provider_instance.id}
							>
								{instance.provider_instance.display_name}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</div>

			{/* Function Parameters Form */}
			<div className="space-y-2">
				<h3 className="text-sm font-medium">Function Parameters</h3>
				<div className="rjsf-form-wrapper">
					<Form
						schema={schema}
						validator={validator}
						onSubmit={handleSubmit}
						uiSchema={uiSchema}
						formData={formData}
						onChange={(e) => setFormData(e.formData)}
						showErrorList={false}
					/>
				</div>
			</div>

			{/* Error Display */}
			{error && (
				<div className="p-4 border border-destructive bg-destructive/10 rounded-lg">
					<h4 className="text-sm font-semibold text-destructive mb-2">Error</h4>
					<p className="text-sm text-destructive">{error}</p>
				</div>
			)}

			{/* Response Display */}
			{responseData !== null &&
				(() => {
					const isSuccess = responseData?.type === "success";
					const isError = responseData?.type === "error";

					// Parse error message if it's a JSON string
					let displayContent: any = responseData;
					if (isError && responseData?.message) {
						try {
							// Try to parse the message as JSON (it might be a JSON string)
							const parsed = JSON.parse(responseData.message);
							displayContent = parsed;
						} catch {
							// If parsing fails, use the message as-is
							displayContent = { message: responseData.message };
						}
					} else if (isSuccess) {
						// For success, remove the "type" field and show the rest (WrappedJsonValue is transparent)
						const { type: _, ...rest } = responseData;
						displayContent = Object.keys(rest).length > 0 ? rest : responseData;
					}

					const borderColor = isSuccess
						? "border-green-500"
						: isError
							? "border-destructive"
							: "border-destructive";
					const bgColor = isSuccess
						? "bg-green-50"
						: isError
							? "bg-destructive/10"
							: "bg-destructive/10";
					const textColor = isSuccess
						? "text-green-800"
						: isError
							? "text-destructive"
							: "text-destructive";

					return (
						<div className="space-y-2">
							<h3 className="text-sm font-medium">Response</h3>
							<div
								className={`p-4 border ${borderColor} ${bgColor} rounded-lg`}
							>
								<pre
									className={`text-xs overflow-auto max-h-[400px] whitespace-pre-wrap ${textColor}`}
								>
									{JSON.stringify(displayContent, null, 2)}
								</pre>
							</div>
						</div>
					);
				})()}
		</div>
	);
}
