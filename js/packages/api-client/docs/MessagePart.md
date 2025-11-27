
# MessagePart


## Properties

Name | Type
------------ | -------------
`metadata` | { [key: string]: any; }
`text` | string
`type` | string

## Example

```typescript
import type { MessagePart } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "metadata": null,
  "text": null,
  "type": null,
} satisfies MessagePart

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as MessagePart
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


