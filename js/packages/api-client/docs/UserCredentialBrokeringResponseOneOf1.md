
# UserCredentialBrokeringResponseOneOf1


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
`type` | string

## Example

```typescript
import type { UserCredentialBrokeringResponseOneOf1 } from '@trysoma/api-client'

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
  "type": null,
} satisfies UserCredentialBrokeringResponseOneOf1

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UserCredentialBrokeringResponseOneOf1
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


