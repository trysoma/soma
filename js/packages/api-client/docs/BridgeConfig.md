
# BridgeConfig


## Properties

Name | Type
------------ | -------------
`providers` | [{ [key: string]: ProviderConfig; }](ProviderConfig.md)

## Example

```typescript
import type { BridgeConfig } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "providers": null,
} satisfies BridgeConfig

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as BridgeConfig
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


