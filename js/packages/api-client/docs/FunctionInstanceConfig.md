
# FunctionInstanceConfig


## Properties

Name | Type
------------ | -------------
`functionController` | [FunctionControllerSerialized](FunctionControllerSerialized.md)
`providerController` | [ProviderControllerSerialized](ProviderControllerSerialized.md)
`providerInstances` | [Array&lt;ProviderInstanceSerializedWithCredentials&gt;](ProviderInstanceSerializedWithCredentials.md)

## Example

```typescript
import type { FunctionInstanceConfig } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "functionController": null,
  "providerController": null,
  "providerInstances": null,
} satisfies FunctionInstanceConfig

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as FunctionInstanceConfig
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


