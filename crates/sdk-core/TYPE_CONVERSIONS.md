# Type Conversions in SDK Core

The SDK Core uses clean, idiomatic Rust types instead of directly exposing protobuf-generated types. This provides better ergonomics, type safety, and error handling.

## Architecture

### Clean Types (types.rs)

All public types in `sdk_core` are defined in `types.rs`:

- `ProviderController`
- `FunctionController`
- `ProviderCredentialController` (enum)
- `Oauth2AuthorizationCodeFlowStaticCredentialConfiguration`
- `Metadata`
- `InvokeFunctionRequest`
- `InvokeFunctionResponse`
- `MetadataResponse`

These types:
- Are fully owned (no lifetimes)
- Implement `Serialize` and `Deserialize`
- Have clean, idiomatic field names
- Use `Option` where appropriate

### Conversion Strategy

We use two conversion traits:

1. **`TryFrom<Proto> for Type`** - For proto → clean types
   - Returns `Result<Type, CommonError>`
   - Validates required fields
   - Returns meaningful errors for missing or invalid data

2. **`From<Type> for Proto`** - For clean types → proto
   - Infallible conversion
   - Always succeeds because clean types are validated

## Error Handling

All conversions use `shared::error::CommonError`:

```rust
impl TryFrom<sdk_proto::ProviderController> for ProviderController {
    type Error = CommonError;

    fn try_from(proto: sdk_proto::ProviderController) -> Result<Self, Self::Error> {
        // Validation and conversion with proper error messages
    }
}
```

### Error Cases

**InvalidRequest** errors are returned when:
- Required fields are missing (e.g., OAuth2 missing `static_credential_configuration`)
- Enum variants have no `kind` set
- Nested conversions fail

Example:
```rust
None => Err(CommonError::InvalidRequest {
    msg: "ProviderCredentialController missing kind".to_string(),
    source: None,
})
```

## Usage in gRPC Service

The gRPC service implementation uses these conversions:

```rust
async fn invoke_function(
    &self,
    request: Request<sdk_proto::InvokeFunctionRequest>,
) -> Result<Response<sdk_proto::InvokeFunctionResponse>, Status> {
    let proto_req = request.into_inner();

    // Convert proto to clean type with error handling
    let req: InvokeFunctionRequest = proto_req
        .try_into()
        .map_err(|e: CommonError| {
            Status::invalid_argument(format!("Invalid request: {}", e))
        })?;

    // ... process request ...

    // Convert response back to proto
    let response = InvokeFunctionResponse { result };
    Ok(Response::new(response.into()))
}
```

## Benefits

1. **Type Safety**: Clean types prevent invalid states at compile time
2. **Better Errors**: `TryFrom` provides validation with meaningful error messages
3. **Ergonomics**: No dealing with `Option<Kind>` everywhere
4. **Testability**: Easy to construct test data without proto complexity
5. **Separation of Concerns**: Proto types are implementation details

## Language Bindings

Both `sdk-js` and `sdk-py` use the clean types from `sdk_core`:

```rust
use sdk_core::{ProviderController, FunctionInvocation, ...};
```

They serialize/deserialize using serde_json:

```rust
let providers: Vec<ProviderController> = serde_json::from_str(&providers_json)?;
```

This means language bindings work with JSON that matches the clean type structure, not the proto structure.

## Example Type Hierarchy

```
ProviderController
├── type_id: String
├── name: String
├── functions: Vec<FunctionController>
│   ├── name: String
│   ├── description: String
│   └── parameters: String
└── credential_controllers: Vec<ProviderCredentialController>
    ├── NoAuth
    ├── ApiKey
    ├── Oauth2 { static_credential_configuration: ... }
    └── Oauth2JwtBearerAssertionFlow { ... }
```

All fields are validated during conversion from proto types.
