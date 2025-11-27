
# ProviderInstanceSerializedWithEverything


## Properties

Name | Type
------------ | -------------
`providerInstance` | [ProviderInstanceSerialized](ProviderInstanceSerialized.md)
`resourceServerCredential` | [ResourceServerCredentialSerialized](ResourceServerCredentialSerialized.md)
`userCredential` | [UserCredentialSerialized](UserCredentialSerialized.md)
`controller` | [ProviderControllerSerialized](ProviderControllerSerialized.md)
`credentialController` | [ProviderCredentialControllerSerialized](ProviderCredentialControllerSerialized.md)
`functions` | [Array&lt;FunctionInstanceListItem&gt;](FunctionInstanceListItem.md)

## Example

```typescript
import type { ProviderInstanceSerializedWithEverything } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "providerInstance": null,
  "resourceServerCredential": null,
  "userCredential": null,
  "controller": null,
  "credentialController": null,
  "functions": null,
} satisfies ProviderInstanceSerializedWithEverything

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ProviderInstanceSerializedWithEverything
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


