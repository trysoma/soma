
# ProviderCredentialControllerSerialized


## Properties

Name | Type
------------ | -------------
`configurationSchema` | [ConfigurationSchema](ConfigurationSchema.md)
`documentation` | string
`name` | string
`requiresBrokering` | boolean
`requiresResourceServerCredentialRefreshing` | boolean
`requiresUserCredentialRefreshing` | boolean
`typeId` | string

## Example

```typescript
import type { ProviderCredentialControllerSerialized } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "configurationSchema": null,
  "documentation": null,
  "name": null,
  "requiresBrokering": null,
  "requiresResourceServerCredentialRefreshing": null,
  "requiresUserCredentialRefreshing": null,
  "typeId": null,
} satisfies ProviderCredentialControllerSerialized

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ProviderCredentialControllerSerialized
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


