# InternalApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**getInternalRuntimeConfig**](InternalApi.md#getinternalruntimeconfig) | **GET** /_internal/v1/runtime_config | Get runtime config |
| [**healthCheck**](InternalApi.md#healthcheck) | **GET** /_internal/v1/health | Health check |
| [**triggerCodegen**](InternalApi.md#triggercodegen) | **POST** /_internal/v1/trigger_codegen | Trigger codegen |



## getInternalRuntimeConfig

> object getInternalRuntimeConfig()

Get runtime config

Get the current runtime configuration

### Example

```ts
import {
  Configuration,
  InternalApi,
} from '@trysoma/api-client';
import type { GetInternalRuntimeConfigRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new InternalApi();

  try {
    const data = await api.getInternalRuntimeConfig();
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters

This endpoint does not need any parameter.

### Return type

**object**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Runtime config |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## healthCheck

> healthCheck()

Health check

Check the health status of the service and SDK server connectivity

### Example

```ts
import {
  Configuration,
  InternalApi,
} from '@trysoma/api-client';
import type { HealthCheckRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new InternalApi();

  try {
    const data = await api.healthCheck();
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters

This endpoint does not need any parameter.

### Return type

`void` (Empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Service is healthy |  -  |
| **503** | Service unavailable - SDK server not ready |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## triggerCodegen

> TriggerCodegenResponse triggerCodegen()

Trigger codegen

Trigger code generation for the SDK

### Example

```ts
import {
  Configuration,
  InternalApi,
} from '@trysoma/api-client';
import type { TriggerCodegenRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new InternalApi();

  try {
    const data = await api.triggerCodegen();
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters

This endpoint does not need any parameter.

### Return type

[**TriggerCodegenResponse**](TriggerCodegenResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Codegen triggered successfully |  -  |
| **400** | Bad Request |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

