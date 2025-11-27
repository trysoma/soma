# EncryptionApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createDataEncryptionKey**](EncryptionApi.md#createdataencryptionkey) | **POST** /api/encryption/v1/envelope/{envelope_id}/dek | Create data key |
| [**createDekAlias**](EncryptionApi.md#createdekaliasoperation) | **POST** /api/encryption/v1/dek/alias | Create DEK alias |
| [**createEnvelopeEncryptionKey**](EncryptionApi.md#createenvelopeencryptionkey) | **POST** /api/encryption/v1/envelope | Create envelope key |
| [**deleteDekAlias**](EncryptionApi.md#deletedekalias) | **DELETE** /api/encryption/v1/dek/alias/{alias} | Delete DEK alias |
| [**getDekByAliasOrId**](EncryptionApi.md#getdekbyaliasorid) | **GET** /api/encryption/v1/dek/alias/{alias} | Get DEK by alias |
| [**importDataEncryptionKey**](EncryptionApi.md#importdataencryptionkey) | **POST** /api/encryption/v1/envelope/{envelope_id}/dek/import | Import data key |
| [**listDataEncryptionKeysByEnvelope**](EncryptionApi.md#listdataencryptionkeysbyenvelope) | **GET** /api/encryption/v1/envelope/{envelope_id}/dek | List data keys |
| [**listEnvelopeEncryptionKeys**](EncryptionApi.md#listenvelopeencryptionkeys) | **GET** /api/encryption/v1/envelope | List envelope keys |
| [**migrateAllDataEncryptionKeys**](EncryptionApi.md#migratealldataencryptionkeys) | **POST** /api/encryption/v1/envelope/{envelope_id}/migrate | Migrate all data keys |
| [**migrateDataEncryptionKey**](EncryptionApi.md#migratedataencryptionkey) | **POST** /api/encryption/v1/envelope/{envelope_id}/dek/{dek_id}/migrate | Migrate data key |
| [**updateDekAlias**](EncryptionApi.md#updatedekalias) | **PUT** /api/encryption/v1/dek/alias/{alias} | Update DEK alias |



## createDataEncryptionKey

> DataEncryptionKey createDataEncryptionKey(envelopeId, createDataEncryptionKeyParamsRoute)

Create data key

Create a new data encryption key (DEK) encrypted with the specified envelope encryption key

### Example

```ts
import {
  Configuration,
  EncryptionApi,
} from '@trysoma/api-client';
import type { CreateDataEncryptionKeyRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new EncryptionApi();

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
  EncryptionApi,
} from '@trysoma/api-client';
import type { CreateDekAliasOperationRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new EncryptionApi();

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
  EncryptionApi,
} from '@trysoma/api-client';
import type { CreateEnvelopeEncryptionKeyRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new EncryptionApi();

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


## deleteDekAlias

> deleteDekAlias(alias)

Delete DEK alias

Delete an alias for a data encryption key

### Example

```ts
import {
  Configuration,
  EncryptionApi,
} from '@trysoma/api-client';
import type { DeleteDekAliasRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new EncryptionApi();

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


## getDekByAliasOrId

> DataEncryptionKey getDekByAliasOrId(alias)

Get DEK by alias

Retrieve a data encryption key by its alias or ID

### Example

```ts
import {
  Configuration,
  EncryptionApi,
} from '@trysoma/api-client';
import type { GetDekByAliasOrIdRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new EncryptionApi();

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


## importDataEncryptionKey

> DataEncryptionKey importDataEncryptionKey(envelopeId, importDataEncryptionKeyParamsRoute)

Import data key

Import an existing pre-encrypted data encryption key into the system

### Example

```ts
import {
  Configuration,
  EncryptionApi,
} from '@trysoma/api-client';
import type { ImportDataEncryptionKeyRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new EncryptionApi();

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


## listDataEncryptionKeysByEnvelope

> DataEncryptionKeyListItemPaginatedResponse listDataEncryptionKeysByEnvelope(envelopeId, pageSize, nextPageToken)

List data keys

List all data encryption keys encrypted with the specified envelope encryption key

### Example

```ts
import {
  Configuration,
  EncryptionApi,
} from '@trysoma/api-client';
import type { ListDataEncryptionKeysByEnvelopeRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new EncryptionApi();

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


## listEnvelopeEncryptionKeys

> EnvelopeEncryptionKeyPaginatedResponse listEnvelopeEncryptionKeys(pageSize, nextPageToken)

List envelope keys

List all envelope encryption keys (master keys) with pagination

### Example

```ts
import {
  Configuration,
  EncryptionApi,
} from '@trysoma/api-client';
import type { ListEnvelopeEncryptionKeysRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new EncryptionApi();

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


## migrateAllDataEncryptionKeys

> migrateAllDataEncryptionKeys(envelopeId, migrateAllDataEncryptionKeysParamsRoute)

Migrate all data keys

Migrate all data encryption keys encrypted with the specified envelope key to a new envelope key

### Example

```ts
import {
  Configuration,
  EncryptionApi,
} from '@trysoma/api-client';
import type { MigrateAllDataEncryptionKeysRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new EncryptionApi();

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
  EncryptionApi,
} from '@trysoma/api-client';
import type { MigrateDataEncryptionKeyRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new EncryptionApi();

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


## updateDekAlias

> DataEncryptionKeyAlias updateDekAlias(alias, updateAliasParams)

Update DEK alias

Update the alias for a data encryption key

### Example

```ts
import {
  Configuration,
  EncryptionApi,
} from '@trysoma/api-client';
import type { UpdateDekAliasRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new EncryptionApi();

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

