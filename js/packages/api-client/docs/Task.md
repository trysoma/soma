
# Task


## Properties

Name | Type
------------ | -------------
`contextId` | string
`createdAt` | Date
`id` | string
`metadata` | { [key: string]: any; }
`status` | [TaskStatus](TaskStatus.md)
`statusMessageId` | string
`statusTimestamp` | Date
`updatedAt` | Date

## Example

```typescript
import type { Task } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "contextId": null,
  "createdAt": null,
  "id": null,
  "metadata": null,
  "status": null,
  "statusMessageId": null,
  "statusTimestamp": null,
  "updatedAt": null,
} satisfies Task

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as Task
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


