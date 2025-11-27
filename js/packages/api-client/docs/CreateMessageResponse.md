
# CreateMessageResponse


## Properties

Name | Type
------------ | -------------
`message` | [Message](Message.md)
`timelineItem` | [TaskTimelineItem](TaskTimelineItem.md)

## Example

```typescript
import type { CreateMessageResponse } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "message": null,
  "timelineItem": null,
} satisfies CreateMessageResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CreateMessageResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


