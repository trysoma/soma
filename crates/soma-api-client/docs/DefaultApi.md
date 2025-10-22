# \DefaultApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**agent_card**](DefaultApi.md#agent_card) | **GET** /api/a2a/v1/.well-known/agent.json | 
[**create_data_encryption_key**](DefaultApi.md#create_data_encryption_key) | **POST** /api/bridge/v1/encryption/data-encryption-key | 
[**create_provider_instance**](DefaultApi.md#create_provider_instance) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id} | 
[**create_resource_server_credential**](DefaultApi.md#create_resource_server_credential) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/resource-server | 
[**create_user_credential**](DefaultApi.md#create_user_credential) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential | 
[**delete_provider_instance**](DefaultApi.md#delete_provider_instance) | **DELETE** /api/bridge/v1/provider/{provider_instance_id} | 
[**disable_function**](DefaultApi.md#disable_function) | **POST** /api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/disable | 
[**enable_function**](DefaultApi.md#enable_function) | **POST** /api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/enable | 
[**encrypt_resource_server_configuration**](DefaultApi.md#encrypt_resource_server_configuration) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/resource-server/encrypt | 
[**encrypt_user_credential_configuration**](DefaultApi.md#encrypt_user_credential_configuration) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential/encrypt | 
[**extended_agent_card**](DefaultApi.md#extended_agent_card) | **GET** /api/a2a/v1/agent/authenticatedExtendedCard | 
[**get_agent_definition**](DefaultApi.md#get_agent_definition) | **GET** /api/a2a/v1/definition | 
[**get_frontend_env**](DefaultApi.md#get_frontend_env) | **GET** /api/frontend/v1/runtime_config | 
[**get_provider_instance**](DefaultApi.md#get_provider_instance) | **GET** /api/bridge/v1/provider/{provider_instance_id} | 
[**get_task_by_id**](DefaultApi.md#get_task_by_id) | **GET** /api/task/v1/{task_id} | 
[**invoke_function**](DefaultApi.md#invoke_function) | **POST** /api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/invoke | 
[**json_rpc**](DefaultApi.md#json_rpc) | **POST** /api/a2a/v1 | 
[**list_available_providers**](DefaultApi.md#list_available_providers) | **GET** /api/bridge/v1/available-providers | 
[**list_contexts**](DefaultApi.md#list_contexts) | **GET** /api/task/v1/context | 
[**list_data_encryption_keys**](DefaultApi.md#list_data_encryption_keys) | **GET** /api/bridge/v1/encryption/data-encryption-key | 
[**list_function_instances**](DefaultApi.md#list_function_instances) | **GET** /api/bridge/v1/function-instances | 
[**list_provider_instances**](DefaultApi.md#list_provider_instances) | **GET** /api/bridge/v1/provider | 
[**list_provider_instances_grouped_by_function**](DefaultApi.md#list_provider_instances_grouped_by_function) | **GET** /api/bridge/v1/provider/grouped-by-function | 
[**list_tasks**](DefaultApi.md#list_tasks) | **GET** /api/task/v1 | 
[**list_tasks_by_context_id**](DefaultApi.md#list_tasks_by_context_id) | **GET** /api/task/v1/context/{context_id}/task | 
[**resume_user_credential_brokering**](DefaultApi.md#resume_user_credential_brokering) | **GET** /api/bridge/v1/generic-oauth-callback | 
[**send_message**](DefaultApi.md#send_message) | **POST** /api/task/v1/{task_id}/message | 
[**start_user_credential_brokering**](DefaultApi.md#start_user_credential_brokering) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential/broker | 
[**task_history**](DefaultApi.md#task_history) | **GET** /api/task/v1/{task_id}/timeline | 
[**update_provider_instance**](DefaultApi.md#update_provider_instance) | **PATCH** /api/bridge/v1/provider/{provider_instance_id} | 
[**update_task_status**](DefaultApi.md#update_task_status) | **PUT** /api/task/v1/{task_id} | 



## agent_card

> serde_json::Value agent_card()


### Parameters

This endpoint does not need any parameter.

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_data_encryption_key

> models::DataEncryptionKey create_data_encryption_key(create_data_encryption_key_params)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_data_encryption_key_params** | [**CreateDataEncryptionKeyParams**](CreateDataEncryptionKeyParams.md) |  | [required] |

### Return type

[**models::DataEncryptionKey**](DataEncryptionKey.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_provider_instance

> models::ProviderInstanceSerialized create_provider_instance(provider_controller_type_id, credential_controller_type_id, create_provider_instance_params_inner)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**provider_controller_type_id** | **String** |  | [required] |
**credential_controller_type_id** | **String** |  | [required] |
**create_provider_instance_params_inner** | [**CreateProviderInstanceParamsInner**](CreateProviderInstanceParamsInner.md) |  | [required] |

### Return type

[**models::ProviderInstanceSerialized**](ProviderInstanceSerialized.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_resource_server_credential

> models::ResourceServerCredentialSerialized create_resource_server_credential(provider_controller_type_id, credential_controller_type_id, create_resource_server_credential_params_inner)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**provider_controller_type_id** | **String** | Provider controller type ID | [required] |
**credential_controller_type_id** | **String** | Credential controller type ID | [required] |
**create_resource_server_credential_params_inner** | [**CreateResourceServerCredentialParamsInner**](CreateResourceServerCredentialParamsInner.md) |  | [required] |

### Return type

[**models::ResourceServerCredentialSerialized**](ResourceServerCredentialSerialized.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_user_credential

> models::UserCredentialSerialized create_user_credential(provider_controller_type_id, credential_controller_type_id, create_user_credential_params_inner)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**provider_controller_type_id** | **String** | Provider controller type ID | [required] |
**credential_controller_type_id** | **String** | Credential controller type ID | [required] |
**create_user_credential_params_inner** | [**CreateUserCredentialParamsInner**](CreateUserCredentialParamsInner.md) |  | [required] |

### Return type

[**models::UserCredentialSerialized**](UserCredentialSerialized.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_provider_instance

> serde_json::Value delete_provider_instance(provider_instance_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**provider_instance_id** | **String** | Provider instance ID | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## disable_function

> serde_json::Value disable_function(provider_instance_id, function_controller_type_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**provider_instance_id** | **String** | Provider instance ID | [required] |
**function_controller_type_id** | **String** | Function controller type ID | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## enable_function

> models::FunctionInstanceSerialized enable_function(provider_instance_id, function_controller_type_id, body)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**provider_instance_id** | **String** | Provider instance ID | [required] |
**function_controller_type_id** | **String** | Function controller type ID | [required] |
**body** | **serde_json::Value** |  | [required] |

### Return type

[**models::FunctionInstanceSerialized**](FunctionInstanceSerialized.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## encrypt_resource_server_configuration

> serde_json::Value encrypt_resource_server_configuration(provider_controller_type_id, credential_controller_type_id, encrypt_credential_configuration_params_inner)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**provider_controller_type_id** | **String** | Provider controller type ID | [required] |
**credential_controller_type_id** | **String** | Credential controller type ID | [required] |
**encrypt_credential_configuration_params_inner** | [**EncryptCredentialConfigurationParamsInner**](EncryptCredentialConfigurationParamsInner.md) |  | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## encrypt_user_credential_configuration

> serde_json::Value encrypt_user_credential_configuration(provider_controller_type_id, credential_controller_type_id, encrypt_credential_configuration_params_inner)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**provider_controller_type_id** | **String** | Provider controller type ID | [required] |
**credential_controller_type_id** | **String** | Credential controller type ID | [required] |
**encrypt_credential_configuration_params_inner** | [**EncryptCredentialConfigurationParamsInner**](EncryptCredentialConfigurationParamsInner.md) |  | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## extended_agent_card

> serde_json::Value extended_agent_card()


### Parameters

This endpoint does not need any parameter.

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_agent_definition

> models::SomaAgentDefinition get_agent_definition()


### Parameters

This endpoint does not need any parameter.

### Return type

[**models::SomaAgentDefinition**](SomaAgentDefinition.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_frontend_env

> serde_json::Value get_frontend_env()


### Parameters

This endpoint does not need any parameter.

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_provider_instance

> models::ProviderInstanceSerializedWithEverything get_provider_instance(provider_instance_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**provider_instance_id** | **String** | Provider instance ID | [required] |

### Return type

[**models::ProviderInstanceSerializedWithEverything**](ProviderInstanceSerializedWithEverything.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_task_by_id

> models::TaskWithDetails get_task_by_id(task_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**task_id** | **uuid::Uuid** | Task ID | [required] |

### Return type

[**models::TaskWithDetails**](TaskWithDetails.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## invoke_function

> serde_json::Value invoke_function(provider_instance_id, function_controller_type_id, invoke_function_params_inner)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**provider_instance_id** | **String** | Provider instance ID | [required] |
**function_controller_type_id** | **String** | Function controller type ID | [required] |
**invoke_function_params_inner** | [**InvokeFunctionParamsInner**](InvokeFunctionParamsInner.md) |  | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## json_rpc

> json_rpc(body)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**body** | **serde_json::Value** |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_available_providers

> models::ProviderControllerSerializedPaginatedResponse list_available_providers(page_size, next_page_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_size** | **i64** |  | [required] |
**next_page_token** | Option<**String**> |  |  |

### Return type

[**models::ProviderControllerSerializedPaginatedResponse**](ProviderControllerSerializedPaginatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_contexts

> models::ContextInfoPaginatedResponse list_contexts(page_size, next_page_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_size** | **i64** |  | [required] |
**next_page_token** | Option<**String**> |  |  |

### Return type

[**models::ContextInfoPaginatedResponse**](ContextInfoPaginatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_data_encryption_keys

> models::DataEncryptionKeyListItemPaginatedResponse list_data_encryption_keys(page_size, next_page_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_size** | **i64** |  | [required] |
**next_page_token** | Option<**String**> |  |  |

### Return type

[**models::DataEncryptionKeyListItemPaginatedResponse**](DataEncryptionKeyListItemPaginatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_function_instances

> models::FunctionInstanceSerializedPaginatedResponse list_function_instances(page_size, next_page_token, provider_instance_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_size** | **i64** |  | [required] |
**next_page_token** | Option<**String**> |  |  |
**provider_instance_id** | Option<**String**> |  |  |

### Return type

[**models::FunctionInstanceSerializedPaginatedResponse**](FunctionInstanceSerializedPaginatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_provider_instances

> models::ProviderInstanceListItemPaginatedResponse list_provider_instances(page_size, next_page_token, status, provider_controller_type_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_size** | **i64** |  | [required] |
**next_page_token** | Option<**String**> |  |  |
**status** | Option<**String**> |  |  |
**provider_controller_type_id** | Option<**String**> |  |  |

### Return type

[**models::ProviderInstanceListItemPaginatedResponse**](ProviderInstanceListItemPaginatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_provider_instances_grouped_by_function

> models::FunctionInstanceConfigPaginatedResponse list_provider_instances_grouped_by_function(page_size, next_page_token, provider_controller_type_id, function_category)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_size** | **i64** |  | [required] |
**next_page_token** | Option<**String**> |  |  |
**provider_controller_type_id** | Option<**String**> |  |  |
**function_category** | Option<**String**> |  |  |

### Return type

[**models::FunctionInstanceConfigPaginatedResponse**](FunctionInstanceConfigPaginatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_tasks

> models::TaskPaginatedResponse list_tasks(page_size, next_page_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_size** | **i64** |  | [required] |
**next_page_token** | Option<**String**> |  |  |

### Return type

[**models::TaskPaginatedResponse**](TaskPaginatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_tasks_by_context_id

> models::TaskPaginatedResponse list_tasks_by_context_id(page_size, context_id, next_page_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_size** | **i64** |  | [required] |
**context_id** | **uuid::Uuid** | Context ID | [required] |
**next_page_token** | Option<**String**> |  |  |

### Return type

[**models::TaskPaginatedResponse**](TaskPaginatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## resume_user_credential_brokering

> models::UserCredentialBrokeringResponse resume_user_credential_brokering(state, code, error, error_description)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**state** | Option<**String**> | OAuth state parameter |  |
**code** | Option<**String**> | OAuth authorization code |  |
**error** | Option<**String**> | OAuth error code |  |
**error_description** | Option<**String**> | OAuth error description |  |

### Return type

[**models::UserCredentialBrokeringResponse**](UserCredentialBrokeringResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## send_message

> models::CreateMessageResponse send_message(task_id, create_message_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**task_id** | **uuid::Uuid** | Task ID | [required] |
**create_message_request** | [**CreateMessageRequest**](CreateMessageRequest.md) |  | [required] |

### Return type

[**models::CreateMessageResponse**](CreateMessageResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## start_user_credential_brokering

> models::UserCredentialBrokeringResponse start_user_credential_brokering(provider_controller_type_id, credential_controller_type_id, start_user_credential_brokering_params_inner)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**provider_controller_type_id** | **String** | Provider controller type ID | [required] |
**credential_controller_type_id** | **String** | Credential controller type ID | [required] |
**start_user_credential_brokering_params_inner** | [**StartUserCredentialBrokeringParamsInner**](StartUserCredentialBrokeringParamsInner.md) |  | [required] |

### Return type

[**models::UserCredentialBrokeringResponse**](UserCredentialBrokeringResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## task_history

> models::TaskTimelineItemPaginatedResponse task_history(page_size, task_id, next_page_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_size** | **i64** |  | [required] |
**task_id** | **uuid::Uuid** | Task ID | [required] |
**next_page_token** | Option<**String**> |  |  |

### Return type

[**models::TaskTimelineItemPaginatedResponse**](TaskTimelineItemPaginatedResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_provider_instance

> serde_json::Value update_provider_instance(provider_instance_id, update_provider_instance_params_inner)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**provider_instance_id** | **String** | Provider instance ID | [required] |
**update_provider_instance_params_inner** | [**UpdateProviderInstanceParamsInner**](UpdateProviderInstanceParamsInner.md) |  | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_task_status

> serde_json::Value update_task_status(task_id, update_task_status_request)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**task_id** | **uuid::Uuid** | Task ID | [required] |
**update_task_status_request** | [**UpdateTaskStatusRequest**](UpdateTaskStatusRequest.md) |  | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

