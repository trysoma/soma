
# ProviderControllerSerializedPaginatedResponse


## Properties

Name | Type
------------ | -------------
`items` | [Array&lt;ProviderControllerSerialized&gt;](ProviderControllerSerialized.md)
`nextPageToken` | string

## Example

```typescript
import type { ProviderControllerSerializedPaginatedResponse } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "items": null,
  "nextPageToken": null,
} satisfies ProviderControllerSerializedPaginatedResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ProviderControllerSerializedPaginatedResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


