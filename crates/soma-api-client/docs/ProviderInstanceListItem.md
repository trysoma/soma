# ProviderInstanceListItem

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **String** |  | 
**credential_controller_type_id** | **String** |  | 
**display_name** | **String** |  | 
**id** | **String** |  | 
**provider_controller_type_id** | **String** |  | 
**resource_server_credential_id** | [**uuid::Uuid**](uuid::Uuid.md) |  | 
**return_on_successful_brokering** | Option<[**models::ReturnAddress**](ReturnAddress.md)> |  | [optional]
**status** | **String** |  | 
**updated_at** | **String** |  | 
**user_credential_id** | Option<[**uuid::Uuid**](uuid::Uuid.md)> |  | [optional]
**controller** | [**models::ProviderControllerSerialized**](ProviderControllerSerialized.md) |  | 
**credential_controller** | [**models::ProviderCredentialControllerSerialized**](ProviderCredentialControllerSerialized.md) |  | 
**functions** | [**Vec<models::FunctionInstanceListItem>**](FunctionInstanceListItem.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


