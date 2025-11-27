
# CredentialConfig


## Properties

Name | Type
------------ | -------------
`dekAlias` | string
`id` | string
`metadata` | any
`nextRotationTime` | string
`typeId` | string
`value` | any

## Example

```typescript
import type { CredentialConfig } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "dekAlias": null,
  "id": null,
  "metadata": null,
  "nextRotationTime": null,
  "typeId": null,
  "value": null,
} satisfies CredentialConfig

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CredentialConfig
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


