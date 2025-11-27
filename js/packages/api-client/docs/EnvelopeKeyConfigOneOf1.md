
# EnvelopeKeyConfigOneOf1


## Properties

Name | Type
------------ | -------------
`deks` | [{ [key: string]: DekConfig; }](DekConfig.md)
`fileName` | string
`type` | string

## Example

```typescript
import type { EnvelopeKeyConfigOneOf1 } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "deks": null,
  "fileName": null,
  "type": null,
} satisfies EnvelopeKeyConfigOneOf1

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as EnvelopeKeyConfigOneOf1
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


