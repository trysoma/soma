
# DataEncryptionKeyAlias

Data encryption key alias struct

## Properties

Name | Type
------------ | -------------
`alias` | string
`createdAt` | Date
`dataEncryptionKeyId` | string

## Example

```typescript
import type { DataEncryptionKeyAlias } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "alias": null,
  "createdAt": null,
  "dataEncryptionKeyId": null,
} satisfies DataEncryptionKeyAlias

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as DataEncryptionKeyAlias
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


