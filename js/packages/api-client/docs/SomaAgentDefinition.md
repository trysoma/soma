
# SomaAgentDefinition


## Properties

Name | Type
------------ | -------------
`bridge` | [BridgeConfig](BridgeConfig.md)
`encryption` | [EncryptionConfig](EncryptionConfig.md)
`secrets` | [{ [key: string]: SecretConfig; }](SecretConfig.md)

## Example

```typescript
import type { SomaAgentDefinition } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "bridge": null,
  "encryption": null,
  "secrets": null,
} satisfies SomaAgentDefinition

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as SomaAgentDefinition
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


