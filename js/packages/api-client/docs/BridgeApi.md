# BridgeApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createProviderInstance**](BridgeApi.md#createproviderinstance) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id} | Create provider |
| [**createResourceServerCredential**](BridgeApi.md#createresourceservercredential) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/resource-server | Create resource server credential |
| [**createUserCredential**](BridgeApi.md#createusercredential) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential | Create user credential |
| [**deleteProviderInstance**](BridgeApi.md#deleteproviderinstance) | **DELETE** /api/bridge/v1/provider/{provider_instance_id} | Delete provider |
| [**disableFunction**](BridgeApi.md#disablefunction) | **POST** /api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/disable | Disable function |
| [**enableFunction**](BridgeApi.md#enablefunction) | **POST** /api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/enable | Enable function |
| [**encryptResourceServerConfiguration**](BridgeApi.md#encryptresourceserverconfiguration) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/resource-server/encrypt | Encrypt resource server config |
| [**encryptUserCredentialConfiguration**](BridgeApi.md#encryptusercredentialconfiguration) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential/encrypt | Encrypt user credential config |
| [**getFunctionInstancesOpenapiSpec**](BridgeApi.md#getfunctioninstancesopenapispec) | **GET** /api/bridge/v1/function-instances/openapi.json | Get function OpenAPI spec |
| [**getProviderInstance**](BridgeApi.md#getproviderinstance) | **GET** /api/bridge/v1/provider/{provider_instance_id} | Get provider |
| [**invokeFunction**](BridgeApi.md#invokefunction) | **POST** /api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/invoke | Invoke function |
| [**listAvailableProviders**](BridgeApi.md#listavailableproviders) | **GET** /api/bridge/v1/available-providers | List providers |
| [**listFunctionInstances**](BridgeApi.md#listfunctioninstances) | **GET** /api/bridge/v1/function-instances | List function instances |
| [**listProviderInstances**](BridgeApi.md#listproviderinstances) | **GET** /api/bridge/v1/provider | List provider instances |
| [**listProviderInstancesGroupedByFunction**](BridgeApi.md#listproviderinstancesgroupedbyfunction) | **GET** /api/bridge/v1/provider/grouped-by-function | List providers by function |
| [**listenToMcpSse**](BridgeApi.md#listentomcpsse) | **GET** /api/bridge/v1/mcp | MCP SSE connection |
| [**resumeUserCredentialBrokering**](BridgeApi.md#resumeusercredentialbrokering) | **GET** /api/bridge/v1/generic-oauth-callback | OAuth callback |
| [**startUserCredentialBrokering**](BridgeApi.md#startusercredentialbrokering) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential/broker | Start credential brokering |
| [**triggerMcpMessage**](BridgeApi.md#triggermcpmessage) | **POST** /api/bridge/v1/mcp | Send MCP message |
| [**updateProviderInstance**](BridgeApi.md#updateproviderinstance) | **PATCH** /api/bridge/v1/provider/{provider_instance_id} | Update provider |



## createProviderInstance

> ProviderInstanceSerialized createProviderInstance(providerControllerTypeId, credentialControllerTypeId, createProviderInstanceParamsInner)

Create provider

Create a new provider instance with the specified configuration

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { CreateProviderInstanceRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  const body = {
    // string
    providerControllerTypeId: providerControllerTypeId_example,
    // string
    credentialControllerTypeId: credentialControllerTypeId_example,
    // CreateProviderInstanceParamsInner
    createProviderInstanceParamsInner: ...,
  } satisfies CreateProviderInstanceRequest;

  try {
    const data = await api.createProviderInstance(body);
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
| **providerControllerTypeId** | `string` |  | [Defaults to `undefined`] |
| **credentialControllerTypeId** | `string` |  | [Defaults to `undefined`] |
| **createProviderInstanceParamsInner** | [CreateProviderInstanceParamsInner](CreateProviderInstanceParamsInner.md) |  | |

### Return type

[**ProviderInstanceSerialized**](ProviderInstanceSerialized.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Create provider instance |  -  |
| **400** | Bad Request |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## createResourceServerCredential

> ResourceServerCredentialSerialized createResourceServerCredential(providerControllerTypeId, credentialControllerTypeId, createResourceServerCredentialParamsInner)

Create resource server credential

Create a new resource server credential

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { CreateResourceServerCredentialRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  const body = {
    // string | Provider controller type ID
    providerControllerTypeId: providerControllerTypeId_example,
    // string | Credential controller type ID
    credentialControllerTypeId: credentialControllerTypeId_example,
    // CreateResourceServerCredentialParamsInner
    createResourceServerCredentialParamsInner: ...,
  } satisfies CreateResourceServerCredentialRequest;

  try {
    const data = await api.createResourceServerCredential(body);
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
| **providerControllerTypeId** | `string` | Provider controller type ID | [Defaults to `undefined`] |
| **credentialControllerTypeId** | `string` | Credential controller type ID | [Defaults to `undefined`] |
| **createResourceServerCredentialParamsInner** | [CreateResourceServerCredentialParamsInner](CreateResourceServerCredentialParamsInner.md) |  | |

### Return type

[**ResourceServerCredentialSerialized**](ResourceServerCredentialSerialized.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Create resource server credential |  -  |
| **400** | Bad Request |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## createUserCredential

> UserCredentialSerialized createUserCredential(providerControllerTypeId, credentialControllerTypeId, createUserCredentialParamsInner)

Create user credential

Create a new user credential

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { CreateUserCredentialRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  const body = {
    // string | Provider controller type ID
    providerControllerTypeId: providerControllerTypeId_example,
    // string | Credential controller type ID
    credentialControllerTypeId: credentialControllerTypeId_example,
    // CreateUserCredentialParamsInner
    createUserCredentialParamsInner: ...,
  } satisfies CreateUserCredentialRequest;

  try {
    const data = await api.createUserCredential(body);
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
| **providerControllerTypeId** | `string` | Provider controller type ID | [Defaults to `undefined`] |
| **credentialControllerTypeId** | `string` | Credential controller type ID | [Defaults to `undefined`] |
| **createUserCredentialParamsInner** | [CreateUserCredentialParamsInner](CreateUserCredentialParamsInner.md) |  | |

### Return type

[**UserCredentialSerialized**](UserCredentialSerialized.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Create user credential |  -  |
| **400** | Bad Request |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## deleteProviderInstance

> any deleteProviderInstance(providerInstanceId)

Delete provider

Delete a provider instance by its unique identifier

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { DeleteProviderInstanceRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  const body = {
    // string | Provider instance ID
    providerInstanceId: providerInstanceId_example,
  } satisfies DeleteProviderInstanceRequest;

  try {
    const data = await api.deleteProviderInstance(body);
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
| **providerInstanceId** | `string` | Provider instance ID | [Defaults to `undefined`] |

### Return type

**any**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Delete provider instance |  -  |
| **400** | Bad Request |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## disableFunction

> any disableFunction(providerInstanceId, functionControllerTypeId)

Disable function

Disable a function for a provider instance

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { DisableFunctionRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  const body = {
    // string | Provider instance ID
    providerInstanceId: providerInstanceId_example,
    // string | Function controller type ID
    functionControllerTypeId: functionControllerTypeId_example,
  } satisfies DisableFunctionRequest;

  try {
    const data = await api.disableFunction(body);
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
| **providerInstanceId** | `string` | Provider instance ID | [Defaults to `undefined`] |
| **functionControllerTypeId** | `string` | Function controller type ID | [Defaults to `undefined`] |

### Return type

**any**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Disable function |  -  |
| **400** | Bad Request |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## enableFunction

> FunctionInstanceSerialized enableFunction(providerInstanceId, functionControllerTypeId, body)

Enable function

Enable a function for a provider instance

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { EnableFunctionRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  const body = {
    // string | Provider instance ID
    providerInstanceId: providerInstanceId_example,
    // string | Function controller type ID
    functionControllerTypeId: functionControllerTypeId_example,
    // object
    body: Object,
  } satisfies EnableFunctionRequest;

  try {
    const data = await api.enableFunction(body);
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
| **providerInstanceId** | `string` | Provider instance ID | [Defaults to `undefined`] |
| **functionControllerTypeId** | `string` | Function controller type ID | [Defaults to `undefined`] |
| **body** | `object` |  | |

### Return type

[**FunctionInstanceSerialized**](FunctionInstanceSerialized.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Enable function |  -  |
| **400** | Bad Request |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## encryptResourceServerConfiguration

> any encryptResourceServerConfiguration(providerControllerTypeId, credentialControllerTypeId, encryptCredentialConfigurationParamsInner)

Encrypt resource server config

Encrypt a resource server credential configuration before storage

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { EncryptResourceServerConfigurationRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  const body = {
    // string | Provider controller type ID
    providerControllerTypeId: providerControllerTypeId_example,
    // string | Credential controller type ID
    credentialControllerTypeId: credentialControllerTypeId_example,
    // EncryptCredentialConfigurationParamsInner
    encryptCredentialConfigurationParamsInner: ...,
  } satisfies EncryptResourceServerConfigurationRequest;

  try {
    const data = await api.encryptResourceServerConfiguration(body);
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
| **providerControllerTypeId** | `string` | Provider controller type ID | [Defaults to `undefined`] |
| **credentialControllerTypeId** | `string` | Credential controller type ID | [Defaults to `undefined`] |
| **encryptCredentialConfigurationParamsInner** | [EncryptCredentialConfigurationParamsInner](EncryptCredentialConfigurationParamsInner.md) |  | |

### Return type

**any**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Encrypt resource server configuration |  -  |
| **400** | Bad Request |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## encryptUserCredentialConfiguration

> any encryptUserCredentialConfiguration(providerControllerTypeId, credentialControllerTypeId, encryptCredentialConfigurationParamsInner)

Encrypt user credential config

Encrypt a user credential configuration before storage

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { EncryptUserCredentialConfigurationRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  const body = {
    // string | Provider controller type ID
    providerControllerTypeId: providerControllerTypeId_example,
    // string | Credential controller type ID
    credentialControllerTypeId: credentialControllerTypeId_example,
    // EncryptCredentialConfigurationParamsInner
    encryptCredentialConfigurationParamsInner: ...,
  } satisfies EncryptUserCredentialConfigurationRequest;

  try {
    const data = await api.encryptUserCredentialConfiguration(body);
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
| **providerControllerTypeId** | `string` | Provider controller type ID | [Defaults to `undefined`] |
| **credentialControllerTypeId** | `string` | Credential controller type ID | [Defaults to `undefined`] |
| **encryptCredentialConfigurationParamsInner** | [EncryptCredentialConfigurationParamsInner](EncryptCredentialConfigurationParamsInner.md) |  | |

### Return type

**any**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Encrypt user credential configuration |  -  |
| **400** | Bad Request |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getFunctionInstancesOpenapiSpec

> string getFunctionInstancesOpenapiSpec()

Get function OpenAPI spec

Get the OpenAPI specification for all function instances

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { GetFunctionInstancesOpenapiSpecRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  try {
    const data = await api.getFunctionInstancesOpenapiSpec();
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

**string**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `text/plain`, `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Get function instances openapi spec |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getProviderInstance

> ProviderInstanceSerializedWithEverything getProviderInstance(providerInstanceId)

Get provider

Retrieve a provider instance by its unique identifier

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { GetProviderInstanceRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  const body = {
    // string | Provider instance ID
    providerInstanceId: providerInstanceId_example,
  } satisfies GetProviderInstanceRequest;

  try {
    const data = await api.getProviderInstance(body);
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
| **providerInstanceId** | `string` | Provider instance ID | [Defaults to `undefined`] |

### Return type

[**ProviderInstanceSerializedWithEverything**](ProviderInstanceSerializedWithEverything.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Get provider instance |  -  |
| **400** | Bad Request |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## invokeFunction

> InvokeResult invokeFunction(providerInstanceId, functionControllerTypeId, invokeFunctionParamsInner)

Invoke function

Invoke a function on a provider instance

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { InvokeFunctionRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  const body = {
    // string | Provider instance ID
    providerInstanceId: providerInstanceId_example,
    // string | Function controller type ID
    functionControllerTypeId: functionControllerTypeId_example,
    // InvokeFunctionParamsInner
    invokeFunctionParamsInner: ...,
  } satisfies InvokeFunctionRequest;

  try {
    const data = await api.invokeFunction(body);
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
| **providerInstanceId** | `string` | Provider instance ID | [Defaults to `undefined`] |
| **functionControllerTypeId** | `string` | Function controller type ID | [Defaults to `undefined`] |
| **invokeFunctionParamsInner** | [InvokeFunctionParamsInner](InvokeFunctionParamsInner.md) |  | |

### Return type

[**InvokeResult**](InvokeResult.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Invoke function |  -  |
| **400** | Bad Request |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listAvailableProviders

> ProviderControllerSerializedPaginatedResponse listAvailableProviders(pageSize, nextPageToken)

List providers

List all available provider types that can be instantiated

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { ListAvailableProvidersRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  const body = {
    // number
    pageSize: 789,
    // string (optional)
    nextPageToken: nextPageToken_example,
  } satisfies ListAvailableProvidersRequest;

  try {
    const data = await api.listAvailableProviders(body);
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
| **pageSize** | `number` |  | [Defaults to `undefined`] |
| **nextPageToken** | `string` |  | [Optional] [Defaults to `undefined`] |

### Return type

[**ProviderControllerSerializedPaginatedResponse**](ProviderControllerSerializedPaginatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | List available providers |  -  |
| **400** | Bad Request |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listFunctionInstances

> FunctionInstanceSerializedPaginatedResponse listFunctionInstances(pageSize, nextPageToken, providerInstanceId)

List function instances

List all function instances with optional filtering by provider instance

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { ListFunctionInstancesRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  const body = {
    // number
    pageSize: 789,
    // string (optional)
    nextPageToken: nextPageToken_example,
    // string (optional)
    providerInstanceId: providerInstanceId_example,
  } satisfies ListFunctionInstancesRequest;

  try {
    const data = await api.listFunctionInstances(body);
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
| **pageSize** | `number` |  | [Defaults to `undefined`] |
| **nextPageToken** | `string` |  | [Optional] [Defaults to `undefined`] |
| **providerInstanceId** | `string` |  | [Optional] [Defaults to `undefined`] |

### Return type

[**FunctionInstanceSerializedPaginatedResponse**](FunctionInstanceSerializedPaginatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | List function instances |  -  |
| **400** | Bad Request |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listProviderInstances

> ProviderInstanceListItemPaginatedResponse listProviderInstances(pageSize, nextPageToken, status, providerControllerTypeId)

List provider instances

List all provider instances with optional filtering by status and provider type

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { ListProviderInstancesRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  const body = {
    // number
    pageSize: 789,
    // string (optional)
    nextPageToken: nextPageToken_example,
    // string (optional)
    status: status_example,
    // string (optional)
    providerControllerTypeId: providerControllerTypeId_example,
  } satisfies ListProviderInstancesRequest;

  try {
    const data = await api.listProviderInstances(body);
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
| **pageSize** | `number` |  | [Defaults to `undefined`] |
| **nextPageToken** | `string` |  | [Optional] [Defaults to `undefined`] |
| **status** | `string` |  | [Optional] [Defaults to `undefined`] |
| **providerControllerTypeId** | `string` |  | [Optional] [Defaults to `undefined`] |

### Return type

[**ProviderInstanceListItemPaginatedResponse**](ProviderInstanceListItemPaginatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | List provider instances |  -  |
| **400** | Bad Request |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listProviderInstancesGroupedByFunction

> FunctionInstanceConfigPaginatedResponse listProviderInstancesGroupedByFunction(pageSize, nextPageToken, providerControllerTypeId, functionCategory)

List providers by function

List provider instances grouped by their associated functions

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { ListProviderInstancesGroupedByFunctionRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  const body = {
    // number
    pageSize: 789,
    // string (optional)
    nextPageToken: nextPageToken_example,
    // string (optional)
    providerControllerTypeId: providerControllerTypeId_example,
    // string (optional)
    functionCategory: functionCategory_example,
  } satisfies ListProviderInstancesGroupedByFunctionRequest;

  try {
    const data = await api.listProviderInstancesGroupedByFunction(body);
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
| **pageSize** | `number` |  | [Defaults to `undefined`] |
| **nextPageToken** | `string` |  | [Optional] [Defaults to `undefined`] |
| **providerControllerTypeId** | `string` |  | [Optional] [Defaults to `undefined`] |
| **functionCategory** | `string` |  | [Optional] [Defaults to `undefined`] |

### Return type

[**FunctionInstanceConfigPaginatedResponse**](FunctionInstanceConfigPaginatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | List provider instances grouped by function |  -  |
| **400** | Bad Request |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listenToMcpSse

> listenToMcpSse()

MCP SSE connection

Establish Server-Sent Events (SSE) connection for MCP protocol communication

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { ListenToMcpSseRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  try {
    const data = await api.listenToMcpSse();
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
| **200** | MCP server running |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## resumeUserCredentialBrokering

> UserCredentialBrokeringResponse resumeUserCredentialBrokering(state, code, error, errorDescription)

OAuth callback

Handle OAuth callback to complete user credential brokering flow

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { ResumeUserCredentialBrokeringRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  const body = {
    // string | OAuth state parameter (optional)
    state: state_example,
    // string | OAuth authorization code (optional)
    code: code_example,
    // string | OAuth error code (optional)
    error: error_example,
    // string | OAuth error description (optional)
    errorDescription: errorDescription_example,
  } satisfies ResumeUserCredentialBrokeringRequest;

  try {
    const data = await api.resumeUserCredentialBrokering(body);
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
| **state** | `string` | OAuth state parameter | [Optional] [Defaults to `undefined`] |
| **code** | `string` | OAuth authorization code | [Optional] [Defaults to `undefined`] |
| **error** | `string` | OAuth error code | [Optional] [Defaults to `undefined`] |
| **errorDescription** | `string` | OAuth error description | [Optional] [Defaults to `undefined`] |

### Return type

[**UserCredentialBrokeringResponse**](UserCredentialBrokeringResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Generic OAuth callback |  -  |
| **400** | Bad Request |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## startUserCredentialBrokering

> UserCredentialBrokeringResponse startUserCredentialBrokering(providerControllerTypeId, credentialControllerTypeId, startUserCredentialBrokeringParamsInner)

Start credential brokering

Start the OAuth flow for user credential brokering

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { StartUserCredentialBrokeringRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  const body = {
    // string | Provider controller type ID
    providerControllerTypeId: providerControllerTypeId_example,
    // string | Credential controller type ID
    credentialControllerTypeId: credentialControllerTypeId_example,
    // StartUserCredentialBrokeringParamsInner
    startUserCredentialBrokeringParamsInner: ...,
  } satisfies StartUserCredentialBrokeringRequest;

  try {
    const data = await api.startUserCredentialBrokering(body);
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
| **providerControllerTypeId** | `string` | Provider controller type ID | [Defaults to `undefined`] |
| **credentialControllerTypeId** | `string` | Credential controller type ID | [Defaults to `undefined`] |
| **startUserCredentialBrokeringParamsInner** | [StartUserCredentialBrokeringParamsInner](StartUserCredentialBrokeringParamsInner.md) |  | |

### Return type

[**UserCredentialBrokeringResponse**](UserCredentialBrokeringResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Start user credential brokering |  -  |
| **400** | Bad Request |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## triggerMcpMessage

> triggerMcpMessage(body)

Send MCP message

Send a JSON-RPC message to the MCP server

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { TriggerMcpMessageRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  const body = {
    // object
    body: Object,
  } satisfies TriggerMcpMessageRequest;

  try {
    const data = await api.triggerMcpMessage(body);
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
- **Accept**: Not defined


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | MCP server running |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## updateProviderInstance

> any updateProviderInstance(providerInstanceId, updateProviderInstanceParamsInner)

Update provider

Update an existing provider instance configuration

### Example

```ts
import {
  Configuration,
  BridgeApi,
} from '@trysoma/api-client';
import type { UpdateProviderInstanceRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new BridgeApi();

  const body = {
    // string | Provider instance ID
    providerInstanceId: providerInstanceId_example,
    // UpdateProviderInstanceParamsInner
    updateProviderInstanceParamsInner: ...,
  } satisfies UpdateProviderInstanceRequest;

  try {
    const data = await api.updateProviderInstance(body);
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
| **providerInstanceId** | `string` | Provider instance ID | [Defaults to `undefined`] |
| **updateProviderInstanceParamsInner** | [UpdateProviderInstanceParamsInner](UpdateProviderInstanceParamsInner.md) |  | |

### Return type

**any**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Update provider instance |  -  |
| **400** | Bad Request |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

