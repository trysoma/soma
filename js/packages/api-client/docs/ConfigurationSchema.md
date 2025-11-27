
# ConfigurationSchema


## Properties

Name | Type
------------ | -------------
`resourceServer` | { [key: string]: any; }
`userCredential` | { [key: string]: any; }

## Example

```typescript
import type { ConfigurationSchema } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "resourceServer": null,
  "userCredential": null,
} satisfies ConfigurationSchema

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ConfigurationSchema
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


