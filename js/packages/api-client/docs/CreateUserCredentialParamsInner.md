
# CreateUserCredentialParamsInner


## Properties

Name | Type
------------ | -------------
`dekAlias` | string
`metadata` | { [key: string]: any; }
`userCredentialConfiguration` | any

## Example

```typescript
import type { CreateUserCredentialParamsInner } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "dekAlias": null,
  "metadata": null,
  "userCredentialConfiguration": null,
} satisfies CreateUserCredentialParamsInner

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CreateUserCredentialParamsInner
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


