
# DataEncryptionKey


## Properties

Name | Type
------------ | -------------
`createdAt` | Date
`encryptedDataEncryptionKey` | string
`envelopeEncryptionKeyId` | [EnvelopeEncryptionKey](EnvelopeEncryptionKey.md)
`id` | string
`updatedAt` | Date

## Example

```typescript
import type { DataEncryptionKey } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "createdAt": null,
  "encryptedDataEncryptionKey": null,
  "envelopeEncryptionKeyId": null,
  "id": null,
  "updatedAt": null,
} satisfies DataEncryptionKey

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as DataEncryptionKey
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


