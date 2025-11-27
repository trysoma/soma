
# FunctionControllerSerialized


## Properties

Name | Type
------------ | -------------
`categories` | Array&lt;string&gt;
`documentation` | string
`name` | string
`output` | { [key: string]: any; }
`parameters` | { [key: string]: any; }
`typeId` | string

## Example

```typescript
import type { FunctionControllerSerialized } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "categories": null,
  "documentation": null,
  "name": null,
  "output": null,
  "parameters": null,
  "typeId": null,
} satisfies FunctionControllerSerialized

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as FunctionControllerSerialized
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


