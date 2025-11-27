
# UserCredentialSerialized


## Properties

Name | Type
------------ | -------------
`createdAt` | Date
`dekAlias` | string
`id` | string
`metadata` | { [key: string]: any; }
`nextRotationTime` | Date
`typeId` | string
`updatedAt` | Date
`value` | any

## Example

```typescript
import type { UserCredentialSerialized } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "createdAt": null,
  "dekAlias": null,
  "id": null,
  "metadata": null,
  "nextRotationTime": null,
  "typeId": null,
  "updatedAt": null,
  "value": null,
} satisfies UserCredentialSerialized

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UserCredentialSerialized
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


