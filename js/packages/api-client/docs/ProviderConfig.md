
# ProviderConfig


## Properties

Name | Type
------------ | -------------
`credentialControllerTypeId` | string
`displayName` | string
`functions` | Array&lt;string&gt;
`providerControllerTypeId` | string
`resourceServerCredential` | [CredentialConfig](CredentialConfig.md)
`userCredential` | [CredentialConfig](CredentialConfig.md)

## Example

```typescript
import type { ProviderConfig } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "credentialControllerTypeId": null,
  "displayName": null,
  "functions": null,
  "providerControllerTypeId": null,
  "resourceServerCredential": null,
  "userCredential": null,
} satisfies ProviderConfig

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ProviderConfig
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


