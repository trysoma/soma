
# ProviderInstanceListItem


## Properties

Name | Type
------------ | -------------
`createdAt` | Date
`credentialControllerTypeId` | string
`displayName` | string
`id` | string
`providerControllerTypeId` | string
`resourceServerCredentialId` | string
`returnOnSuccessfulBrokering` | [ReturnAddress](ReturnAddress.md)
`status` | string
`updatedAt` | Date
`userCredentialId` | string
`controller` | [ProviderControllerSerialized](ProviderControllerSerialized.md)
`credentialController` | [ProviderCredentialControllerSerialized](ProviderCredentialControllerSerialized.md)
`functions` | [Array&lt;FunctionInstanceListItem&gt;](FunctionInstanceListItem.md)

## Example

```typescript
import type { ProviderInstanceListItem } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "createdAt": null,
  "credentialControllerTypeId": null,
  "displayName": null,
  "id": null,
  "providerControllerTypeId": null,
  "resourceServerCredentialId": null,
  "returnOnSuccessfulBrokering": null,
  "status": null,
  "updatedAt": null,
  "userCredentialId": null,
  "controller": null,
  "credentialController": null,
  "functions": null,
} satisfies ProviderInstanceListItem

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ProviderInstanceListItem
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


