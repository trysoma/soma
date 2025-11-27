# A2aApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**getAgentCard**](A2aApi.md#getagentcard) | **GET** /api/a2a/v1/.well-known/agent.json | Get agent card |
| [**getAgentDefinition**](A2aApi.md#getagentdefinition) | **GET** /api/a2a/v1/definition | Get agent definition |
| [**getExtendedAgentCard**](A2aApi.md#getextendedagentcard) | **GET** /api/a2a/v1/agent/authenticatedExtendedCard | Get extended agent card |
| [**handleJsonrpcRequest**](A2aApi.md#handlejsonrpcrequest) | **POST** /api/a2a/v1 | Handle JSON-RPC |



## getAgentCard

> object getAgentCard()

Get agent card

Get the agent card describing agent capabilities and metadata

### Example

```ts
import {
  Configuration,
  A2aApi,
} from '@trysoma/api-client';
import type { GetAgentCardRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new A2aApi();

  try {
    const data = await api.getAgentCard();
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
| **200** | Successful response |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getAgentDefinition

> SomaAgentDefinition getAgentDefinition()

Get agent definition

Get the agent definition (capabilities and metadata)

### Example

```ts
import {
  Configuration,
  A2aApi,
} from '@trysoma/api-client';
import type { GetAgentDefinitionRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new A2aApi();

  try {
    const data = await api.getAgentDefinition();
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

[**SomaAgentDefinition**](SomaAgentDefinition.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Agent definition |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getExtendedAgentCard

> object getExtendedAgentCard()

Get extended agent card

Get the authenticated extended agent card with additional metadata

### Example

```ts
import {
  Configuration,
  A2aApi,
} from '@trysoma/api-client';
import type { GetExtendedAgentCardRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new A2aApi();

  try {
    const data = await api.getExtendedAgentCard();
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
| **200** | Successful response |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## handleJsonrpcRequest

> handleJsonrpcRequest(body)

Handle JSON-RPC

Handle JSON-RPC requests for agent-to-agent communication (tasks, messages, etc.)

### Example

```ts
import {
  Configuration,
  A2aApi,
} from '@trysoma/api-client';
import type { HandleJsonrpcRequestRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new A2aApi();

  const body = {
    // object
    body: Object,
  } satisfies HandleJsonrpcRequestRequest;

  try {
    const data = await api.handleJsonrpcRequest(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **body** | `object` |  | |

### Return type

`void` (Empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Successful response |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

