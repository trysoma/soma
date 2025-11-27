
# CreateMessageRequest


## Properties

Name | Type
------------ | -------------
`metadata` | { [key: string]: any; }
`parts` | [Array&lt;MessagePart&gt;](MessagePart.md)
`referenceTaskIds` | Array&lt;string&gt;
`role` | [MessageRole](MessageRole.md)

## Example

```typescript
import type { CreateMessageRequest } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "metadata": null,
  "parts": null,
  "referenceTaskIds": null,
  "role": null,
} satisfies CreateMessageRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CreateMessageRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


