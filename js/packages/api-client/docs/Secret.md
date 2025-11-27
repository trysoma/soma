
# Secret


## Properties

Name | Type
------------ | -------------
`createdAt` | Date
`dekAlias` | string
`encryptedSecret` | string
`id` | string
`key` | string
`updatedAt` | Date

## Example

```typescript
import type { Secret } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "createdAt": null,
  "dekAlias": null,
  "encryptedSecret": null,
  "id": null,
  "key": null,
  "updatedAt": null,
} satisfies Secret

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as Secret
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


