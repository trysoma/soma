
# DataEncryptionKeyListItem


## Properties

Name | Type
------------ | -------------
`createdAt` | Date
`envelopeEncryptionKeyId` | [EnvelopeEncryptionKey](EnvelopeEncryptionKey.md)
`id` | string
`updatedAt` | Date

## Example

```typescript
import type { DataEncryptionKeyListItem } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "createdAt": null,
  "envelopeEncryptionKeyId": null,
  "id": null,
  "updatedAt": null,
} satisfies DataEncryptionKeyListItem

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as DataEncryptionKeyListItem
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


