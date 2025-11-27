
# EncryptionConfig

Top-level encryption configuration

## Properties

Name | Type
------------ | -------------
`envelopeKeys` | [{ [key: string]: EnvelopeKeyConfig; }](EnvelopeKeyConfig.md)

## Example

```typescript
import type { EncryptionConfig } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "envelopeKeys": null,
} satisfies EncryptionConfig

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as EncryptionConfig
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


