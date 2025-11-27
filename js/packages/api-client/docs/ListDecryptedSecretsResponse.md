
# ListDecryptedSecretsResponse


## Properties

Name | Type
------------ | -------------
`nextPageToken` | string
`secrets` | [Array&lt;DecryptedSecret&gt;](DecryptedSecret.md)

## Example

```typescript
import type { ListDecryptedSecretsResponse } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "nextPageToken": null,
  "secrets": null,
} satisfies ListDecryptedSecretsResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ListDecryptedSecretsResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


