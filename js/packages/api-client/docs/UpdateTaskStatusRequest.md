
# UpdateTaskStatusRequest


## Properties

Name | Type
------------ | -------------
`message` | [CreateMessageRequest](CreateMessageRequest.md)
`status` | [TaskStatus](TaskStatus.md)

## Example

```typescript
import type { UpdateTaskStatusRequest } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "message": null,
  "status": null,
} satisfies UpdateTaskStatusRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UpdateTaskStatusRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


