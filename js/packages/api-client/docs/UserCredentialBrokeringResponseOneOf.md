
# UserCredentialBrokeringResponseOneOf


## Properties

Name | Type
------------ | -------------
`action` | [BrokerAction](BrokerAction.md)
`createdAt` | Date
`credentialControllerTypeId` | string
`id` | string
`metadata` | { [key: string]: any; }
`providerControllerTypeId` | string
`providerInstanceId` | string
`updatedAt` | Date
`type` | string

## Example

```typescript
import type { UserCredentialBrokeringResponseOneOf } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "action": null,
  "createdAt": null,
  "credentialControllerTypeId": null,
  "id": null,
  "metadata": null,
  "providerControllerTypeId": null,
  "providerInstanceId": null,
  "updatedAt": null,
  "type": null,
} satisfies UserCredentialBrokeringResponseOneOf

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UserCredentialBrokeringResponseOneOf
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


