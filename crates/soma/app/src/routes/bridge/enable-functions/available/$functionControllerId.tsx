"use client";
import { createFileRoute, useNavigate, useParams, Link, useLocation } from '@tanstack/react-router'
import { Package } from "lucide-react";
import { SlideOutPanel } from "@/components/ui/slide-out-panel";
import { SlideOutTabs } from "@/components/ui/slide-out-tabs";
import { LINKS } from "@/lib/links";
import $api from '@/lib/api-client.client';
import { useMemo } from "react";
import type { components } from "@/@types/openapi";

type JsonSchema = components["schemas"]["JsonSchema"];

interface AvailableFunction {
	id: string;
	providerTypeId: string;
	providerName: string;
	functionName: string;
	documentation: string;
	parametersSchema: JsonSchema;
	outputSchema: JsonSchema;
	categories: string[];
}

export const Route = createFileRoute('/bridge/enable-functions/available/$functionControllerId')({
  component: RouteComponent,
})

function RouteComponent() {
  const { functionControllerId } = useParams({ from: '/bridge/enable-functions/available/$functionControllerId' });
  const navigate = useNavigate();
  const location = useLocation();

  // Query available providers
  const {
    data: availableProviders,
    isLoading: isLoadingProviders,
  } = $api.useQuery("get", "/api/bridge/v1/available-providers", {
    params: {
      query: {
        page_size: 1000,
      },
    },
  });

  // Find the function and provider
  const { func, provider } = useMemo(() => {
    if (!availableProviders?.items) return { func: null, provider: null };

    let foundFunc: AvailableFunction | null = null;
    let foundProvider: components["schemas"]["ProviderControllerSerialized"] | null = null;

    for (const prov of availableProviders.items) {
      const fn = prov.functions.find((f) => f.type_id === functionControllerId);
      if (fn) {
        foundFunc = {
          id: fn.type_id,
          providerTypeId: prov.type_id,
          providerName: prov.name,
          functionName: fn.name,
          documentation: fn.documentation,
          parametersSchema: fn.parameters,
          outputSchema: fn.output,
          categories: prov.categories || [],
        };
        foundProvider = prov;
        break;
      }
    }

    return { func: foundFunc, provider: foundProvider };
  }, [availableProviders, functionControllerId]);

  const handleClose = () => {
    navigate({ to: LINKS.BRIDGE_ENABLE_FUNCTIONS() });
  };

  // Query enabled provider instances for this function
  const {
    data: enabledInstancesData,
  } = $api.useQuery("get", "/api/bridge/v1/provider/grouped-by-function", {
    params: {
      query: {
        page_size: 1000,
        provider_controller_type_id: provider?.type_id,
        function_category: null,
      },
    },
  }, {
    enabled: !!provider,
  });

  // Find this specific function's enabled instances
  const enabledInstances = useMemo(() => {
    if (!enabledInstancesData?.items) return [];
    const functionData = enabledInstancesData.items.find(
      (item) => item.function_controller.type_id === functionControllerId
    );
    return functionData?.provider_instances || [];
  }, [enabledInstancesData, functionControllerId]);

  const hasEnabledInstances = enabledInstances.length > 0;

  // Determine current tab from pathname
  const getCurrentTab = () => {
    if (location.pathname.includes('/function_documentation')) return 'function';
    if (location.pathname.includes('/provider_documentation')) return 'provider';
    if (location.pathname.includes('/configure')) return 'configure';
    if (location.pathname.includes('/test')) return 'test';
    return 'function';
  };

  if (isLoadingProviders || !func || !provider) {
    return null;
  }

  const tabs = [
    {
      value: 'function',
      label: 'Fn Documentation',
      pathPattern: '/function_documentation',
      component: (
        <Link to={LINKS.BRIDGE_ENABLE_FUNCTIONS_FUNCTION(functionControllerId)}>
          Fn Documentation
        </Link>
      ),
    },
    {
      value: 'provider',
      label: 'Provider Documentation',
      pathPattern: '/provider_documentation',
      component: (
        <Link to={LINKS.BRIDGE_ENABLE_FUNCTIONS_PROVIDER(functionControllerId)}>
          Provider Documentation
        </Link>
      ),
    },
    {
      value: 'configure',
      label: 'Configure',
      pathPattern: '/configure',
      component: (
        <Link to={LINKS.BRIDGE_ENABLE_FUNCTIONS_CONFIGURE(functionControllerId)}>
          Configure
        </Link>
      ),
    },
    ...(hasEnabledInstances ? [{
      value: 'test',
      label: 'Test',
      pathPattern: '/test',
      component: (
        <Link to={LINKS.BRIDGE_ENABLE_FUNCTIONS_TEST(functionControllerId)}>
          Test
        </Link>
      ),
    }] : []),
  ];

  return (
    <SlideOutPanel
      onClose={handleClose}
      title={func.functionName}
      subtitle={func.providerName}
      icon={<Package className="h-5 w-5" />}
    >
      <SlideOutTabs tabs={tabs} getCurrentTab={getCurrentTab} className={hasEnabledInstances ? "grid-cols-4" : "grid-cols-3"} />
    </SlideOutPanel>
  );
}
