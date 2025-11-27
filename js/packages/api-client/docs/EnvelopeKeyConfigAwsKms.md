
# EnvelopeKeyConfigAwsKms

Envelope encryption key configuration with nested DEKs

## Properties

Name | Type
------------ | -------------
`arn` | string
`deks` | [{ [key: string]: DekConfig; }](DekConfig.md)
`region` | string

## Example

```typescript
import type { EnvelopeKeyConfigAwsKms } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "arn": null,
  "deks": null,
  "region": null,
} satisfies EnvelopeKeyConfigAwsKms

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as EnvelopeKeyConfigAwsKms
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


