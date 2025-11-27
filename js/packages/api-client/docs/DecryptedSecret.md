
# DecryptedSecret


## Properties

Name | Type
------------ | -------------
`createdAt` | Date
`decryptedValue` | string
`dekAlias` | string
`id` | string
`key` | string
`updatedAt` | Date

## Example

```typescript
import type { DecryptedSecret } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "createdAt": null,
  "decryptedValue": null,
  "dekAlias": null,
  "id": null,
  "key": null,
  "updatedAt": null,
} satisfies DecryptedSecret

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as DecryptedSecret
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


