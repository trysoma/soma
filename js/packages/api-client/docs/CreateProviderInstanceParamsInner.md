
# CreateProviderInstanceParamsInner


## Properties

Name | Type
------------ | -------------
`displayName` | string
`providerInstanceId` | string
`resourceServerCredentialId` | string
`returnOnSuccessfulBrokering` | [ReturnAddress](ReturnAddress.md)
`userCredentialId` | string

## Example

```typescript
import type { CreateProviderInstanceParamsInner } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "displayName": null,
  "providerInstanceId": null,
  "resourceServerCredentialId": null,
  "returnOnSuccessfulBrokering": null,
  "userCredentialId": null,
} satisfies CreateProviderInstanceParamsInner

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CreateProviderInstanceParamsInner
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


