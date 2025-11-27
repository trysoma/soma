
# TaskTimelineItem


## Properties

Name | Type
------------ | -------------
`createdAt` | Date
`eventPayload` | [TaskTimelineItemPayload](TaskTimelineItemPayload.md)
`id` | string
`taskId` | string

## Example

```typescript
import type { TaskTimelineItem } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "createdAt": null,
  "eventPayload": null,
  "id": null,
  "taskId": null,
} satisfies TaskTimelineItem

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as TaskTimelineItem
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


