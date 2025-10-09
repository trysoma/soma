import createClient from "openapi-fetch";
import type { paths } from "@/@types/openapi.ts";

export const getBaseUrl = () => {
  let baseUrl = "";

  if (typeof window !== "undefined") {
    baseUrl = `${window.location.protocol}//${window.location.hostname}`;

    if (window.location.port) {
      baseUrl += `:${window.location.port}`;
    }
  } else if (process != null) {
    // TODO: need to update nodejs types
    baseUrl = process.env.BASE_API_URL || "";
  } else {
    baseUrl = "";
  }

  return baseUrl;
};

const client = createClient<paths>({
  credentials: "include",
  baseUrl: getBaseUrl(),
});

export default client;
