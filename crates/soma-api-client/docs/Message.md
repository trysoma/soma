# Message

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **String** |  | 
**id** | [**uuid::Uuid**](uuid::Uuid.md) |  | 
**metadata** | [**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md) |  | 
**parts** | [**Vec<models::MessagePart>**](MessagePart.md) |  | 
**reference_task_ids** | [**Vec<uuid::Uuid>**](uuid::Uuid.md) |  | 
**role** | [**models::MessageRole**](MessageRole.md) |  | 
**task_id** | [**uuid::Uuid**](uuid::Uuid.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


