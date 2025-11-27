
# TaskWithDetails


## Properties

Name | Type
------------ | -------------
`messages` | [Array&lt;Message&gt;](Message.md)
`messagesNextPageToken` | string
`statusMessage` | [Message](Message.md)
`task` | [Task](Task.md)

## Example

```typescript
import type { TaskWithDetails } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "messages": null,
  "messagesNextPageToken": null,
  "statusMessage": null,
  "task": null,
} satisfies TaskWithDetails

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as TaskWithDetails
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


