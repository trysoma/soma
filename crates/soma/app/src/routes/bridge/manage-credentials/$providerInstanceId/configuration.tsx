"use client";
import { createFileRoute, useParams, useNavigate } from '@tanstack/react-router'
import { ConfigurationForm } from "@/components/bridge/configuration-form";
import { LINKS } from "@/lib/links";
import $api from '@/lib/api-client.client';

export const Route = createFileRoute('/bridge/manage-credentials/$providerInstanceId/configuration')({
  component: RouteComponent,
})

function RouteComponent() {
  const { providerInstanceId } = useParams({ from: '/bridge/manage-credentials/$providerInstanceId/configuration' });
  const navigate = useNavigate();

  // Query the specific provider instance with all its details
  const {
    data: providerInstanceData,
  } = $api.useQuery("get", "/api/bridge/v1/provider/{provider_instance_id}", {
    params: {
      path: {
        provider_instance_id: providerInstanceId,
      },
    },
  });

  const instance = providerInstanceData?.provider_instance;
  const providerController = providerInstanceData?.controller;
  const resourceServerCredential = providerInstanceData?.resource_server_credential;
  const userCredential = providerInstanceData?.user_credential;

  const handleSuccess = () => {
    navigate({ to: LINKS.BRIDGE_MANAGE_CREDENTIALS() });
  };

  if (!instance || !providerController) {
    return null;
  }

  return (
    <div className="p-6 mt-0">
      <ConfigurationForm
        provider={providerController}
        existingProviderInstance={{
          id: instance.id,
          credential_controller_type_id: instance.credential_controller_type_id,
          display_name: instance.display_name,
        }}
        existingResourceServerCredential={resourceServerCredential}
        existingUserCredential={userCredential}
        onSuccess={handleSuccess}
      />
    </div>
  );
}
