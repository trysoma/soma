
# EncryptCredentialConfigurationParamsInner

Parameters for encrypting credential configuration. Uses dek_alias to look up the DEK to use for encryption.

## Properties

Name | Type
------------ | -------------
`dekAlias` | string
`value` | any

## Example

```typescript
import type { EncryptCredentialConfigurationParamsInner } from '@trysoma/api-client'

// TODO: Update the object below with actual values
const example = {
  "dekAlias": null,
  "value": null,
} satisfies EncryptCredentialConfigurationParamsInner

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as EncryptCredentialConfigurationParamsInner
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


