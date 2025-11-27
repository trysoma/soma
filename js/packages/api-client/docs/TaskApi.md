# TaskApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**getTaskById**](TaskApi.md#gettaskbyid) | **GET** /api/task/v1/{task_id} | Get task |
| [**listContexts**](TaskApi.md#listcontexts) | **GET** /api/task/v1/context | List contexts |
| [**listTasks**](TaskApi.md#listtasks) | **GET** /api/task/v1 | List tasks |
| [**listTasksByContextId**](TaskApi.md#listtasksbycontextid) | **GET** /api/task/v1/context/{context_id}/task | List tasks by context |
| [**sendMessage**](TaskApi.md#sendmessage) | **POST** /api/task/v1/{task_id}/message | Send message |
| [**taskHistory**](TaskApi.md#taskhistory) | **GET** /api/task/v1/{task_id}/timeline | Get task timeline |
| [**updateTaskStatus**](TaskApi.md#updatetaskstatusoperation) | **PUT** /api/task/v1/{task_id} | Update task status |



## getTaskById

> TaskWithDetails getTaskById(taskId)

Get task

Retrieve a task by its unique identifier

### Example

```ts
import {
  Configuration,
  TaskApi,
} from '@trysoma/api-client';
import type { GetTaskByIdRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new TaskApi();

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


## listContexts

> ContextInfoPaginatedResponse listContexts(pageSize, nextPageToken)

List contexts

List all unique task contexts with pagination

### Example

```ts
import {
  Configuration,
  TaskApi,
} from '@trysoma/api-client';
import type { ListContextsRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new TaskApi();

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


## listTasks

> TaskPaginatedResponse listTasks(pageSize, nextPageToken)

List tasks

List all tasks with pagination

### Example

```ts
import {
  Configuration,
  TaskApi,
} from '@trysoma/api-client';
import type { ListTasksRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new TaskApi();

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
  TaskApi,
} from '@trysoma/api-client';
import type { ListTasksByContextIdRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new TaskApi();

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


## sendMessage

> CreateMessageResponse sendMessage(taskId, createMessageRequest)

Send message

Send a message to a task

### Example

```ts
import {
  Configuration,
  TaskApi,
} from '@trysoma/api-client';
import type { SendMessageRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new TaskApi();

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


## taskHistory

> TaskTimelineItemPaginatedResponse taskHistory(pageSize, taskId, nextPageToken)

Get task timeline

Get the timeline history of a task with pagination

### Example

```ts
import {
  Configuration,
  TaskApi,
} from '@trysoma/api-client';
import type { TaskHistoryRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new TaskApi();

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


## updateTaskStatus

> any updateTaskStatus(taskId, updateTaskStatusRequest)

Update task status

Update the status of a task

### Example

```ts
import {
  Configuration,
  TaskApi,
} from '@trysoma/api-client';
import type { UpdateTaskStatusOperationRequest } from '@trysoma/api-client';

async function example() {
  console.log("🚀 Testing @trysoma/api-client SDK...");
  const api = new TaskApi();

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

