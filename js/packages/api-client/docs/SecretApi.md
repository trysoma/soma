# SecretApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createSecret**](SecretApi.md#createsecretoperation) | **POST** /api/secret/v1 | Create secret |
| [**deleteSecret**](SecretApi.md#deletesecret) | **DELETE** /api/secret/v1/{secret_id} | Delete secret |
| [**getSecretById**](SecretApi.md#getsecretbyid) | **GET** /api/secret/v1/{secret_id} | Get secret |
| [**getSecretByKey**](SecretApi.md#getsecretbykey) | **GET** /api/secret/v1/key/{key} | Get secret by key |
| [**importSecret**](SecretApi.md#importsecretoperation) | **POST** /api/secret/v1/import | Import secret |
| [**listDecryptedSecrets**](SecretApi.md#listdecryptedsecrets) | **GET** /api/secret/v1/list-decrypted | List decrypted secrets |
| [**listSecrets**](SecretApi.md#listsecrets) | **GET** /api/secret/v1 | List secrets |
| [**updateSecret**](SecretApi.md#updatesecretoperation) | **PUT** /api/secret/v1/{secret_id} | Update secret |



## createSecret

> Secret createSecret(createSecretRequest)

Create secret

Create a new encrypted secret with the specified key and value

### Example

```ts
import {
  Configuration,
  SecretApi,
} from '@trysoma/api-client';
import type { CreateSecretOperationRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new SecretApi();

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


## deleteSecret

> DeleteSecretResponse deleteSecret(secretId)

Delete secret

Delete a secret by its unique identifier

### Example

```ts
import {
  Configuration,
  SecretApi,
} from '@trysoma/api-client';
import type { DeleteSecretRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new SecretApi();

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


## getSecretById

> Secret getSecretById(secretId)

Get secret

Retrieve a secret by its unique identifier

### Example

```ts
import {
  Configuration,
  SecretApi,
} from '@trysoma/api-client';
import type { GetSecretByIdRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new SecretApi();

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
  SecretApi,
} from '@trysoma/api-client';
import type { GetSecretByKeyRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new SecretApi();

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


## importSecret

> Secret importSecret(importSecretRequest)

Import secret

Import an existing pre-encrypted secret into the system

### Example

```ts
import {
  Configuration,
  SecretApi,
} from '@trysoma/api-client';
import type { ImportSecretOperationRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new SecretApi();

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


## listDecryptedSecrets

> ListDecryptedSecretsResponse listDecryptedSecrets(pageSize, nextPageToken)

List decrypted secrets

List all secrets with decrypted values (requires decryption access)

### Example

```ts
import {
  Configuration,
  SecretApi,
} from '@trysoma/api-client';
import type { ListDecryptedSecretsRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new SecretApi();

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


## listSecrets

> ListSecretsResponse listSecrets(pageSize, nextPageToken)

List secrets

List all secrets with pagination (values are encrypted)

### Example

```ts
import {
  Configuration,
  SecretApi,
} from '@trysoma/api-client';
import type { ListSecretsRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new SecretApi();

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


## updateSecret

> Secret updateSecret(secretId, updateSecretRequest)

Update secret

Update an existing secret\&#39;s value or metadata

### Example

```ts
import {
  Configuration,
  SecretApi,
} from '@trysoma/api-client';
import type { UpdateSecretOperationRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new SecretApi();

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

