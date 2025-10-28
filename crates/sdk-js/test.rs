#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use napi_derive::napi;
use sdk_core::{
    start_grpc_server, FunctionController as CoreFunctionController,
    InvokeFunctionRequest, InvokeFunctionResponse,
    ProviderController as CoreProviderController,
    ProviderCredentialController as CoreProviderCredentialController,
    Oauth2AuthorizationCodeFlowStaticCredentialConfiguration, Metadata,
};
use std::path::PathBuf;
use std::sync::Arc;
pub struct JsInvocationRequest {
    pub provider_controller_type_id: String,
    pub function_controller_type_id: String,
    pub credential_controller_type_id: String,
    pub credentials: String,
    pub parameters: String,
}
#[automatically_derived]
impl napi::bindgen_prelude::TypeName for JsInvocationRequest {
    fn type_name() -> &'static str {
        "JsInvocationRequest"
    }
    fn value_type() -> napi::ValueType {
        napi::ValueType::Object
    }
}
#[automatically_derived]
impl napi::bindgen_prelude::ToNapiValue for JsInvocationRequest {
    unsafe fn to_napi_value(
        env: napi::bindgen_prelude::sys::napi_env,
        val: JsInvocationRequest,
    ) -> napi::bindgen_prelude::Result<napi::bindgen_prelude::sys::napi_value> {
        #[allow(unused_variables)]
        let env_wrapper = napi::bindgen_prelude::Env::from(env);
        #[allow(unused_mut)]
        let mut obj = napi::bindgen_prelude::Object::new(&env_wrapper)?;
        let Self {
            provider_controller_type_id: provider_controller_type_id_,
            function_controller_type_id: function_controller_type_id_,
            credential_controller_type_id: credential_controller_type_id_,
            credentials: credentials_,
            parameters: parameters_,
        } = val;
        obj.set("providerControllerTypeId", provider_controller_type_id_)?;
        obj.set("functionControllerTypeId", function_controller_type_id_)?;
        obj.set("credentialControllerTypeId", credential_controller_type_id_)?;
        obj.set("credentials", credentials_)?;
        obj.set("parameters", parameters_)?;
        napi::bindgen_prelude::Object::to_napi_value(env, obj)
    }
}
#[automatically_derived]
impl napi::bindgen_prelude::FromNapiValue for JsInvocationRequest {
    unsafe fn from_napi_value(
        env: napi::bindgen_prelude::sys::napi_env,
        napi_val: napi::bindgen_prelude::sys::napi_value,
    ) -> napi::bindgen_prelude::Result<JsInvocationRequest> {
        #[allow(unused_variables)]
        let env_wrapper = napi::bindgen_prelude::Env::from(env);
        #[allow(unused_mut)]
        let mut obj = napi::bindgen_prelude::Object::from_napi_value(env, napi_val)?;
        let provider_controller_type_id_: String = obj
            .get("providerControllerTypeId")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsInvocationRequest",
                            "providerControllerTypeId",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!("Missing field `{0}`", "providerControllerTypeId"),
                    )
                }),
            ))?;
        let function_controller_type_id_: String = obj
            .get("functionControllerTypeId")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsInvocationRequest",
                            "functionControllerTypeId",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!("Missing field `{0}`", "functionControllerTypeId"),
                    )
                }),
            ))?;
        let credential_controller_type_id_: String = obj
            .get("credentialControllerTypeId")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsInvocationRequest",
                            "credentialControllerTypeId",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!("Missing field `{0}`", "credentialControllerTypeId"),
                    )
                }),
            ))?;
        let credentials_: String = obj
            .get("credentials")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsInvocationRequest",
                            "credentials",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!("Missing field `{0}`", "credentials"),
                    )
                }),
            ))?;
        let parameters_: String = obj
            .get("parameters")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsInvocationRequest",
                            "parameters",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!("Missing field `{0}`", "parameters"),
                    )
                }),
            ))?;
        let val = Self {
            provider_controller_type_id: provider_controller_type_id_,
            function_controller_type_id: function_controller_type_id_,
            credential_controller_type_id: credential_controller_type_id_,
            credentials: credentials_,
            parameters: parameters_,
        };
        Ok(val)
    }
}
#[automatically_derived]
impl napi::bindgen_prelude::ValidateNapiValue for JsInvocationRequest {}
pub struct JsInvocationResponse {
    pub success: bool,
    pub data: Option<String>,
    pub error: Option<String>,
}
#[automatically_derived]
impl napi::bindgen_prelude::TypeName for JsInvocationResponse {
    fn type_name() -> &'static str {
        "JsInvocationResponse"
    }
    fn value_type() -> napi::ValueType {
        napi::ValueType::Object
    }
}
#[automatically_derived]
impl napi::bindgen_prelude::ToNapiValue for JsInvocationResponse {
    unsafe fn to_napi_value(
        env: napi::bindgen_prelude::sys::napi_env,
        val: JsInvocationResponse,
    ) -> napi::bindgen_prelude::Result<napi::bindgen_prelude::sys::napi_value> {
        #[allow(unused_variables)]
        let env_wrapper = napi::bindgen_prelude::Env::from(env);
        #[allow(unused_mut)]
        let mut obj = napi::bindgen_prelude::Object::new(&env_wrapper)?;
        let Self { success: success_, data: data_, error: error_ } = val;
        obj.set("success", success_)?;
        if data_.is_some() {
            obj.set("data", data_)?;
        }
        if error_.is_some() {
            obj.set("error", error_)?;
        }
        napi::bindgen_prelude::Object::to_napi_value(env, obj)
    }
}
#[automatically_derived]
impl napi::bindgen_prelude::FromNapiValue for JsInvocationResponse {
    unsafe fn from_napi_value(
        env: napi::bindgen_prelude::sys::napi_env,
        napi_val: napi::bindgen_prelude::sys::napi_value,
    ) -> napi::bindgen_prelude::Result<JsInvocationResponse> {
        #[allow(unused_variables)]
        let env_wrapper = napi::bindgen_prelude::Env::from(env);
        #[allow(unused_mut)]
        let mut obj = napi::bindgen_prelude::Object::from_napi_value(env, napi_val)?;
        let success_: bool = obj
            .get("success")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsInvocationResponse",
                            "success",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("Missing field `{0}`", "success"))
                }),
            ))?;
        let data_: Option<String> = obj
            .get("data")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsInvocationResponse",
                            "data",
                        ),
                    )
                });
                err
            })?;
        let error_: Option<String> = obj
            .get("error")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsInvocationResponse",
                            "error",
                        ),
                    )
                });
                err
            })?;
        let val = Self {
            success: success_,
            data: data_,
            error: error_,
        };
        Ok(val)
    }
}
#[automatically_derived]
impl napi::bindgen_prelude::ValidateNapiValue for JsInvocationResponse {}
pub struct JsMetadata {
    pub key: String,
    pub value: String,
}
#[automatically_derived]
impl napi::bindgen_prelude::TypeName for JsMetadata {
    fn type_name() -> &'static str {
        "JsMetadata"
    }
    fn value_type() -> napi::ValueType {
        napi::ValueType::Object
    }
}
#[automatically_derived]
impl napi::bindgen_prelude::ToNapiValue for JsMetadata {
    unsafe fn to_napi_value(
        env: napi::bindgen_prelude::sys::napi_env,
        val: JsMetadata,
    ) -> napi::bindgen_prelude::Result<napi::bindgen_prelude::sys::napi_value> {
        #[allow(unused_variables)]
        let env_wrapper = napi::bindgen_prelude::Env::from(env);
        #[allow(unused_mut)]
        let mut obj = napi::bindgen_prelude::Object::new(&env_wrapper)?;
        let Self { key: key_, value: value_ } = val;
        obj.set("key", key_)?;
        obj.set("value", value_)?;
        napi::bindgen_prelude::Object::to_napi_value(env, obj)
    }
}
#[automatically_derived]
impl napi::bindgen_prelude::FromNapiValue for JsMetadata {
    unsafe fn from_napi_value(
        env: napi::bindgen_prelude::sys::napi_env,
        napi_val: napi::bindgen_prelude::sys::napi_value,
    ) -> napi::bindgen_prelude::Result<JsMetadata> {
        #[allow(unused_variables)]
        let env_wrapper = napi::bindgen_prelude::Env::from(env);
        #[allow(unused_mut)]
        let mut obj = napi::bindgen_prelude::Object::from_napi_value(env, napi_val)?;
        let key_: String = obj
            .get("key")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!("{0} on {1}.{2}", err.reason, "JsMetadata", "key"),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("Missing field `{0}`", "key"))
                }),
            ))?;
        let value_: String = obj
            .get("value")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!("{0} on {1}.{2}", err.reason, "JsMetadata", "value"),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("Missing field `{0}`", "value"))
                }),
            ))?;
        let val = Self { key: key_, value: value_ };
        Ok(val)
    }
}
#[automatically_derived]
impl napi::bindgen_prelude::ValidateNapiValue for JsMetadata {}
pub struct JsOauth2Config {
    pub auth_uri: String,
    pub token_uri: String,
    pub userinfo_uri: String,
    pub jwks_uri: String,
    pub issuer: String,
    pub scopes: Vec<String>,
    pub metadata: Option<Vec<JsMetadata>>,
}
#[automatically_derived]
impl napi::bindgen_prelude::TypeName for JsOauth2Config {
    fn type_name() -> &'static str {
        "JsOauth2Config"
    }
    fn value_type() -> napi::ValueType {
        napi::ValueType::Object
    }
}
#[automatically_derived]
impl napi::bindgen_prelude::ToNapiValue for JsOauth2Config {
    unsafe fn to_napi_value(
        env: napi::bindgen_prelude::sys::napi_env,
        val: JsOauth2Config,
    ) -> napi::bindgen_prelude::Result<napi::bindgen_prelude::sys::napi_value> {
        #[allow(unused_variables)]
        let env_wrapper = napi::bindgen_prelude::Env::from(env);
        #[allow(unused_mut)]
        let mut obj = napi::bindgen_prelude::Object::new(&env_wrapper)?;
        let Self {
            auth_uri: auth_uri_,
            token_uri: token_uri_,
            userinfo_uri: userinfo_uri_,
            jwks_uri: jwks_uri_,
            issuer: issuer_,
            scopes: scopes_,
            metadata: metadata_,
        } = val;
        obj.set("authUri", auth_uri_)?;
        obj.set("tokenUri", token_uri_)?;
        obj.set("userinfoUri", userinfo_uri_)?;
        obj.set("jwksUri", jwks_uri_)?;
        obj.set("issuer", issuer_)?;
        obj.set("scopes", scopes_)?;
        if metadata_.is_some() {
            obj.set("metadata", metadata_)?;
        }
        napi::bindgen_prelude::Object::to_napi_value(env, obj)
    }
}
#[automatically_derived]
impl napi::bindgen_prelude::FromNapiValue for JsOauth2Config {
    unsafe fn from_napi_value(
        env: napi::bindgen_prelude::sys::napi_env,
        napi_val: napi::bindgen_prelude::sys::napi_value,
    ) -> napi::bindgen_prelude::Result<JsOauth2Config> {
        #[allow(unused_variables)]
        let env_wrapper = napi::bindgen_prelude::Env::from(env);
        #[allow(unused_mut)]
        let mut obj = napi::bindgen_prelude::Object::from_napi_value(env, napi_val)?;
        let auth_uri_: String = obj
            .get("authUri")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsOauth2Config",
                            "authUri",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("Missing field `{0}`", "authUri"))
                }),
            ))?;
        let token_uri_: String = obj
            .get("tokenUri")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsOauth2Config",
                            "tokenUri",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("Missing field `{0}`", "tokenUri"))
                }),
            ))?;
        let userinfo_uri_: String = obj
            .get("userinfoUri")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsOauth2Config",
                            "userinfoUri",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!("Missing field `{0}`", "userinfoUri"),
                    )
                }),
            ))?;
        let jwks_uri_: String = obj
            .get("jwksUri")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsOauth2Config",
                            "jwksUri",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("Missing field `{0}`", "jwksUri"))
                }),
            ))?;
        let issuer_: String = obj
            .get("issuer")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsOauth2Config",
                            "issuer",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("Missing field `{0}`", "issuer"))
                }),
            ))?;
        let scopes_: Vec<String> = obj
            .get("scopes")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsOauth2Config",
                            "scopes",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("Missing field `{0}`", "scopes"))
                }),
            ))?;
        let metadata_: Option<Vec<JsMetadata>> = obj
            .get("metadata")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsOauth2Config",
                            "metadata",
                        ),
                    )
                });
                err
            })?;
        let val = Self {
            auth_uri: auth_uri_,
            token_uri: token_uri_,
            userinfo_uri: userinfo_uri_,
            jwks_uri: jwks_uri_,
            issuer: issuer_,
            scopes: scopes_,
            metadata: metadata_,
        };
        Ok(val)
    }
}
#[automatically_derived]
impl napi::bindgen_prelude::ValidateNapiValue for JsOauth2Config {}
pub struct JsOauth2JwtBearerConfig {
    pub client_id: String,
    pub private_key: String,
    pub token_url: String,
}
#[automatically_derived]
impl napi::bindgen_prelude::TypeName for JsOauth2JwtBearerConfig {
    fn type_name() -> &'static str {
        "JsOauth2JwtBearerConfig"
    }
    fn value_type() -> napi::ValueType {
        napi::ValueType::Object
    }
}
#[automatically_derived]
impl napi::bindgen_prelude::ToNapiValue for JsOauth2JwtBearerConfig {
    unsafe fn to_napi_value(
        env: napi::bindgen_prelude::sys::napi_env,
        val: JsOauth2JwtBearerConfig,
    ) -> napi::bindgen_prelude::Result<napi::bindgen_prelude::sys::napi_value> {
        #[allow(unused_variables)]
        let env_wrapper = napi::bindgen_prelude::Env::from(env);
        #[allow(unused_mut)]
        let mut obj = napi::bindgen_prelude::Object::new(&env_wrapper)?;
        let Self {
            client_id: client_id_,
            private_key: private_key_,
            token_url: token_url_,
        } = val;
        obj.set("clientId", client_id_)?;
        obj.set("privateKey", private_key_)?;
        obj.set("tokenUrl", token_url_)?;
        napi::bindgen_prelude::Object::to_napi_value(env, obj)
    }
}
#[automatically_derived]
impl napi::bindgen_prelude::FromNapiValue for JsOauth2JwtBearerConfig {
    unsafe fn from_napi_value(
        env: napi::bindgen_prelude::sys::napi_env,
        napi_val: napi::bindgen_prelude::sys::napi_value,
    ) -> napi::bindgen_prelude::Result<JsOauth2JwtBearerConfig> {
        #[allow(unused_variables)]
        let env_wrapper = napi::bindgen_prelude::Env::from(env);
        #[allow(unused_mut)]
        let mut obj = napi::bindgen_prelude::Object::from_napi_value(env, napi_val)?;
        let client_id_: String = obj
            .get("clientId")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsOauth2JwtBearerConfig",
                            "clientId",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("Missing field `{0}`", "clientId"))
                }),
            ))?;
        let private_key_: String = obj
            .get("privateKey")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsOauth2JwtBearerConfig",
                            "privateKey",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!("Missing field `{0}`", "privateKey"),
                    )
                }),
            ))?;
        let token_url_: String = obj
            .get("tokenUrl")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsOauth2JwtBearerConfig",
                            "tokenUrl",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("Missing field `{0}`", "tokenUrl"))
                }),
            ))?;
        let val = Self {
            client_id: client_id_,
            private_key: private_key_,
            token_url: token_url_,
        };
        Ok(val)
    }
}
#[automatically_derived]
impl napi::bindgen_prelude::ValidateNapiValue for JsOauth2JwtBearerConfig {}
pub struct JsCredentialController {
    pub credential_type: String,
    pub oauth2_config: Option<JsOauth2Config>,
    pub oauth2_jwt_config: Option<JsOauth2JwtBearerConfig>,
}
#[automatically_derived]
impl napi::bindgen_prelude::TypeName for JsCredentialController {
    fn type_name() -> &'static str {
        "JsCredentialController"
    }
    fn value_type() -> napi::ValueType {
        napi::ValueType::Object
    }
}
#[automatically_derived]
impl napi::bindgen_prelude::ToNapiValue for JsCredentialController {
    unsafe fn to_napi_value(
        env: napi::bindgen_prelude::sys::napi_env,
        val: JsCredentialController,
    ) -> napi::bindgen_prelude::Result<napi::bindgen_prelude::sys::napi_value> {
        #[allow(unused_variables)]
        let env_wrapper = napi::bindgen_prelude::Env::from(env);
        #[allow(unused_mut)]
        let mut obj = napi::bindgen_prelude::Object::new(&env_wrapper)?;
        let Self {
            credential_type: credential_type_,
            oauth2_config: oauth2_config_,
            oauth2_jwt_config: oauth2_jwt_config_,
        } = val;
        obj.set("credentialType", credential_type_)?;
        if oauth2_config_.is_some() {
            obj.set("oauth2Config", oauth2_config_)?;
        }
        if oauth2_jwt_config_.is_some() {
            obj.set("oauth2JwtConfig", oauth2_jwt_config_)?;
        }
        napi::bindgen_prelude::Object::to_napi_value(env, obj)
    }
}
#[automatically_derived]
impl napi::bindgen_prelude::FromNapiValue for JsCredentialController {
    unsafe fn from_napi_value(
        env: napi::bindgen_prelude::sys::napi_env,
        napi_val: napi::bindgen_prelude::sys::napi_value,
    ) -> napi::bindgen_prelude::Result<JsCredentialController> {
        #[allow(unused_variables)]
        let env_wrapper = napi::bindgen_prelude::Env::from(env);
        #[allow(unused_mut)]
        let mut obj = napi::bindgen_prelude::Object::from_napi_value(env, napi_val)?;
        let credential_type_: String = obj
            .get("credentialType")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsCredentialController",
                            "credentialType",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!("Missing field `{0}`", "credentialType"),
                    )
                }),
            ))?;
        let oauth2_config_: Option<JsOauth2Config> = obj
            .get("oauth2Config")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsCredentialController",
                            "oauth2Config",
                        ),
                    )
                });
                err
            })?;
        let oauth2_jwt_config_: Option<JsOauth2JwtBearerConfig> = obj
            .get("oauth2JwtConfig")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsCredentialController",
                            "oauth2JwtConfig",
                        ),
                    )
                });
                err
            })?;
        let val = Self {
            credential_type: credential_type_,
            oauth2_config: oauth2_config_,
            oauth2_jwt_config: oauth2_jwt_config_,
        };
        Ok(val)
    }
}
#[automatically_derived]
impl napi::bindgen_prelude::ValidateNapiValue for JsCredentialController {}
pub type InvokeTsFn = ThreadsafeFunction<JsInvocationRequest, JsInvocationResponse>;
pub struct JsFunctionController<'scope> {
    pub name: String,
    pub description: String,
    pub parameters: String,
    pub output: String,
    pub invoke: Function<'scope, JsInvocationRequest, JsInvocationResponse>,
}
#[automatically_derived]
impl<'_javascript_function_scope> napi::bindgen_prelude::TypeName
for JsFunctionController<'_javascript_function_scope> {
    fn type_name() -> &'static str {
        "JsFunctionController"
    }
    fn value_type() -> napi::ValueType {
        napi::ValueType::Object
    }
}
#[automatically_derived]
impl<'_javascript_function_scope> napi::bindgen_prelude::ToNapiValue
for JsFunctionController<'_javascript_function_scope> {
    unsafe fn to_napi_value(
        env: napi::bindgen_prelude::sys::napi_env,
        val: JsFunctionController<'_javascript_function_scope>,
    ) -> napi::bindgen_prelude::Result<napi::bindgen_prelude::sys::napi_value> {
        #[allow(unused_variables)]
        let env_wrapper = napi::bindgen_prelude::Env::from(env);
        #[allow(unused_mut)]
        let mut obj = napi::bindgen_prelude::Object::new(&env_wrapper)?;
        let Self {
            name: name_,
            description: description_,
            parameters: parameters_,
            output: output_,
            invoke: invoke_,
        } = val;
        obj.set("name", name_)?;
        obj.set("description", description_)?;
        obj.set("parameters", parameters_)?;
        obj.set("output", output_)?;
        obj.set("invoke", invoke_)?;
        napi::bindgen_prelude::Object::to_napi_value(env, obj)
    }
}
#[automatically_derived]
impl<'_javascript_function_scope> napi::bindgen_prelude::FromNapiValue
for JsFunctionController<'_javascript_function_scope> {
    unsafe fn from_napi_value(
        env: napi::bindgen_prelude::sys::napi_env,
        napi_val: napi::bindgen_prelude::sys::napi_value,
    ) -> napi::bindgen_prelude::Result<
        JsFunctionController<'_javascript_function_scope>,
    > {
        #[allow(unused_variables)]
        let env_wrapper = napi::bindgen_prelude::Env::from(env);
        #[allow(unused_mut)]
        let mut obj = napi::bindgen_prelude::Object::from_napi_value(env, napi_val)?;
        let name_: String = obj
            .get("name")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsFunctionController",
                            "name",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("Missing field `{0}`", "name"))
                }),
            ))?;
        let description_: String = obj
            .get("description")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsFunctionController",
                            "description",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!("Missing field `{0}`", "description"),
                    )
                }),
            ))?;
        let parameters_: String = obj
            .get("parameters")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsFunctionController",
                            "parameters",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!("Missing field `{0}`", "parameters"),
                    )
                }),
            ))?;
        let output_: String = obj
            .get("output")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsFunctionController",
                            "output",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("Missing field `{0}`", "output"))
                }),
            ))?;
        let invoke_: Function<'_, JsInvocationRequest, JsInvocationResponse> = obj
            .get("invoke")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsFunctionController",
                            "invoke",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("Missing field `{0}`", "invoke"))
                }),
            ))?;
        let val = Self {
            name: name_,
            description: description_,
            parameters: parameters_,
            output: output_,
            invoke: invoke_,
        };
        Ok(val)
    }
}
#[automatically_derived]
impl<'_javascript_function_scope> napi::bindgen_prelude::ValidateNapiValue
for JsFunctionController<'_javascript_function_scope> {}
pub struct JsProviderController<'scope> {
    pub type_id: String,
    pub name: String,
    pub documentation: String,
    pub categories: Vec<String>,
    pub functions: Vec<JsFunctionController<'scope>>,
    pub credential_controllers: Vec<JsCredentialController>,
}
#[automatically_derived]
impl<'_javascript_function_scope> napi::bindgen_prelude::TypeName
for JsProviderController<'_javascript_function_scope> {
    fn type_name() -> &'static str {
        "JsProviderController"
    }
    fn value_type() -> napi::ValueType {
        napi::ValueType::Object
    }
}
#[automatically_derived]
impl<'_javascript_function_scope> napi::bindgen_prelude::ToNapiValue
for JsProviderController<'_javascript_function_scope> {
    unsafe fn to_napi_value(
        env: napi::bindgen_prelude::sys::napi_env,
        val: JsProviderController<'_javascript_function_scope>,
    ) -> napi::bindgen_prelude::Result<napi::bindgen_prelude::sys::napi_value> {
        #[allow(unused_variables)]
        let env_wrapper = napi::bindgen_prelude::Env::from(env);
        #[allow(unused_mut)]
        let mut obj = napi::bindgen_prelude::Object::new(&env_wrapper)?;
        let Self {
            type_id: type_id_,
            name: name_,
            documentation: documentation_,
            categories: categories_,
            functions: functions_,
            credential_controllers: credential_controllers_,
        } = val;
        obj.set("typeId", type_id_)?;
        obj.set("name", name_)?;
        obj.set("documentation", documentation_)?;
        obj.set("categories", categories_)?;
        obj.set("functions", functions_)?;
        obj.set("credentialControllers", credential_controllers_)?;
        napi::bindgen_prelude::Object::to_napi_value(env, obj)
    }
}
#[automatically_derived]
impl<'_javascript_function_scope> napi::bindgen_prelude::FromNapiValue
for JsProviderController<'_javascript_function_scope> {
    unsafe fn from_napi_value(
        env: napi::bindgen_prelude::sys::napi_env,
        napi_val: napi::bindgen_prelude::sys::napi_value,
    ) -> napi::bindgen_prelude::Result<
        JsProviderController<'_javascript_function_scope>,
    > {
        #[allow(unused_variables)]
        let env_wrapper = napi::bindgen_prelude::Env::from(env);
        #[allow(unused_mut)]
        let mut obj = napi::bindgen_prelude::Object::from_napi_value(env, napi_val)?;
        let type_id_: String = obj
            .get("typeId")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsProviderController",
                            "typeId",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("Missing field `{0}`", "typeId"))
                }),
            ))?;
        let name_: String = obj
            .get("name")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsProviderController",
                            "name",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("Missing field `{0}`", "name"))
                }),
            ))?;
        let documentation_: String = obj
            .get("documentation")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsProviderController",
                            "documentation",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!("Missing field `{0}`", "documentation"),
                    )
                }),
            ))?;
        let categories_: Vec<String> = obj
            .get("categories")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsProviderController",
                            "categories",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!("Missing field `{0}`", "categories"),
                    )
                }),
            ))?;
        let functions_: Vec<JsFunctionController<'_>> = obj
            .get("functions")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsProviderController",
                            "functions",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!("Missing field `{0}`", "functions"),
                    )
                }),
            ))?;
        let credential_controllers_: Vec<JsCredentialController> = obj
            .get("credentialControllers")
            .map_err(|mut err| {
                err.reason = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0} on {1}.{2}",
                            err.reason,
                            "JsProviderController",
                            "credentialControllers",
                        ),
                    )
                });
                err
            })?
            .ok_or_else(|| napi::bindgen_prelude::Error::new(
                napi::bindgen_prelude::Status::InvalidArg,
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!("Missing field `{0}`", "credentialControllers"),
                    )
                }),
            ))?;
        let val = Self {
            type_id: type_id_,
            name: name_,
            documentation: documentation_,
            categories: categories_,
            functions: functions_,
            credential_controllers: credential_controllers_,
        };
        Ok(val)
    }
}
#[automatically_derived]
impl<'_javascript_function_scope> napi::bindgen_prelude::ValidateNapiValue
for JsProviderController<'_javascript_function_scope> {}
/// Start the gRPC server with the given providers over a Unix socket
pub async fn start_sdk_server(
    providers: Vec<JsProviderController<'scope>>,
    socket_path: String,
) -> Result<()> {
    let path = PathBuf::from(socket_path);
    let core_providers: Vec<CoreProviderController> = providers
        .into_iter()
        .map(|js_provider| {
            let functions: Vec<CoreFunctionController> = js_provider
                .functions
                .into_iter()
                .map(|js_func| {
                    let tsfn: Arc<InvokeTsFn> = Arc::new(js_func.invoke);
                    Ok(CoreFunctionController {
                        name: js_func.name,
                        description: js_func.description,
                        parameters: js_func.parameters,
                        output: js_func.output,
                        invoke: Box::new(move |req: InvokeFunctionRequest| {
                            let tsfn = Arc::clone(&tsfn);
                            Box::pin(async move {
                                let js_req = JsInvocationRequest {
                                    provider_controller_type_id: req
                                        .provider_controller_type_id,
                                    function_controller_type_id: req
                                        .function_controller_type_id,
                                    credential_controller_type_id: req
                                        .credential_controller_type_id,
                                    credentials: req.credentials,
                                    parameters: req.parameters,
                                };
                                let result = tsfn.call_async(Ok(js_req)).await;
                                match result {
                                    Ok(js_response) => {
                                        if js_response.success {
                                            Ok(InvokeFunctionResponse {
                                                result: Ok(js_response.data.unwrap_or_default()),
                                            })
                                        } else {
                                            Ok(InvokeFunctionResponse {
                                                result: Err(
                                                    js_response
                                                        .error
                                                        .unwrap_or_else(|| "Unknown error".to_string()),
                                                ),
                                            })
                                        }
                                    }
                                    Err(e) => {
                                        Ok(InvokeFunctionResponse {
                                            result: Err(
                                                ::alloc::__export::must_use({
                                                    ::alloc::fmt::format(
                                                        format_args!("JavaScript function error: {0}", e),
                                                    )
                                                }),
                                            ),
                                        })
                                    }
                                }
                            })
                        }),
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let credential_controllers: Vec<CoreProviderCredentialController> = js_provider
                .credential_controllers
                .into_iter()
                .map(|js_cred| {
                    match js_cred.credential_type.as_str() {
                        "no_auth" => Ok(CoreProviderCredentialController::NoAuth),
                        "api_key" => Ok(CoreProviderCredentialController::ApiKey),
                        "oauth2" => {
                            let config = js_cred
                                .oauth2_config
                                .ok_or_else(|| {
                                    Error::from_reason(
                                        "oauth2_config is required for oauth2 credential type",
                                    )
                                })?;
                            let metadata = config
                                .metadata
                                .map(|m| {
                                    m.into_iter()
                                        .map(|js_meta| Metadata {
                                            key: js_meta.key,
                                            value: js_meta.value,
                                        })
                                        .collect()
                                });
                            Ok(CoreProviderCredentialController::Oauth2 {
                                static_credential_configuration: Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
                                    auth_uri: config.auth_uri,
                                    token_uri: config.token_uri,
                                    userinfo_uri: config.userinfo_uri,
                                    jwks_uri: config.jwks_uri,
                                    issuer: config.issuer,
                                    scopes: config.scopes,
                                    metadata,
                                },
                            })
                        }
                        "oauth2_jwt_bearer" => {
                            let config = js_cred
                                .oauth2_jwt_config
                                .ok_or_else(|| {
                                    Error::from_reason(
                                        "oauth2_jwt_config is required for oauth2_jwt_bearer credential type",
                                    )
                                })?;
                            Ok(CoreProviderCredentialController::Oauth2JwtBearerAssertionFlow {
                                client_id: config.client_id,
                                private_key: config.private_key,
                                token_url: config.token_url,
                            })
                        }
                        _ => {
                            Err(
                                Error::from_reason(
                                    ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!(
                                                "Unknown credential type: {0}",
                                                js_cred.credential_type,
                                            ),
                                        )
                                    }),
                                ),
                            )
                        }
                    }
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(CoreProviderController {
                type_id: js_provider.type_id,
                name: js_provider.name,
                documentation: js_provider.documentation,
                categories: js_provider.categories,
                functions,
                credential_controllers,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    start_grpc_server(core_providers, path)
        .await
        .map_err(|e| Error::from_reason(
            ::alloc::__export::must_use({
                ::alloc::fmt::format(format_args!("Failed to start server: {0}", e))
            }),
        ))?;
    Ok(())
}
/// Start the gRPC server with the given providers over a Unix socket
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(clippy::all)]
extern "C" fn start_sdk_server_c_callback(
    env: napi::bindgen_prelude::sys::napi_env,
    cb: napi::bindgen_prelude::sys::napi_callback_info,
) -> napi::bindgen_prelude::sys::napi_value {
    unsafe {
        napi::bindgen_prelude::CallbackInfo::<2usize>::new(env, cb, None, false)
            .and_then(|#[allow(unused_mut)] mut cb| {
                let __wrapped_env = napi::bindgen_prelude::Env::from(env);
                struct NapiRefContainer([napi::sys::napi_ref; 0usize]);
                impl NapiRefContainer {
                    fn drop(self, env: napi::sys::napi_env) {
                        for r in self.0.into_iter() {
                            match (
                                &unsafe { napi::sys::napi_reference_unref(env, r, &mut 0) },
                                &napi::sys::Status::napi_ok,
                            ) {
                                (left_val, right_val) => {
                                    if !(*left_val == *right_val) {
                                        let kind = ::core::panicking::AssertKind::Eq;
                                        ::core::panicking::assert_failed(
                                            kind,
                                            &*left_val,
                                            &*right_val,
                                            ::core::option::Option::Some(
                                                format_args!("failed to delete napi ref"),
                                            ),
                                        );
                                    }
                                }
                            };
                            match (
                                &unsafe { napi::sys::napi_delete_reference(env, r) },
                                &napi::sys::Status::napi_ok,
                            ) {
                                (left_val, right_val) => {
                                    if !(*left_val == *right_val) {
                                        let kind = ::core::panicking::AssertKind::Eq;
                                        ::core::panicking::assert_failed(
                                            kind,
                                            &*left_val,
                                            &*right_val,
                                            ::core::option::Option::Some(
                                                format_args!("failed to delete napi ref"),
                                            ),
                                        );
                                    }
                                }
                            };
                        }
                    }
                }
                unsafe impl Send for NapiRefContainer {}
                unsafe impl Sync for NapiRefContainer {}
                let _make_ref = |
                    a: ::std::ptr::NonNull<napi::bindgen_prelude::sys::napi_value__>|
                {
                    let mut node_ref = ::std::mem::MaybeUninit::uninit();
                    {
                        let c = unsafe {
                            napi::bindgen_prelude::sys::napi_create_reference(
                                env,
                                a.as_ptr(),
                                1,
                                node_ref.as_mut_ptr(),
                            )
                        };
                        match c {
                            ::napi::sys::Status::napi_ok => Ok(()),
                            _ => {
                                Err(
                                    ::napi::Error::new(
                                        ::napi::Status::from(c),
                                        ::alloc::__export::must_use({
                                            ::alloc::fmt::format(
                                                format_args!("failed to create napi ref"),
                                            )
                                        }),
                                    ),
                                )
                            }
                        }
                    }?;
                    Ok::<
                        napi::sys::napi_ref,
                        napi::Error,
                    >(unsafe { node_ref.assume_init() })
                };
                let mut _args_array = [::std::ptr::null_mut::<
                    napi::bindgen_prelude::sys::napi_ref__,
                >(); 0usize];
                let mut _arg_write_index = 0;
                {
                    for a in &_args_array {
                        if !!a.is_null() {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!("failed to initialize napi ref"),
                                );
                            }
                        }
                    }
                }
                let _args_ref = NapiRefContainer(_args_array);
                let arg0 = {
                    <Vec<
                        JsProviderController<'_>,
                    > as napi::bindgen_prelude::FromNapiValue>::from_napi_value(
                        env,
                        cb.get_arg(0usize),
                    )?
                };
                let arg1 = {
                    <String as napi::bindgen_prelude::FromNapiValue>::from_napi_value(
                        env,
                        cb.get_arg(1usize),
                    )?
                };
                napi::bindgen_prelude::execute_tokio_future(
                    env,
                    async move { start_sdk_server(arg0, arg1).await },
                    move |env, _ret| {
                        _args_ref.drop(env);
                        <() as napi::bindgen_prelude::ToNapiValue>::to_napi_value(
                            env,
                            _ret,
                        )
                    },
                )
            })
            .unwrap_or_else(|e| {
                napi::bindgen_prelude::JsError::from(e).throw_into(env);
                std::ptr::null_mut::<napi::bindgen_prelude::sys::napi_value__>()
            })
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
#[allow(clippy::all)]
unsafe fn _napi_rs_internal_register_start_sdk_server(
    env: napi::bindgen_prelude::sys::napi_env,
) -> napi::bindgen_prelude::Result<napi::bindgen_prelude::sys::napi_value> {
    let mut fn_ptr = std::ptr::null_mut();
    {
        let c = napi::bindgen_prelude::sys::napi_create_function(
            env,
            "startSdkServer\0".as_ptr().cast(),
            14usize as isize,
            Some(start_sdk_server_c_callback),
            std::ptr::null_mut(),
            &mut fn_ptr,
        );
        match c {
            ::napi::sys::Status::napi_ok => Ok(()),
            _ => {
                Err(
                    ::napi::Error::new(
                        ::napi::Status::from(c),
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!(
                                    "Failed to register function `{0}`",
                                    "start_sdk_server",
                                ),
                            )
                        }),
                    ),
                )
            }
        }
    }?;
    Ok(fn_ptr)
}
#[doc(hidden)]
#[allow(clippy::all)]
#[allow(non_snake_case)]
#[allow(unused)]
fn __napi_register__start_sdk_server_8() {
    #[allow(unsafe_code)]
    {
        #[link_section = ".init_array"]
        #[used]
        #[allow(non_upper_case_globals, non_snake_case)]
        #[doc(hidden)]
        static f: extern "C" fn() -> ::ctor::__support::CtorRetType = {
            #[link_section = ".text.startup"]
            #[allow(non_snake_case)]
            extern "C" fn f() -> ::ctor::__support::CtorRetType {
                unsafe {
                    __napi_register__start_sdk_server_8();
                };
                core::default::Default::default()
            }
            f
        };
    }
    {
        napi::bindgen_prelude::register_module_export(
            None,
            "startSdkServer\0",
            _napi_rs_internal_register_start_sdk_server,
        );
    }
}
