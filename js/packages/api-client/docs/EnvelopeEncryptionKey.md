
# EnvelopeEncryptionKey


## Properties

Name | Type
------------ | -------------
`arn` | string
`region` | string
`type` | string
`fileName` | string

## Example

```typescript
import type { EnvelopeEncryptionKey } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "arn": null,
  "region": null,
  "type": null,
  "fileName": null,
} satisfies EnvelopeEncryptionKey

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as EnvelopeEncryptionKey
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


