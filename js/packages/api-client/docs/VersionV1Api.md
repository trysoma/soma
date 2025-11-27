# VersionV1Api

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createDataEncryptionKey**](VersionV1Api.md#createdataencryptionkey) | **POST** /api/encryption/v1/envelope/{envelope_id}/dek | Create data key |
| [**createDekAlias**](VersionV1Api.md#createdekaliasoperation) | **POST** /api/encryption/v1/dek/alias | Create DEK alias |
| [**createEnvelopeEncryptionKey**](VersionV1Api.md#createenvelopeencryptionkey) | **POST** /api/encryption/v1/envelope | Create envelope key |
| [**createProviderInstance**](VersionV1Api.md#createproviderinstance) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id} | Create provider |
| [**createResourceServerCredential**](VersionV1Api.md#createresourceservercredential) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/resource-server | Create resource server credential |
| [**createSecret**](VersionV1Api.md#createsecretoperation) | **POST** /api/secret/v1 | Create secret |
| [**createUserCredential**](VersionV1Api.md#createusercredential) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential | Create user credential |
| [**deleteDekAlias**](VersionV1Api.md#deletedekalias) | **DELETE** /api/encryption/v1/dek/alias/{alias} | Delete DEK alias |
| [**deleteProviderInstance**](VersionV1Api.md#deleteproviderinstance) | **DELETE** /api/bridge/v1/provider/{provider_instance_id} | Delete provider |
| [**deleteSecret**](VersionV1Api.md#deletesecret) | **DELETE** /api/secret/v1/{secret_id} | Delete secret |
| [**disableFunction**](VersionV1Api.md#disablefunction) | **POST** /api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/disable | Disable function |
| [**enableFunction**](VersionV1Api.md#enablefunction) | **POST** /api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/enable | Enable function |
| [**encryptResourceServerConfiguration**](VersionV1Api.md#encryptresourceserverconfiguration) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/resource-server/encrypt | Encrypt resource server config |
| [**encryptUserCredentialConfiguration**](VersionV1Api.md#encryptusercredentialconfiguration) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential/encrypt | Encrypt user credential config |
| [**getAgentCard**](VersionV1Api.md#getagentcard) | **GET** /api/a2a/v1/.well-known/agent.json | Get agent card |
| [**getAgentDefinition**](VersionV1Api.md#getagentdefinition) | **GET** /api/a2a/v1/definition | Get agent definition |
| [**getDekByAliasOrId**](VersionV1Api.md#getdekbyaliasorid) | **GET** /api/encryption/v1/dek/alias/{alias} | Get DEK by alias |
| [**getExtendedAgentCard**](VersionV1Api.md#getextendedagentcard) | **GET** /api/a2a/v1/agent/authenticatedExtendedCard | Get extended agent card |
| [**getFunctionInstancesOpenapiSpec**](VersionV1Api.md#getfunctioninstancesopenapispec) | **GET** /api/bridge/v1/function-instances/openapi.json | Get function OpenAPI spec |
| [**getInternalRuntimeConfig**](VersionV1Api.md#getinternalruntimeconfig) | **GET** /_internal/v1/runtime_config | Get runtime config |
| [**getProviderInstance**](VersionV1Api.md#getproviderinstance) | **GET** /api/bridge/v1/provider/{provider_instance_id} | Get provider |
| [**getSecretById**](VersionV1Api.md#getsecretbyid) | **GET** /api/secret/v1/{secret_id} | Get secret |
| [**getSecretByKey**](VersionV1Api.md#getsecretbykey) | **GET** /api/secret/v1/key/{key} | Get secret by key |
| [**getTaskById**](VersionV1Api.md#gettaskbyid) | **GET** /api/task/v1/{task_id} | Get task |
| [**handleJsonrpcRequest**](VersionV1Api.md#handlejsonrpcrequest) | **POST** /api/a2a/v1 | Handle JSON-RPC |
| [**healthCheck**](VersionV1Api.md#healthcheck) | **GET** /_internal/v1/health | Health check |
| [**importDataEncryptionKey**](VersionV1Api.md#importdataencryptionkey) | **POST** /api/encryption/v1/envelope/{envelope_id}/dek/import | Import data key |
| [**importSecret**](VersionV1Api.md#importsecretoperation) | **POST** /api/secret/v1/import | Import secret |
| [**invokeFunction**](VersionV1Api.md#invokefunction) | **POST** /api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/invoke | Invoke function |
| [**listAvailableProviders**](VersionV1Api.md#listavailableproviders) | **GET** /api/bridge/v1/available-providers | List providers |
| [**listContexts**](VersionV1Api.md#listcontexts) | **GET** /api/task/v1/context | List contexts |
| [**listDataEncryptionKeysByEnvelope**](VersionV1Api.md#listdataencryptionkeysbyenvelope) | **GET** /api/encryption/v1/envelope/{envelope_id}/dek | List data keys |
| [**listDecryptedSecrets**](VersionV1Api.md#listdecryptedsecrets) | **GET** /api/secret/v1/list-decrypted | List decrypted secrets |
| [**listEnvelopeEncryptionKeys**](VersionV1Api.md#listenvelopeencryptionkeys) | **GET** /api/encryption/v1/envelope | List envelope keys |
| [**listFunctionInstances**](VersionV1Api.md#listfunctioninstances) | **GET** /api/bridge/v1/function-instances | List function instances |
| [**listProviderInstances**](VersionV1Api.md#listproviderinstances) | **GET** /api/bridge/v1/provider | List provider instances |
| [**listProviderInstancesGroupedByFunction**](VersionV1Api.md#listproviderinstancesgroupedbyfunction) | **GET** /api/bridge/v1/provider/grouped-by-function | List providers by function |
| [**listSecrets**](VersionV1Api.md#listsecrets) | **GET** /api/secret/v1 | List secrets |
| [**listTasks**](VersionV1Api.md#listtasks) | **GET** /api/task/v1 | List tasks |
| [**listTasksByContextId**](VersionV1Api.md#listtasksbycontextid) | **GET** /api/task/v1/context/{context_id}/task | List tasks by context |
| [**listenToMcpSse**](VersionV1Api.md#listentomcpsse) | **GET** /api/bridge/v1/mcp | MCP SSE connection |
| [**migrateAllDataEncryptionKeys**](VersionV1Api.md#migratealldataencryptionkeys) | **POST** /api/encryption/v1/envelope/{envelope_id}/migrate | Migrate all data keys |
| [**migrateDataEncryptionKey**](VersionV1Api.md#migratedataencryptionkey) | **POST** /api/encryption/v1/envelope/{envelope_id}/dek/{dek_id}/migrate | Migrate data key |
| [**resumeUserCredentialBrokering**](VersionV1Api.md#resumeusercredentialbrokering) | **GET** /api/bridge/v1/generic-oauth-callback | OAuth callback |
| [**sendMessage**](VersionV1Api.md#sendmessage) | **POST** /api/task/v1/{task_id}/message | Send message |
| [**startUserCredentialBrokering**](VersionV1Api.md#startusercredentialbrokering) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential/broker | Start credential brokering |
| [**taskHistory**](VersionV1Api.md#taskhistory) | **GET** /api/task/v1/{task_id}/timeline | Get task timeline |
| [**triggerCodegen**](VersionV1Api.md#triggercodegen) | **POST** /_internal/v1/trigger_codegen | Trigger codegen |
| [**triggerMcpMessage**](VersionV1Api.md#triggermcpmessage) | **POST** /api/bridge/v1/mcp | Send MCP message |
| [**updateDekAlias**](VersionV1Api.md#updatedekalias) | **PUT** /api/encryption/v1/dek/alias/{alias} | Update DEK alias |
| [**updateProviderInstance**](VersionV1Api.md#updateproviderinstance) | **PATCH** /api/bridge/v1/provider/{provider_instance_id} | Update provider |
| [**updateSecret**](VersionV1Api.md#updatesecretoperation) | **PUT** /api/secret/v1/{secret_id} | Update secret |
| [**updateTaskStatus**](VersionV1Api.md#updatetaskstatusoperation) | **PUT** /api/task/v1/{task_id} | Update task status |



## createDataEncryptionKey

> DataEncryptionKey createDataEncryptionKey(envelopeId, createDataEncryptionKeyParamsRoute)

Create data key

Create a new data encryption key (DEK) encrypted with the specified envelope encryption key

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { CreateDataEncryptionKeyRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // string | Envelope encryption key ID
    envelopeId: envelopeId_example,
    // CreateDataEncryptionKeyParamsRoute
    createDataEncryptionKeyParamsRoute: ...,
  } satisfies CreateDataEncryptionKeyRequest;

  try {
    const data = await api.createDataEncryptionKey(body);
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
| **envelopeId** | `string` | Envelope encryption key ID | [Defaults to `undefined`] |
| **createDataEncryptionKeyParamsRoute** | [CreateDataEncryptionKeyParamsRoute](CreateDataEncryptionKeyParamsRoute.md) |  | |

### Return type

[**DataEncryptionKey**](DataEncryptionKey.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Create data encryption key |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **404** | Not Found |  -  |
| **500** | Internal Server Error |  -  |
| **502** | Bad Gateway |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## createDekAlias

> DataEncryptionKeyAlias createDekAlias(createDekAliasRequest)

Create DEK alias

Create an alias for a data encryption key to enable lookup by friendly name

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { CreateDekAliasOperationRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // CreateDekAliasRequest
    createDekAliasRequest: ...,
  } satisfies CreateDekAliasOperationRequest;

  try {
    const data = await api.createDekAlias(body);
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
| **createDekAliasRequest** | [CreateDekAliasRequest](CreateDekAliasRequest.md) |  | |

### Return type

[**DataEncryptionKeyAlias**](DataEncryptionKeyAlias.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Create DEK alias |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **404** | Not Found |  -  |
| **500** | Internal Server Error |  -  |
| **502** | Bad Gateway |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## createEnvelopeEncryptionKey

> EnvelopeEncryptionKey createEnvelopeEncryptionKey(envelopeEncryptionKey)

Create envelope key

Create a new envelope encryption key (master key) for encrypting data encryption keys

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { CreateEnvelopeEncryptionKeyRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // EnvelopeEncryptionKey
    envelopeEncryptionKey: ...,
  } satisfies CreateEnvelopeEncryptionKeyRequest;

  try {
    const data = await api.createEnvelopeEncryptionKey(body);
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
| **envelopeEncryptionKey** | [EnvelopeEncryptionKey](EnvelopeEncryptionKey.md) |  | |

### Return type

[**EnvelopeEncryptionKey**](EnvelopeEncryptionKey.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Create envelope encryption key |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal Server Error |  -  |
| **502** | Bad Gateway |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## createProviderInstance

> ProviderInstanceSerialized createProviderInstance(providerControllerTypeId, credentialControllerTypeId, createProviderInstanceParamsInner)

Create provider

Create a new provider instance with the specified configuration

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { CreateProviderInstanceRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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
  VersionV1Api,
} from '@trysoma/api-client';
import type { CreateResourceServerCredentialRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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


## createSecret

> Secret createSecret(createSecretRequest)

Create secret

Create a new encrypted secret with the specified key and value

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { CreateSecretOperationRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // CreateSecretRequest
    createSecretRequest: ...,
  } satisfies CreateSecretOperationRequest;

  try {
    const data = await api.createSecret(body);
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
| **createSecretRequest** | [CreateSecretRequest](CreateSecretRequest.md) |  | |

### Return type

[**Secret**](Secret.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Create a secret |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
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
  VersionV1Api,
} from '@trysoma/api-client';
import type { CreateUserCredentialRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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


## deleteDekAlias

> deleteDekAlias(alias)

Delete DEK alias

Delete an alias for a data encryption key

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { DeleteDekAliasRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // string | DEK alias
    alias: alias_example,
  } satisfies DeleteDekAliasRequest;

  try {
    const data = await api.deleteDekAlias(body);
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
| **alias** | `string` | DEK alias | [Defaults to `undefined`] |

### Return type

`void` (Empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Delete DEK alias |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **404** | Not Found |  -  |
| **500** | Internal Server Error |  -  |
| **502** | Bad Gateway |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## deleteProviderInstance

> any deleteProviderInstance(providerInstanceId)

Delete provider

Delete a provider instance by its unique identifier

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { DeleteProviderInstanceRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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


## deleteSecret

> DeleteSecretResponse deleteSecret(secretId)

Delete secret

Delete a secret by its unique identifier

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { DeleteSecretRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // string | Secret ID
    secretId: 38400000-8cf0-11bd-b23e-10b96e4ef00d,
  } satisfies DeleteSecretRequest;

  try {
    const data = await api.deleteSecret(body);
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
| **secretId** | `string` | Secret ID | [Defaults to `undefined`] |

### Return type

[**DeleteSecretResponse**](DeleteSecretResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Delete secret |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **404** | Not Found |  -  |
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
  VersionV1Api,
} from '@trysoma/api-client';
import type { DisableFunctionRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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
  VersionV1Api,
} from '@trysoma/api-client';
import type { EnableFunctionRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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
  VersionV1Api,
} from '@trysoma/api-client';
import type { EncryptResourceServerConfigurationRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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
  VersionV1Api,
} from '@trysoma/api-client';
import type { EncryptUserCredentialConfigurationRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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


## getAgentCard

> object getAgentCard()

Get agent card

Get the agent card describing agent capabilities and metadata

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { GetAgentCardRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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
  VersionV1Api,
} from '@trysoma/api-client';
import type { GetAgentDefinitionRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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


## getDekByAliasOrId

> DataEncryptionKey getDekByAliasOrId(alias)

Get DEK by alias

Retrieve a data encryption key by its alias or ID

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { GetDekByAliasOrIdRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // string | DEK alias or ID
    alias: alias_example,
  } satisfies GetDekByAliasOrIdRequest;

  try {
    const data = await api.getDekByAliasOrId(body);
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
| **alias** | `string` | DEK alias or ID | [Defaults to `undefined`] |

### Return type

[**DataEncryptionKey**](DataEncryptionKey.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Get DEK by alias or ID |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **404** | Not Found |  -  |
| **500** | Internal Server Error |  -  |
| **502** | Bad Gateway |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getExtendedAgentCard

> object getExtendedAgentCard()

Get extended agent card

Get the authenticated extended agent card with additional metadata

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { GetExtendedAgentCardRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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


## getFunctionInstancesOpenapiSpec

> string getFunctionInstancesOpenapiSpec()

Get function OpenAPI spec

Get the OpenAPI specification for all function instances

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { GetFunctionInstancesOpenapiSpecRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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


## getInternalRuntimeConfig

> object getInternalRuntimeConfig()

Get runtime config

Get the current runtime configuration

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { GetInternalRuntimeConfigRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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


## getProviderInstance

> ProviderInstanceSerializedWithEverything getProviderInstance(providerInstanceId)

Get provider

Retrieve a provider instance by its unique identifier

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { GetProviderInstanceRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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


## getSecretById

> Secret getSecretById(secretId)

Get secret

Retrieve a secret by its unique identifier

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { GetSecretByIdRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // string | Secret ID
    secretId: 38400000-8cf0-11bd-b23e-10b96e4ef00d,
  } satisfies GetSecretByIdRequest;

  try {
    const data = await api.getSecretById(body);
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
| **secretId** | `string` | Secret ID | [Defaults to `undefined`] |

### Return type

[**Secret**](Secret.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Get secret by id |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **404** | Not Found |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getSecretByKey

> Secret getSecretByKey(key)

Get secret by key

Retrieve a secret by its key name

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { GetSecretByKeyRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // string | Secret key
    key: key_example,
  } satisfies GetSecretByKeyRequest;

  try {
    const data = await api.getSecretByKey(body);
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
| **key** | `string` | Secret key | [Defaults to `undefined`] |

### Return type

[**Secret**](Secret.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Get secret by key |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **404** | Not Found |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getTaskById

> TaskWithDetails getTaskById(taskId)

Get task

Retrieve a task by its unique identifier

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { GetTaskByIdRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // string | Task ID
    taskId: 38400000-8cf0-11bd-b23e-10b96e4ef00d,
  } satisfies GetTaskByIdRequest;

  try {
    const data = await api.getTaskById(body);
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
| **taskId** | `string` | Task ID | [Defaults to `undefined`] |

### Return type

[**TaskWithDetails**](TaskWithDetails.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Get task by id |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal Server Error |  -  |
| **502** | Bad Gateway |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## handleJsonrpcRequest

> handleJsonrpcRequest(body)

Handle JSON-RPC

Handle JSON-RPC requests for agent-to-agent communication (tasks, messages, etc.)

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { HandleJsonrpcRequestRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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


## healthCheck

> healthCheck()

Health check

Check the health status of the service and SDK server connectivity

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { HealthCheckRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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


## importDataEncryptionKey

> DataEncryptionKey importDataEncryptionKey(envelopeId, importDataEncryptionKeyParamsRoute)

Import data key

Import an existing pre-encrypted data encryption key into the system

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { ImportDataEncryptionKeyRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // string | Envelope encryption key ID
    envelopeId: envelopeId_example,
    // ImportDataEncryptionKeyParamsRoute
    importDataEncryptionKeyParamsRoute: ...,
  } satisfies ImportDataEncryptionKeyRequest;

  try {
    const data = await api.importDataEncryptionKey(body);
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
| **envelopeId** | `string` | Envelope encryption key ID | [Defaults to `undefined`] |
| **importDataEncryptionKeyParamsRoute** | [ImportDataEncryptionKeyParamsRoute](ImportDataEncryptionKeyParamsRoute.md) |  | |

### Return type

[**DataEncryptionKey**](DataEncryptionKey.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Import data encryption key |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **404** | Not Found |  -  |
| **500** | Internal Server Error |  -  |
| **502** | Bad Gateway |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## importSecret

> Secret importSecret(importSecretRequest)

Import secret

Import an existing pre-encrypted secret into the system

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { ImportSecretOperationRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // ImportSecretRequest
    importSecretRequest: ...,
  } satisfies ImportSecretOperationRequest;

  try {
    const data = await api.importSecret(body);
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
| **importSecretRequest** | [ImportSecretRequest](ImportSecretRequest.md) |  | |

### Return type

[**Secret**](Secret.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Import a pre-encrypted secret |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
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
  VersionV1Api,
} from '@trysoma/api-client';
import type { InvokeFunctionRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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
  VersionV1Api,
} from '@trysoma/api-client';
import type { ListAvailableProvidersRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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


## listContexts

> ContextInfoPaginatedResponse listContexts(pageSize, nextPageToken)

List contexts

List all unique task contexts with pagination

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { ListContextsRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // number
    pageSize: 789,
    // string (optional)
    nextPageToken: nextPageToken_example,
  } satisfies ListContextsRequest;

  try {
    const data = await api.listContexts(body);
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

[**ContextInfoPaginatedResponse**](ContextInfoPaginatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | List contexts |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal Server Error |  -  |
| **502** | Bad Gateway |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listDataEncryptionKeysByEnvelope

> DataEncryptionKeyListItemPaginatedResponse listDataEncryptionKeysByEnvelope(envelopeId, pageSize, nextPageToken)

List data keys

List all data encryption keys encrypted with the specified envelope encryption key

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { ListDataEncryptionKeysByEnvelopeRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // string | Envelope encryption key ID
    envelopeId: envelopeId_example,
    // number
    pageSize: 789,
    // string (optional)
    nextPageToken: nextPageToken_example,
  } satisfies ListDataEncryptionKeysByEnvelopeRequest;

  try {
    const data = await api.listDataEncryptionKeysByEnvelope(body);
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
| **envelopeId** | `string` | Envelope encryption key ID | [Defaults to `undefined`] |
| **pageSize** | `number` |  | [Defaults to `undefined`] |
| **nextPageToken** | `string` |  | [Optional] [Defaults to `undefined`] |

### Return type

[**DataEncryptionKeyListItemPaginatedResponse**](DataEncryptionKeyListItemPaginatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | List data encryption keys |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **404** | Not Found |  -  |
| **500** | Internal Server Error |  -  |
| **502** | Bad Gateway |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listDecryptedSecrets

> ListDecryptedSecretsResponse listDecryptedSecrets(pageSize, nextPageToken)

List decrypted secrets

List all secrets with decrypted values (requires decryption access)

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { ListDecryptedSecretsRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // number
    pageSize: 789,
    // string (optional)
    nextPageToken: nextPageToken_example,
  } satisfies ListDecryptedSecretsRequest;

  try {
    const data = await api.listDecryptedSecrets(body);
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

[**ListDecryptedSecretsResponse**](ListDecryptedSecretsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | List secrets with decrypted values |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listEnvelopeEncryptionKeys

> EnvelopeEncryptionKeyPaginatedResponse listEnvelopeEncryptionKeys(pageSize, nextPageToken)

List envelope keys

List all envelope encryption keys (master keys) with pagination

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { ListEnvelopeEncryptionKeysRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // number
    pageSize: 789,
    // string (optional)
    nextPageToken: nextPageToken_example,
  } satisfies ListEnvelopeEncryptionKeysRequest;

  try {
    const data = await api.listEnvelopeEncryptionKeys(body);
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

[**EnvelopeEncryptionKeyPaginatedResponse**](EnvelopeEncryptionKeyPaginatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | List envelope encryption keys |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal Server Error |  -  |
| **502** | Bad Gateway |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listFunctionInstances

> FunctionInstanceSerializedPaginatedResponse listFunctionInstances(pageSize, nextPageToken, providerInstanceId)

List function instances

List all function instances with optional filtering by provider instance

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { ListFunctionInstancesRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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
  VersionV1Api,
} from '@trysoma/api-client';
import type { ListProviderInstancesRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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
  VersionV1Api,
} from '@trysoma/api-client';
import type { ListProviderInstancesGroupedByFunctionRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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


## listSecrets

> ListSecretsResponse listSecrets(pageSize, nextPageToken)

List secrets

List all secrets with pagination (values are encrypted)

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { ListSecretsRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // number
    pageSize: 789,
    // string (optional)
    nextPageToken: nextPageToken_example,
  } satisfies ListSecretsRequest;

  try {
    const data = await api.listSecrets(body);
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

[**ListSecretsResponse**](ListSecretsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | List secrets |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listTasks

> TaskPaginatedResponse listTasks(pageSize, nextPageToken)

List tasks

List all tasks with pagination

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { ListTasksRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // number
    pageSize: 789,
    // string (optional)
    nextPageToken: nextPageToken_example,
  } satisfies ListTasksRequest;

  try {
    const data = await api.listTasks(body);
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

[**TaskPaginatedResponse**](TaskPaginatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | List tasks |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal Server Error |  -  |
| **502** | Bad Gateway |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listTasksByContextId

> TaskPaginatedResponse listTasksByContextId(pageSize, contextId, nextPageToken)

List tasks by context

List all tasks for a specific context ID with pagination

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { ListTasksByContextIdRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // number
    pageSize: 789,
    // string | Context ID
    contextId: 38400000-8cf0-11bd-b23e-10b96e4ef00d,
    // string (optional)
    nextPageToken: nextPageToken_example,
  } satisfies ListTasksByContextIdRequest;

  try {
    const data = await api.listTasksByContextId(body);
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
| **contextId** | `string` | Context ID | [Defaults to `undefined`] |
| **nextPageToken** | `string` |  | [Optional] [Defaults to `undefined`] |

### Return type

[**TaskPaginatedResponse**](TaskPaginatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | List tasks |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal Server Error |  -  |
| **502** | Bad Gateway |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listenToMcpSse

> listenToMcpSse()

MCP SSE connection

Establish Server-Sent Events (SSE) connection for MCP protocol communication

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { ListenToMcpSseRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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


## migrateAllDataEncryptionKeys

> migrateAllDataEncryptionKeys(envelopeId, migrateAllDataEncryptionKeysParamsRoute)

Migrate all data keys

Migrate all data encryption keys encrypted with the specified envelope key to a new envelope key

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { MigrateAllDataEncryptionKeysRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // string | Envelope encryption key ID
    envelopeId: envelopeId_example,
    // MigrateAllDataEncryptionKeysParamsRoute
    migrateAllDataEncryptionKeysParamsRoute: ...,
  } satisfies MigrateAllDataEncryptionKeysRequest;

  try {
    const data = await api.migrateAllDataEncryptionKeys(body);
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
| **envelopeId** | `string` | Envelope encryption key ID | [Defaults to `undefined`] |
| **migrateAllDataEncryptionKeysParamsRoute** | [MigrateAllDataEncryptionKeysParamsRoute](MigrateAllDataEncryptionKeysParamsRoute.md) |  | |

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
| **200** | Migrate all data encryption keys |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **404** | Not Found |  -  |
| **500** | Internal Server Error |  -  |
| **502** | Bad Gateway |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## migrateDataEncryptionKey

> migrateDataEncryptionKey(envelopeId, dekId, migrateDataEncryptionKeyParamsRoute)

Migrate data key

Migrate a data encryption key to be encrypted with a different envelope encryption key

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { MigrateDataEncryptionKeyRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // string | Envelope encryption key ID
    envelopeId: envelopeId_example,
    // string | Data encryption key ID
    dekId: dekId_example,
    // MigrateDataEncryptionKeyParamsRoute
    migrateDataEncryptionKeyParamsRoute: ...,
  } satisfies MigrateDataEncryptionKeyRequest;

  try {
    const data = await api.migrateDataEncryptionKey(body);
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
| **envelopeId** | `string` | Envelope encryption key ID | [Defaults to `undefined`] |
| **dekId** | `string` | Data encryption key ID | [Defaults to `undefined`] |
| **migrateDataEncryptionKeyParamsRoute** | [MigrateDataEncryptionKeyParamsRoute](MigrateDataEncryptionKeyParamsRoute.md) |  | |

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
| **200** | Migrate data encryption key |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **404** | Not Found |  -  |
| **500** | Internal Server Error |  -  |
| **502** | Bad Gateway |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## resumeUserCredentialBrokering

> UserCredentialBrokeringResponse resumeUserCredentialBrokering(state, code, error, errorDescription)

OAuth callback

Handle OAuth callback to complete user credential brokering flow

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { ResumeUserCredentialBrokeringRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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


## sendMessage

> CreateMessageResponse sendMessage(taskId, createMessageRequest)

Send message

Send a message to a task

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { SendMessageRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // string | Task ID
    taskId: 38400000-8cf0-11bd-b23e-10b96e4ef00d,
    // CreateMessageRequest
    createMessageRequest: ...,
  } satisfies SendMessageRequest;

  try {
    const data = await api.sendMessage(body);
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
| **taskId** | `string` | Task ID | [Defaults to `undefined`] |
| **createMessageRequest** | [CreateMessageRequest](CreateMessageRequest.md) |  | |

### Return type

[**CreateMessageResponse**](CreateMessageResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Create message |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal Server Error |  -  |
| **502** | Bad Gateway |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## startUserCredentialBrokering

> UserCredentialBrokeringResponse startUserCredentialBrokering(providerControllerTypeId, credentialControllerTypeId, startUserCredentialBrokeringParamsInner)

Start credential brokering

Start the OAuth flow for user credential brokering

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { StartUserCredentialBrokeringRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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


## taskHistory

> TaskTimelineItemPaginatedResponse taskHistory(pageSize, taskId, nextPageToken)

Get task timeline

Get the timeline history of a task with pagination

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { TaskHistoryRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // number
    pageSize: 789,
    // string | Task ID
    taskId: 38400000-8cf0-11bd-b23e-10b96e4ef00d,
    // string (optional)
    nextPageToken: nextPageToken_example,
  } satisfies TaskHistoryRequest;

  try {
    const data = await api.taskHistory(body);
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
| **taskId** | `string` | Task ID | [Defaults to `undefined`] |
| **nextPageToken** | `string` |  | [Optional] [Defaults to `undefined`] |

### Return type

[**TaskTimelineItemPaginatedResponse**](TaskTimelineItemPaginatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Get task timeline items |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal Server Error |  -  |
| **502** | Bad Gateway |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## triggerCodegen

> TriggerCodegenResponse triggerCodegen()

Trigger codegen

Trigger code generation for the SDK

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { TriggerCodegenRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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


## triggerMcpMessage

> triggerMcpMessage(body)

Send MCP message

Send a JSON-RPC message to the MCP server

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { TriggerMcpMessageRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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


## updateDekAlias

> DataEncryptionKeyAlias updateDekAlias(alias, updateAliasParams)

Update DEK alias

Update the alias for a data encryption key

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { UpdateDekAliasRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // string | DEK alias
    alias: alias_example,
    // UpdateAliasParams
    updateAliasParams: ...,
  } satisfies UpdateDekAliasRequest;

  try {
    const data = await api.updateDekAlias(body);
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
| **alias** | `string` | DEK alias | [Defaults to `undefined`] |
| **updateAliasParams** | [UpdateAliasParams](UpdateAliasParams.md) |  | |

### Return type

[**DataEncryptionKeyAlias**](DataEncryptionKeyAlias.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Update DEK alias |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **404** | Not Found |  -  |
| **500** | Internal Server Error |  -  |
| **502** | Bad Gateway |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## updateProviderInstance

> any updateProviderInstance(providerInstanceId, updateProviderInstanceParamsInner)

Update provider

Update an existing provider instance configuration

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { UpdateProviderInstanceRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

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


## updateSecret

> Secret updateSecret(secretId, updateSecretRequest)

Update secret

Update an existing secret\&#39;s value or metadata

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { UpdateSecretOperationRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // string | Secret ID
    secretId: 38400000-8cf0-11bd-b23e-10b96e4ef00d,
    // UpdateSecretRequest
    updateSecretRequest: ...,
  } satisfies UpdateSecretOperationRequest;

  try {
    const data = await api.updateSecret(body);
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
| **secretId** | `string` | Secret ID | [Defaults to `undefined`] |
| **updateSecretRequest** | [UpdateSecretRequest](UpdateSecretRequest.md) |  | |

### Return type

[**Secret**](Secret.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Update secret |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **404** | Not Found |  -  |
| **500** | Internal Server Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## updateTaskStatus

> any updateTaskStatus(taskId, updateTaskStatusRequest)

Update task status

Update the status of a task

### Example

```ts
import {
  Configuration,
  VersionV1Api,
} from '@trysoma/api-client';
import type { UpdateTaskStatusOperationRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new VersionV1Api();

  const body = {
    // string | Task ID
    taskId: 38400000-8cf0-11bd-b23e-10b96e4ef00d,
    // UpdateTaskStatusRequest
    updateTaskStatusRequest: ...,
  } satisfies UpdateTaskStatusOperationRequest;

  try {
    const data = await api.updateTaskStatus(body);
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
| **taskId** | `string` | Task ID | [Defaults to `undefined`] |
| **updateTaskStatusRequest** | [UpdateTaskStatusRequest](UpdateTaskStatusRequest.md) |  | |

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
| **200** | Update task status |  -  |
| **400** | Bad Request |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **500** | Internal Server Error |  -  |
| **502** | Bad Gateway |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

