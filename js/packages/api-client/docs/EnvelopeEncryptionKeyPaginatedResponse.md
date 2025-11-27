
# EnvelopeEncryptionKeyPaginatedResponse


## Properties

Name | Type
------------ | -------------
`items` | [Array&lt;EnvelopeEncryptionKey&gt;](EnvelopeEncryptionKey.md)
`nextPageToken` | string

## Example

```typescript
import type { EnvelopeEncryptionKeyPaginatedResponse } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "items": null,
  "nextPageToken": null,
} satisfies EnvelopeEncryptionKeyPaginatedResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as EnvelopeEncryptionKeyPaginatedResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


