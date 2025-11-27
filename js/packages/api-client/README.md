# @trysoma/api-client@

A TypeScript SDK client for the localhost API.

## Usage

First, install the SDK from npm.

```bash
npm install @trysoma/api-client --save
```

Next, try it out.


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


## Documentation

### API Endpoints

All URIs are relative to *http://localhost*

| Class | Method | HTTP request | Description
| ----- | ------ | ------------ | -------------
*A2aApi* | [**getAgentCard**](docs/A2aApi.md#getagentcard) | **GET** /api/a2a/v1/.well-known/agent.json | Get agent card
*A2aApi* | [**getAgentDefinition**](docs/A2aApi.md#getagentdefinition) | **GET** /api/a2a/v1/definition | Get agent definition
*A2aApi* | [**getExtendedAgentCard**](docs/A2aApi.md#getextendedagentcard) | **GET** /api/a2a/v1/agent/authenticatedExtendedCard | Get extended agent card
*A2aApi* | [**handleJsonrpcRequest**](docs/A2aApi.md#handlejsonrpcrequest) | **POST** /api/a2a/v1 | Handle JSON-RPC
*BridgeApi* | [**createProviderInstance**](docs/BridgeApi.md#createproviderinstance) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id} | Create provider
*BridgeApi* | [**createResourceServerCredential**](docs/BridgeApi.md#createresourceservercredential) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/resource-server | Create resource server credential
*BridgeApi* | [**createUserCredential**](docs/BridgeApi.md#createusercredential) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential | Create user credential
*BridgeApi* | [**deleteProviderInstance**](docs/BridgeApi.md#deleteproviderinstance) | **DELETE** /api/bridge/v1/provider/{provider_instance_id} | Delete provider
*BridgeApi* | [**disableFunction**](docs/BridgeApi.md#disablefunction) | **POST** /api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/disable | Disable function
*BridgeApi* | [**enableFunction**](docs/BridgeApi.md#enablefunction) | **POST** /api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/enable | Enable function
*BridgeApi* | [**encryptResourceServerConfiguration**](docs/BridgeApi.md#encryptresourceserverconfiguration) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/resource-server/encrypt | Encrypt resource server config
*BridgeApi* | [**encryptUserCredentialConfiguration**](docs/BridgeApi.md#encryptusercredentialconfiguration) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential/encrypt | Encrypt user credential config
*BridgeApi* | [**getFunctionInstancesOpenapiSpec**](docs/BridgeApi.md#getfunctioninstancesopenapispec) | **GET** /api/bridge/v1/function-instances/openapi.json | Get function OpenAPI spec
*BridgeApi* | [**getProviderInstance**](docs/BridgeApi.md#getproviderinstance) | **GET** /api/bridge/v1/provider/{provider_instance_id} | Get provider
*BridgeApi* | [**invokeFunction**](docs/BridgeApi.md#invokefunction) | **POST** /api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/invoke | Invoke function
*BridgeApi* | [**listAvailableProviders**](docs/BridgeApi.md#listavailableproviders) | **GET** /api/bridge/v1/available-providers | List providers
*BridgeApi* | [**listFunctionInstances**](docs/BridgeApi.md#listfunctioninstances) | **GET** /api/bridge/v1/function-instances | List function instances
*BridgeApi* | [**listProviderInstances**](docs/BridgeApi.md#listproviderinstances) | **GET** /api/bridge/v1/provider | List provider instances
*BridgeApi* | [**listProviderInstancesGroupedByFunction**](docs/BridgeApi.md#listproviderinstancesgroupedbyfunction) | **GET** /api/bridge/v1/provider/grouped-by-function | List providers by function
*BridgeApi* | [**listenToMcpSse**](docs/BridgeApi.md#listentomcpsse) | **GET** /api/bridge/v1/mcp | MCP SSE connection
*BridgeApi* | [**resumeUserCredentialBrokering**](docs/BridgeApi.md#resumeusercredentialbrokering) | **GET** /api/bridge/v1/generic-oauth-callback | OAuth callback
*BridgeApi* | [**startUserCredentialBrokering**](docs/BridgeApi.md#startusercredentialbrokering) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential/broker | Start credential brokering
*BridgeApi* | [**triggerMcpMessage**](docs/BridgeApi.md#triggermcpmessage) | **POST** /api/bridge/v1/mcp | Send MCP message
*BridgeApi* | [**updateProviderInstance**](docs/BridgeApi.md#updateproviderinstance) | **PATCH** /api/bridge/v1/provider/{provider_instance_id} | Update provider
*EncryptionApi* | [**createDataEncryptionKey**](docs/EncryptionApi.md#createdataencryptionkey) | **POST** /api/encryption/v1/envelope/{envelope_id}/dek | Create data key
*EncryptionApi* | [**createDekAlias**](docs/EncryptionApi.md#createdekaliasoperation) | **POST** /api/encryption/v1/dek/alias | Create DEK alias
*EncryptionApi* | [**createEnvelopeEncryptionKey**](docs/EncryptionApi.md#createenvelopeencryptionkey) | **POST** /api/encryption/v1/envelope | Create envelope key
*EncryptionApi* | [**deleteDekAlias**](docs/EncryptionApi.md#deletedekalias) | **DELETE** /api/encryption/v1/dek/alias/{alias} | Delete DEK alias
*EncryptionApi* | [**getDekByAliasOrId**](docs/EncryptionApi.md#getdekbyaliasorid) | **GET** /api/encryption/v1/dek/alias/{alias} | Get DEK by alias
*EncryptionApi* | [**importDataEncryptionKey**](docs/EncryptionApi.md#importdataencryptionkey) | **POST** /api/encryption/v1/envelope/{envelope_id}/dek/import | Import data key
*EncryptionApi* | [**listDataEncryptionKeysByEnvelope**](docs/EncryptionApi.md#listdataencryptionkeysbyenvelope) | **GET** /api/encryption/v1/envelope/{envelope_id}/dek | List data keys
*EncryptionApi* | [**listEnvelopeEncryptionKeys**](docs/EncryptionApi.md#listenvelopeencryptionkeys) | **GET** /api/encryption/v1/envelope | List envelope keys
*EncryptionApi* | [**migrateAllDataEncryptionKeys**](docs/EncryptionApi.md#migratealldataencryptionkeys) | **POST** /api/encryption/v1/envelope/{envelope_id}/migrate | Migrate all data keys
*EncryptionApi* | [**migrateDataEncryptionKey**](docs/EncryptionApi.md#migratedataencryptionkey) | **POST** /api/encryption/v1/envelope/{envelope_id}/dek/{dek_id}/migrate | Migrate data key
*EncryptionApi* | [**updateDekAlias**](docs/EncryptionApi.md#updatedekalias) | **PUT** /api/encryption/v1/dek/alias/{alias} | Update DEK alias
*InternalApi* | [**getInternalRuntimeConfig**](docs/InternalApi.md#getinternalruntimeconfig) | **GET** /_internal/v1/runtime_config | Get runtime config
*InternalApi* | [**healthCheck**](docs/InternalApi.md#healthcheck) | **GET** /_internal/v1/health | Health check
*InternalApi* | [**triggerCodegen**](docs/InternalApi.md#triggercodegen) | **POST** /_internal/v1/trigger_codegen | Trigger codegen
*SecretApi* | [**createSecret**](docs/SecretApi.md#createsecretoperation) | **POST** /api/secret/v1 | Create secret
*SecretApi* | [**deleteSecret**](docs/SecretApi.md#deletesecret) | **DELETE** /api/secret/v1/{secret_id} | Delete secret
*SecretApi* | [**getSecretById**](docs/SecretApi.md#getsecretbyid) | **GET** /api/secret/v1/{secret_id} | Get secret
*SecretApi* | [**getSecretByKey**](docs/SecretApi.md#getsecretbykey) | **GET** /api/secret/v1/key/{key} | Get secret by key
*SecretApi* | [**importSecret**](docs/SecretApi.md#importsecretoperation) | **POST** /api/secret/v1/import | Import secret
*SecretApi* | [**listDecryptedSecrets**](docs/SecretApi.md#listdecryptedsecrets) | **GET** /api/secret/v1/list-decrypted | List decrypted secrets
*SecretApi* | [**listSecrets**](docs/SecretApi.md#listsecrets) | **GET** /api/secret/v1 | List secrets
*SecretApi* | [**updateSecret**](docs/SecretApi.md#updatesecretoperation) | **PUT** /api/secret/v1/{secret_id} | Update secret
*TaskApi* | [**getTaskById**](docs/TaskApi.md#gettaskbyid) | **GET** /api/task/v1/{task_id} | Get task
*TaskApi* | [**listContexts**](docs/TaskApi.md#listcontexts) | **GET** /api/task/v1/context | List contexts
*TaskApi* | [**listTasks**](docs/TaskApi.md#listtasks) | **GET** /api/task/v1 | List tasks
*TaskApi* | [**listTasksByContextId**](docs/TaskApi.md#listtasksbycontextid) | **GET** /api/task/v1/context/{context_id}/task | List tasks by context
*TaskApi* | [**sendMessage**](docs/TaskApi.md#sendmessage) | **POST** /api/task/v1/{task_id}/message | Send message
*TaskApi* | [**taskHistory**](docs/TaskApi.md#taskhistory) | **GET** /api/task/v1/{task_id}/timeline | Get task timeline
*TaskApi* | [**updateTaskStatus**](docs/TaskApi.md#updatetaskstatusoperation) | **PUT** /api/task/v1/{task_id} | Update task status
*V1Api* | [**createDataEncryptionKey**](docs/V1Api.md#createdataencryptionkey) | **POST** /api/encryption/v1/envelope/{envelope_id}/dek | Create data key
*V1Api* | [**createDekAlias**](docs/V1Api.md#createdekaliasoperation) | **POST** /api/encryption/v1/dek/alias | Create DEK alias
*V1Api* | [**createEnvelopeEncryptionKey**](docs/V1Api.md#createenvelopeencryptionkey) | **POST** /api/encryption/v1/envelope | Create envelope key
*V1Api* | [**createProviderInstance**](docs/V1Api.md#createproviderinstance) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id} | Create provider
*V1Api* | [**createResourceServerCredential**](docs/V1Api.md#createresourceservercredential) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/resource-server | Create resource server credential
*V1Api* | [**createSecret**](docs/V1Api.md#createsecretoperation) | **POST** /api/secret/v1 | Create secret
*V1Api* | [**createUserCredential**](docs/V1Api.md#createusercredential) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential | Create user credential
*V1Api* | [**deleteDekAlias**](docs/V1Api.md#deletedekalias) | **DELETE** /api/encryption/v1/dek/alias/{alias} | Delete DEK alias
*V1Api* | [**deleteProviderInstance**](docs/V1Api.md#deleteproviderinstance) | **DELETE** /api/bridge/v1/provider/{provider_instance_id} | Delete provider
*V1Api* | [**deleteSecret**](docs/V1Api.md#deletesecret) | **DELETE** /api/secret/v1/{secret_id} | Delete secret
*V1Api* | [**disableFunction**](docs/V1Api.md#disablefunction) | **POST** /api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/disable | Disable function
*V1Api* | [**enableFunction**](docs/V1Api.md#enablefunction) | **POST** /api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/enable | Enable function
*V1Api* | [**encryptResourceServerConfiguration**](docs/V1Api.md#encryptresourceserverconfiguration) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/resource-server/encrypt | Encrypt resource server config
*V1Api* | [**encryptUserCredentialConfiguration**](docs/V1Api.md#encryptusercredentialconfiguration) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential/encrypt | Encrypt user credential config
*V1Api* | [**getAgentCard**](docs/V1Api.md#getagentcard) | **GET** /api/a2a/v1/.well-known/agent.json | Get agent card
*V1Api* | [**getAgentDefinition**](docs/V1Api.md#getagentdefinition) | **GET** /api/a2a/v1/definition | Get agent definition
*V1Api* | [**getDekByAliasOrId**](docs/V1Api.md#getdekbyaliasorid) | **GET** /api/encryption/v1/dek/alias/{alias} | Get DEK by alias
*V1Api* | [**getExtendedAgentCard**](docs/V1Api.md#getextendedagentcard) | **GET** /api/a2a/v1/agent/authenticatedExtendedCard | Get extended agent card
*V1Api* | [**getFunctionInstancesOpenapiSpec**](docs/V1Api.md#getfunctioninstancesopenapispec) | **GET** /api/bridge/v1/function-instances/openapi.json | Get function OpenAPI spec
*V1Api* | [**getInternalRuntimeConfig**](docs/V1Api.md#getinternalruntimeconfig) | **GET** /_internal/v1/runtime_config | Get runtime config
*V1Api* | [**getProviderInstance**](docs/V1Api.md#getproviderinstance) | **GET** /api/bridge/v1/provider/{provider_instance_id} | Get provider
*V1Api* | [**getSecretById**](docs/V1Api.md#getsecretbyid) | **GET** /api/secret/v1/{secret_id} | Get secret
*V1Api* | [**getSecretByKey**](docs/V1Api.md#getsecretbykey) | **GET** /api/secret/v1/key/{key} | Get secret by key
*V1Api* | [**getTaskById**](docs/V1Api.md#gettaskbyid) | **GET** /api/task/v1/{task_id} | Get task
*V1Api* | [**handleJsonrpcRequest**](docs/V1Api.md#handlejsonrpcrequest) | **POST** /api/a2a/v1 | Handle JSON-RPC
*V1Api* | [**healthCheck**](docs/V1Api.md#healthcheck) | **GET** /_internal/v1/health | Health check
*V1Api* | [**importDataEncryptionKey**](docs/V1Api.md#importdataencryptionkey) | **POST** /api/encryption/v1/envelope/{envelope_id}/dek/import | Import data key
*V1Api* | [**importSecret**](docs/V1Api.md#importsecretoperation) | **POST** /api/secret/v1/import | Import secret
*V1Api* | [**invokeFunction**](docs/V1Api.md#invokefunction) | **POST** /api/bridge/v1/provider/{provider_instance_id}/function/{function_controller_type_id}/invoke | Invoke function
*V1Api* | [**listAvailableProviders**](docs/V1Api.md#listavailableproviders) | **GET** /api/bridge/v1/available-providers | List providers
*V1Api* | [**listContexts**](docs/V1Api.md#listcontexts) | **GET** /api/task/v1/context | List contexts
*V1Api* | [**listDataEncryptionKeysByEnvelope**](docs/V1Api.md#listdataencryptionkeysbyenvelope) | **GET** /api/encryption/v1/envelope/{envelope_id}/dek | List data keys
*V1Api* | [**listDecryptedSecrets**](docs/V1Api.md#listdecryptedsecrets) | **GET** /api/secret/v1/list-decrypted | List decrypted secrets
*V1Api* | [**listEnvelopeEncryptionKeys**](docs/V1Api.md#listenvelopeencryptionkeys) | **GET** /api/encryption/v1/envelope | List envelope keys
*V1Api* | [**listFunctionInstances**](docs/V1Api.md#listfunctioninstances) | **GET** /api/bridge/v1/function-instances | List function instances
*V1Api* | [**listProviderInstances**](docs/V1Api.md#listproviderinstances) | **GET** /api/bridge/v1/provider | List provider instances
*V1Api* | [**listProviderInstancesGroupedByFunction**](docs/V1Api.md#listproviderinstancesgroupedbyfunction) | **GET** /api/bridge/v1/provider/grouped-by-function | List providers by function
*V1Api* | [**listSecrets**](docs/V1Api.md#listsecrets) | **GET** /api/secret/v1 | List secrets
*V1Api* | [**listTasks**](docs/V1Api.md#listtasks) | **GET** /api/task/v1 | List tasks
*V1Api* | [**listTasksByContextId**](docs/V1Api.md#listtasksbycontextid) | **GET** /api/task/v1/context/{context_id}/task | List tasks by context
*V1Api* | [**listenToMcpSse**](docs/V1Api.md#listentomcpsse) | **GET** /api/bridge/v1/mcp | MCP SSE connection
*V1Api* | [**migrateAllDataEncryptionKeys**](docs/V1Api.md#migratealldataencryptionkeys) | **POST** /api/encryption/v1/envelope/{envelope_id}/migrate | Migrate all data keys
*V1Api* | [**migrateDataEncryptionKey**](docs/V1Api.md#migratedataencryptionkey) | **POST** /api/encryption/v1/envelope/{envelope_id}/dek/{dek_id}/migrate | Migrate data key
*V1Api* | [**resumeUserCredentialBrokering**](docs/V1Api.md#resumeusercredentialbrokering) | **GET** /api/bridge/v1/generic-oauth-callback | OAuth callback
*V1Api* | [**sendMessage**](docs/V1Api.md#sendmessage) | **POST** /api/task/v1/{task_id}/message | Send message
*V1Api* | [**startUserCredentialBrokering**](docs/V1Api.md#startusercredentialbrokering) | **POST** /api/bridge/v1/available-providers/{provider_controller_type_id}/available-credentials/{credential_controller_type_id}/credential/user-credential/broker | Start credential brokering
*V1Api* | [**taskHistory**](docs/V1Api.md#taskhistory) | **GET** /api/task/v1/{task_id}/timeline | Get task timeline
*V1Api* | [**triggerCodegen**](docs/V1Api.md#triggercodegen) | **POST** /_internal/v1/trigger_codegen | Trigger codegen
*V1Api* | [**triggerMcpMessage**](docs/V1Api.md#triggermcpmessage) | **POST** /api/bridge/v1/mcp | Send MCP message
*V1Api* | [**updateDekAlias**](docs/V1Api.md#updatedekalias) | **PUT** /api/encryption/v1/dek/alias/{alias} | Update DEK alias
*V1Api* | [**updateProviderInstance**](docs/V1Api.md#updateproviderinstance) | **PATCH** /api/bridge/v1/provider/{provider_instance_id} | Update provider
*V1Api* | [**updateSecret**](docs/V1Api.md#updatesecretoperation) | **PUT** /api/secret/v1/{secret_id} | Update secret
*V1Api* | [**updateTaskStatus**](docs/V1Api.md#updatetaskstatusoperation) | **PUT** /api/task/v1/{task_id} | Update task status


### Models

- [BridgeConfig](docs/BridgeConfig.md)
- [BrokerAction](docs/BrokerAction.md)
- [BrokerActionOneOf](docs/BrokerActionOneOf.md)
- [BrokerActionRedirect](docs/BrokerActionRedirect.md)
- [BrokerState](docs/BrokerState.md)
- [ConfigurationSchema](docs/ConfigurationSchema.md)
- [ContextInfo](docs/ContextInfo.md)
- [ContextInfoPaginatedResponse](docs/ContextInfoPaginatedResponse.md)
- [CreateDataEncryptionKeyParamsRoute](docs/CreateDataEncryptionKeyParamsRoute.md)
- [CreateDekAliasRequest](docs/CreateDekAliasRequest.md)
- [CreateMessageRequest](docs/CreateMessageRequest.md)
- [CreateMessageResponse](docs/CreateMessageResponse.md)
- [CreateProviderInstanceParamsInner](docs/CreateProviderInstanceParamsInner.md)
- [CreateResourceServerCredentialParamsInner](docs/CreateResourceServerCredentialParamsInner.md)
- [CreateSecretRequest](docs/CreateSecretRequest.md)
- [CreateUserCredentialParamsInner](docs/CreateUserCredentialParamsInner.md)
- [CredentialConfig](docs/CredentialConfig.md)
- [DataEncryptionKey](docs/DataEncryptionKey.md)
- [DataEncryptionKeyAlias](docs/DataEncryptionKeyAlias.md)
- [DataEncryptionKeyListItem](docs/DataEncryptionKeyListItem.md)
- [DataEncryptionKeyListItemPaginatedResponse](docs/DataEncryptionKeyListItemPaginatedResponse.md)
- [DecryptedSecret](docs/DecryptedSecret.md)
- [DekConfig](docs/DekConfig.md)
- [DeleteSecretResponse](docs/DeleteSecretResponse.md)
- [EncryptCredentialConfigurationParamsInner](docs/EncryptCredentialConfigurationParamsInner.md)
- [EncryptionConfig](docs/EncryptionConfig.md)
- [EnvelopeEncryptionKey](docs/EnvelopeEncryptionKey.md)
- [EnvelopeEncryptionKeyAwsKms](docs/EnvelopeEncryptionKeyAwsKms.md)
- [EnvelopeEncryptionKeyLocal](docs/EnvelopeEncryptionKeyLocal.md)
- [EnvelopeEncryptionKeyOneOf](docs/EnvelopeEncryptionKeyOneOf.md)
- [EnvelopeEncryptionKeyOneOf1](docs/EnvelopeEncryptionKeyOneOf1.md)
- [EnvelopeEncryptionKeyPaginatedResponse](docs/EnvelopeEncryptionKeyPaginatedResponse.md)
- [EnvelopeKeyConfig](docs/EnvelopeKeyConfig.md)
- [EnvelopeKeyConfigAwsKms](docs/EnvelopeKeyConfigAwsKms.md)
- [EnvelopeKeyConfigLocal](docs/EnvelopeKeyConfigLocal.md)
- [EnvelopeKeyConfigOneOf](docs/EnvelopeKeyConfigOneOf.md)
- [EnvelopeKeyConfigOneOf1](docs/EnvelopeKeyConfigOneOf1.md)
- [FunctionControllerSerialized](docs/FunctionControllerSerialized.md)
- [FunctionInstanceConfig](docs/FunctionInstanceConfig.md)
- [FunctionInstanceConfigPaginatedResponse](docs/FunctionInstanceConfigPaginatedResponse.md)
- [FunctionInstanceListItem](docs/FunctionInstanceListItem.md)
- [FunctionInstanceSerialized](docs/FunctionInstanceSerialized.md)
- [FunctionInstanceSerializedPaginatedResponse](docs/FunctionInstanceSerializedPaginatedResponse.md)
- [ImportDataEncryptionKeyParamsRoute](docs/ImportDataEncryptionKeyParamsRoute.md)
- [ImportSecretRequest](docs/ImportSecretRequest.md)
- [InvokeError](docs/InvokeError.md)
- [InvokeFunctionParamsInner](docs/InvokeFunctionParamsInner.md)
- [InvokeResult](docs/InvokeResult.md)
- [InvokeResultOneOf](docs/InvokeResultOneOf.md)
- [InvokeResultOneOf1](docs/InvokeResultOneOf1.md)
- [ListDecryptedSecretsResponse](docs/ListDecryptedSecretsResponse.md)
- [ListSecretsResponse](docs/ListSecretsResponse.md)
- [Message](docs/Message.md)
- [MessagePart](docs/MessagePart.md)
- [MessageRole](docs/MessageRole.md)
- [MessageTaskTimelineItem](docs/MessageTaskTimelineItem.md)
- [MigrateAllDataEncryptionKeysParamsRoute](docs/MigrateAllDataEncryptionKeysParamsRoute.md)
- [MigrateDataEncryptionKeyParamsRoute](docs/MigrateDataEncryptionKeyParamsRoute.md)
- [ModelError](docs/ModelError.md)
- [ProviderConfig](docs/ProviderConfig.md)
- [ProviderControllerSerialized](docs/ProviderControllerSerialized.md)
- [ProviderControllerSerializedPaginatedResponse](docs/ProviderControllerSerializedPaginatedResponse.md)
- [ProviderCredentialControllerSerialized](docs/ProviderCredentialControllerSerialized.md)
- [ProviderInstanceListItem](docs/ProviderInstanceListItem.md)
- [ProviderInstanceListItemPaginatedResponse](docs/ProviderInstanceListItemPaginatedResponse.md)
- [ProviderInstanceSerialized](docs/ProviderInstanceSerialized.md)
- [ProviderInstanceSerializedWithCredentials](docs/ProviderInstanceSerializedWithCredentials.md)
- [ProviderInstanceSerializedWithEverything](docs/ProviderInstanceSerializedWithEverything.md)
- [ResourceServerCredentialSerialized](docs/ResourceServerCredentialSerialized.md)
- [ReturnAddress](docs/ReturnAddress.md)
- [ReturnAddressUrl](docs/ReturnAddressUrl.md)
- [Secret](docs/Secret.md)
- [SecretConfig](docs/SecretConfig.md)
- [SomaAgentDefinition](docs/SomaAgentDefinition.md)
- [StartUserCredentialBrokeringParamsInner](docs/StartUserCredentialBrokeringParamsInner.md)
- [Task](docs/Task.md)
- [TaskPaginatedResponse](docs/TaskPaginatedResponse.md)
- [TaskStatus](docs/TaskStatus.md)
- [TaskStatusUpdateTaskTimelineItem](docs/TaskStatusUpdateTaskTimelineItem.md)
- [TaskTimelineItem](docs/TaskTimelineItem.md)
- [TaskTimelineItemPaginatedResponse](docs/TaskTimelineItemPaginatedResponse.md)
- [TaskTimelineItemPayload](docs/TaskTimelineItemPayload.md)
- [TaskTimelineItemPayloadOneOf](docs/TaskTimelineItemPayloadOneOf.md)
- [TaskTimelineItemPayloadOneOf1](docs/TaskTimelineItemPayloadOneOf1.md)
- [TaskWithDetails](docs/TaskWithDetails.md)
- [TextPart](docs/TextPart.md)
- [TriggerCodegenResponse](docs/TriggerCodegenResponse.md)
- [UpdateAliasParams](docs/UpdateAliasParams.md)
- [UpdateProviderInstanceParamsInner](docs/UpdateProviderInstanceParamsInner.md)
- [UpdateSecretRequest](docs/UpdateSecretRequest.md)
- [UpdateTaskStatusRequest](docs/UpdateTaskStatusRequest.md)
- [UserCredentialBrokeringResponse](docs/UserCredentialBrokeringResponse.md)
- [UserCredentialBrokeringResponseOneOf](docs/UserCredentialBrokeringResponseOneOf.md)
- [UserCredentialBrokeringResponseOneOf1](docs/UserCredentialBrokeringResponseOneOf1.md)
- [UserCredentialBrokeringResponseOneOf2](docs/UserCredentialBrokeringResponseOneOf2.md)
- [UserCredentialSerialized](docs/UserCredentialSerialized.md)

### Authorization

Endpoints do not require authorization.


## About

This TypeScript SDK client supports the [Fetch API](https://fetch.spec.whatwg.org/)
and is automatically generated by the
[OpenAPI Generator](https://openapi-generator.tech) project:

- API version: `v1`
- Package version: ``
- Generator version: `7.17.0`
- Build package: `org.openapitools.codegen.languages.TypeScriptFetchClientCodegen`

The generated npm module supports the following:

- Environments
  * Node.js
  * Webpack
  * Browserify
- Language levels
  * ES5 - you must have a Promises/A+ library installed
  * ES6
- Module systems
  * CommonJS
  * ES6 module system


## Development

### Building

To build the TypeScript source code, you need to have Node.js and npm installed.
After cloning the repository, navigate to the project directory and run:

```bash
npm install
npm run build
```

### Publishing

Once you've built the package, you can publish it to npm:

```bash
npm publish
```

## License

[]()
