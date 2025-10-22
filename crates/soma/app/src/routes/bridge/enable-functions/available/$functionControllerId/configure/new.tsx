"use client";
import { createFileRoute, useParams, useNavigate } from '@tanstack/react-router'
import { ConfigurationForm } from "@/components/bridge/configuration-form";
import { LINKS } from "@/lib/links";
import $api from '@/lib/api-client.client';
import { useMemo } from "react";

export const Route = createFileRoute('/bridge/enable-functions/available/$functionControllerId/configure/new')({
  component: RouteComponent,
})

export function RouteComponent() {
  const { functionControllerId } = useParams({ from: '/bridge/enable-functions/available/$functionControllerId/configure' });
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

  const handleSuccess = () => {
    // After successfully creating new credentials, redirect to existing tab
    navigate({ to: LINKS.BRIDGE_ENABLE_FUNCTIONS_CONFIGURE_EXISTING(functionControllerId) });
  };

  if (!provider) {
    return null;
  }

  return (
    <ConfigurationForm
      provider={provider}
      functionControllerId={functionControllerId}
      onSuccess={handleSuccess}
    />
  );
}
