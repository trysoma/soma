import { createFileRoute, useLocation, useNavigate, useParams } from '@tanstack/react-router'
import { useMemo } from 'react';
import $api from '@/lib/api-client.client';
import { LINKS } from '@/lib/links';


const RouteComponent = () => {
  const { functionControllerId } = useParams({ from: '/bridge/enable-functions/available/$functionControllerId/configure' });
  const location = useLocation();
  const navigate = useNavigate();

  // Query available providers
  const {
    data: availableProviders,
  } = $api.useQuery("get", "/api/bridge/v1/available-providers", {
    params: {
      query: {
        page_size: 1000,
      },
    },
  });

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
  const {
    data: providerInstancesData,
  } = $api.useQuery("get", "/api/bridge/v1/provider", {
    params: {
      query: {
        page_size: 1000,
        status: "active",
      },
    },
  }, {
    enabled: !!provider,
  });

  // Filter instances by provider controller type
  const existingProviderInstances = useMemo(() => {
    if (!providerInstancesData?.items || !provider) return [];
    return providerInstancesData.items.filter(
      (instance) => instance.provider_controller_type_id === provider.type_id
    );
  }, [providerInstancesData, provider]);

  const hasExistingProviders = existingProviderInstances.length > 0;
  
  if(!hasExistingProviders) {
    navigate({ to: LINKS.BRIDGE_ENABLE_FUNCTIONS_CONFIGURE_NEW(functionControllerId) });
    return null;
  }

  navigate({ to: LINKS.BRIDGE_ENABLE_FUNCTIONS_CONFIGURE_EXISTING(functionControllerId) });
  return null;
}

export const Route = createFileRoute(
  '/bridge/enable-functions/available/$functionControllerId/configure/',
)({
  component: RouteComponent,
})
