
# ProviderControllerSerialized


## Properties

Name | Type
------------ | -------------
`categories` | Array&lt;string&gt;
`credentialControllers` | [Array&lt;ProviderCredentialControllerSerialized&gt;](ProviderCredentialControllerSerialized.md)
`documentation` | string
`functions` | [Array&lt;FunctionControllerSerialized&gt;](FunctionControllerSerialized.md)
`name` | string
`typeId` | string

## Example

```typescript
import type { ProviderControllerSerialized } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "categories": null,
  "credentialControllers": null,
  "documentation": null,
  "functions": null,
  "name": null,
  "typeId": null,
} satisfies ProviderControllerSerialized

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ProviderControllerSerialized
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


