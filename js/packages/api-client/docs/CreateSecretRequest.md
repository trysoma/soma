
# CreateSecretRequest


## Properties

Name | Type
------------ | -------------
`dekAlias` | string
`key` | string
`rawValue` | string

## Example

```typescript
import type { CreateSecretRequest } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "dekAlias": null,
  "key": null,
  "rawValue": null,
} satisfies CreateSecretRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CreateSecretRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


