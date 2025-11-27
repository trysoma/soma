
# FunctionInstanceSerialized


## Properties

Name | Type
------------ | -------------
`createdAt` | Date
`functionControllerTypeId` | string
`providerControllerTypeId` | string
`providerInstanceId` | string
`updatedAt` | Date

## Example

```typescript
import type { FunctionInstanceSerialized } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "createdAt": null,
  "functionControllerTypeId": null,
  "providerControllerTypeId": null,
  "providerInstanceId": null,
  "updatedAt": null,
} satisfies FunctionInstanceSerialized

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as FunctionInstanceSerialized
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


