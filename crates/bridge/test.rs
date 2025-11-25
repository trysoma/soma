#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2024::*;
#[macro_use]
extern crate std;
mod router {}
mod logic {
    use enum_dispatch::enum_dispatch;
    use serde::{Deserialize, Serialize};
    use chrono::{DateTime, Utc};
    use shared::{error::CommonError, primitives::{WrappedChronoDateTime, WrappedUuidV4}};
    use reqwest::Request;
    #[serde(transparent)]
    pub struct Metadata(pub serde_json::Map<String, serde_json::Value>);
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for Metadata {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                _serde::Serialize::serialize(&self.0, __serializer)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for Metadata {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                _serde::__private228::Result::map(
                    _serde::Deserialize::deserialize(__deserializer),
                    |__transparent| Metadata { 0: __transparent },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for Metadata {
        #[inline]
        fn clone(&self) -> Metadata {
            Metadata(::core::clone::Clone::clone(&self.0))
        }
    }
    impl Metadata {
        pub fn new() -> Self {
            Self(serde_json::Map::new())
        }
    }
    pub struct DatabaseCredential<T> {
        pub inner: T,
        pub metadata: Metadata,
        pub id: WrappedUuidV4,
        pub created_at: WrappedChronoDateTime,
        pub updated_at: WrappedChronoDateTime,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<T> _serde::Serialize for DatabaseCredential<T>
        where
            T: _serde::Serialize,
        {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "DatabaseCredential",
                    false as usize + 1 + 1 + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "inner",
                    &self.inner,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "metadata",
                    &self.metadata,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "id",
                    &self.id,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "created_at",
                    &self.created_at,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "updated_at",
                    &self.updated_at,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de, T> _serde::Deserialize<'de> for DatabaseCredential<T>
        where
            T: _serde::Deserialize<'de>,
        {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            1u64 => _serde::__private228::Ok(__Field::__field1),
                            2u64 => _serde::__private228::Ok(__Field::__field2),
                            3u64 => _serde::__private228::Ok(__Field::__field3),
                            4u64 => _serde::__private228::Ok(__Field::__field4),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "inner" => _serde::__private228::Ok(__Field::__field0),
                            "metadata" => _serde::__private228::Ok(__Field::__field1),
                            "id" => _serde::__private228::Ok(__Field::__field2),
                            "created_at" => _serde::__private228::Ok(__Field::__field3),
                            "updated_at" => _serde::__private228::Ok(__Field::__field4),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"inner" => _serde::__private228::Ok(__Field::__field0),
                            b"metadata" => _serde::__private228::Ok(__Field::__field1),
                            b"id" => _serde::__private228::Ok(__Field::__field2),
                            b"created_at" => _serde::__private228::Ok(__Field::__field3),
                            b"updated_at" => _serde::__private228::Ok(__Field::__field4),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de, T>
                where
                    T: _serde::Deserialize<'de>,
                {
                    marker: _serde::__private228::PhantomData<DatabaseCredential<T>>,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de, T> _serde::de::Visitor<'de> for __Visitor<'de, T>
                where
                    T: _serde::Deserialize<'de>,
                {
                    type Value = DatabaseCredential<T>;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct DatabaseCredential",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            T,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct DatabaseCredential with 5 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            Metadata,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct DatabaseCredential with 5 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            WrappedUuidV4,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct DatabaseCredential with 5 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            WrappedChronoDateTime,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct DatabaseCredential with 5 elements",
                                    ),
                                );
                            }
                        };
                        let __field4 = match _serde::de::SeqAccess::next_element::<
                            WrappedChronoDateTime,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        4usize,
                                        &"struct DatabaseCredential with 5 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(DatabaseCredential {
                            inner: __field0,
                            metadata: __field1,
                            id: __field2,
                            created_at: __field3,
                            updated_at: __field4,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<T> = _serde::__private228::None;
                        let mut __field1: _serde::__private228::Option<Metadata> = _serde::__private228::None;
                        let mut __field2: _serde::__private228::Option<WrappedUuidV4> = _serde::__private228::None;
                        let mut __field3: _serde::__private228::Option<
                            WrappedChronoDateTime,
                        > = _serde::__private228::None;
                        let mut __field4: _serde::__private228::Option<
                            WrappedChronoDateTime,
                        > = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("inner"),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<T>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private228::Option::is_some(&__field1) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "metadata",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<Metadata>(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private228::Option::is_some(&__field2) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("id"),
                                        );
                                    }
                                    __field2 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            WrappedUuidV4,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private228::Option::is_some(&__field3) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "created_at",
                                            ),
                                        );
                                    }
                                    __field3 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            WrappedChronoDateTime,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private228::Option::is_some(&__field4) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "updated_at",
                                            ),
                                        );
                                    }
                                    __field4 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            WrappedChronoDateTime,
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("inner")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private228::Some(__field1) => __field1,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("metadata")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private228::Some(__field2) => __field2,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("id")?
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private228::Some(__field3) => __field3,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("created_at")?
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private228::Some(__field4) => __field4,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("updated_at")?
                            }
                        };
                        _serde::__private228::Ok(DatabaseCredential {
                            inner: __field0,
                            metadata: __field1,
                            id: __field2,
                            created_at: __field3,
                            updated_at: __field4,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "inner",
                    "metadata",
                    "id",
                    "created_at",
                    "updated_at",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "DatabaseCredential",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<
                            DatabaseCredential<T>,
                        >,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl<T: ::core::clone::Clone> ::core::clone::Clone for DatabaseCredential<T> {
        #[inline]
        fn clone(&self) -> DatabaseCredential<T> {
            DatabaseCredential {
                inner: ::core::clone::Clone::clone(&self.inner),
                metadata: ::core::clone::Clone::clone(&self.metadata),
                id: ::core::clone::Clone::clone(&self.id),
                created_at: ::core::clone::Clone::clone(&self.created_at),
                updated_at: ::core::clone::Clone::clone(&self.updated_at),
            }
        }
    }
    #[serde(tag = "type")]
    pub enum StaticCredentialConfigurationVariant {
        NoAuth(NoAuthStaticCredentialConfiguration),
        Oauth2AuthorizationCodeFlow(
            Oauth2AuthorizationCodeFlowStaticCredentialConfiguration,
        ),
        Oauth2JwtBearerAssertionFlow(
            Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration,
        ),
        Custom(CustomStaticCredentialConfiguration),
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for StaticCredentialConfigurationVariant {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    StaticCredentialConfigurationVariant::NoAuth(ref __field0) => {
                        _serde::__private228::ser::serialize_tagged_newtype(
                            __serializer,
                            "StaticCredentialConfigurationVariant",
                            "NoAuth",
                            "type",
                            "NoAuth",
                            __field0,
                        )
                    }
                    StaticCredentialConfigurationVariant::Oauth2AuthorizationCodeFlow(
                        ref __field0,
                    ) => {
                        _serde::__private228::ser::serialize_tagged_newtype(
                            __serializer,
                            "StaticCredentialConfigurationVariant",
                            "Oauth2AuthorizationCodeFlow",
                            "type",
                            "Oauth2AuthorizationCodeFlow",
                            __field0,
                        )
                    }
                    StaticCredentialConfigurationVariant::Oauth2JwtBearerAssertionFlow(
                        ref __field0,
                    ) => {
                        _serde::__private228::ser::serialize_tagged_newtype(
                            __serializer,
                            "StaticCredentialConfigurationVariant",
                            "Oauth2JwtBearerAssertionFlow",
                            "type",
                            "Oauth2JwtBearerAssertionFlow",
                            __field0,
                        )
                    }
                    StaticCredentialConfigurationVariant::Custom(ref __field0) => {
                        _serde::__private228::ser::serialize_tagged_newtype(
                            __serializer,
                            "StaticCredentialConfigurationVariant",
                            "Custom",
                            "type",
                            "Custom",
                            __field0,
                        )
                    }
                }
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for StaticCredentialConfigurationVariant {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "variant identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            1u64 => _serde::__private228::Ok(__Field::__field1),
                            2u64 => _serde::__private228::Ok(__Field::__field2),
                            3u64 => _serde::__private228::Ok(__Field::__field3),
                            _ => {
                                _serde::__private228::Err(
                                    _serde::de::Error::invalid_value(
                                        _serde::de::Unexpected::Unsigned(__value),
                                        &"variant index 0 <= i < 4",
                                    ),
                                )
                            }
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "NoAuth" => _serde::__private228::Ok(__Field::__field0),
                            "Oauth2AuthorizationCodeFlow" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            "Oauth2JwtBearerAssertionFlow" => {
                                _serde::__private228::Ok(__Field::__field2)
                            }
                            "Custom" => _serde::__private228::Ok(__Field::__field3),
                            _ => {
                                _serde::__private228::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"NoAuth" => _serde::__private228::Ok(__Field::__field0),
                            b"Oauth2AuthorizationCodeFlow" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            b"Oauth2JwtBearerAssertionFlow" => {
                                _serde::__private228::Ok(__Field::__field2)
                            }
                            b"Custom" => _serde::__private228::Ok(__Field::__field3),
                            _ => {
                                let __value = &_serde::__private228::from_utf8_lossy(
                                    __value,
                                );
                                _serde::__private228::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                const VARIANTS: &'static [&'static str] = &[
                    "NoAuth",
                    "Oauth2AuthorizationCodeFlow",
                    "Oauth2JwtBearerAssertionFlow",
                    "Custom",
                ];
                let (__tag, __content) = _serde::Deserializer::deserialize_any(
                    __deserializer,
                    _serde::__private228::de::TaggedContentVisitor::<
                        __Field,
                    >::new(
                        "type",
                        "internally tagged enum StaticCredentialConfigurationVariant",
                    ),
                )?;
                let __deserializer = _serde::__private228::de::ContentDeserializer::<
                    __D::Error,
                >::new(__content);
                match __tag {
                    __Field::__field0 => {
                        _serde::__private228::Result::map(
                            <NoAuthStaticCredentialConfiguration as _serde::Deserialize>::deserialize(
                                __deserializer,
                            ),
                            StaticCredentialConfigurationVariant::NoAuth,
                        )
                    }
                    __Field::__field1 => {
                        _serde::__private228::Result::map(
                            <Oauth2AuthorizationCodeFlowStaticCredentialConfiguration as _serde::Deserialize>::deserialize(
                                __deserializer,
                            ),
                            StaticCredentialConfigurationVariant::Oauth2AuthorizationCodeFlow,
                        )
                    }
                    __Field::__field2 => {
                        _serde::__private228::Result::map(
                            <Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration as _serde::Deserialize>::deserialize(
                                __deserializer,
                            ),
                            StaticCredentialConfigurationVariant::Oauth2JwtBearerAssertionFlow,
                        )
                    }
                    __Field::__field3 => {
                        _serde::__private228::Result::map(
                            <CustomStaticCredentialConfiguration as _serde::Deserialize>::deserialize(
                                __deserializer,
                            ),
                            StaticCredentialConfigurationVariant::Custom,
                        )
                    }
                }
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for StaticCredentialConfigurationVariant {
        #[inline]
        fn clone(&self) -> StaticCredentialConfigurationVariant {
            match self {
                StaticCredentialConfigurationVariant::NoAuth(__self_0) => {
                    StaticCredentialConfigurationVariant::NoAuth(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                StaticCredentialConfigurationVariant::Oauth2AuthorizationCodeFlow(
                    __self_0,
                ) => {
                    StaticCredentialConfigurationVariant::Oauth2AuthorizationCodeFlow(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                StaticCredentialConfigurationVariant::Oauth2JwtBearerAssertionFlow(
                    __self_0,
                ) => {
                    StaticCredentialConfigurationVariant::Oauth2JwtBearerAssertionFlow(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                StaticCredentialConfigurationVariant::Custom(__self_0) => {
                    StaticCredentialConfigurationVariant::Custom(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
            }
        }
    }
    #[serde(rename_all = "snake_case")]
    pub enum StaticCredentialConfigurationType {
        NoAuth,
        Oauth2AuthorizationCodeFlow,
        Oauth2JwtBearerAssertionFlow,
        Custom,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for StaticCredentialConfigurationType {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    StaticCredentialConfigurationType::NoAuth => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "StaticCredentialConfigurationType",
                            0u32,
                            "no_auth",
                        )
                    }
                    StaticCredentialConfigurationType::Oauth2AuthorizationCodeFlow => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "StaticCredentialConfigurationType",
                            1u32,
                            "oauth2_authorization_code_flow",
                        )
                    }
                    StaticCredentialConfigurationType::Oauth2JwtBearerAssertionFlow => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "StaticCredentialConfigurationType",
                            2u32,
                            "oauth2_jwt_bearer_assertion_flow",
                        )
                    }
                    StaticCredentialConfigurationType::Custom => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "StaticCredentialConfigurationType",
                            3u32,
                            "custom",
                        )
                    }
                }
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for StaticCredentialConfigurationType {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "variant identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            1u64 => _serde::__private228::Ok(__Field::__field1),
                            2u64 => _serde::__private228::Ok(__Field::__field2),
                            3u64 => _serde::__private228::Ok(__Field::__field3),
                            _ => {
                                _serde::__private228::Err(
                                    _serde::de::Error::invalid_value(
                                        _serde::de::Unexpected::Unsigned(__value),
                                        &"variant index 0 <= i < 4",
                                    ),
                                )
                            }
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "no_auth" => _serde::__private228::Ok(__Field::__field0),
                            "oauth2_authorization_code_flow" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            "oauth2_jwt_bearer_assertion_flow" => {
                                _serde::__private228::Ok(__Field::__field2)
                            }
                            "custom" => _serde::__private228::Ok(__Field::__field3),
                            _ => {
                                _serde::__private228::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"no_auth" => _serde::__private228::Ok(__Field::__field0),
                            b"oauth2_authorization_code_flow" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            b"oauth2_jwt_bearer_assertion_flow" => {
                                _serde::__private228::Ok(__Field::__field2)
                            }
                            b"custom" => _serde::__private228::Ok(__Field::__field3),
                            _ => {
                                let __value = &_serde::__private228::from_utf8_lossy(
                                    __value,
                                );
                                _serde::__private228::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<
                        StaticCredentialConfigurationType,
                    >,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = StaticCredentialConfigurationType;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "enum StaticCredentialConfigurationType",
                        )
                    }
                    fn visit_enum<__A>(
                        self,
                        __data: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::EnumAccess<'de>,
                    {
                        match _serde::de::EnumAccess::variant(__data)? {
                            (__Field::__field0, __variant) => {
                                _serde::de::VariantAccess::unit_variant(__variant)?;
                                _serde::__private228::Ok(
                                    StaticCredentialConfigurationType::NoAuth,
                                )
                            }
                            (__Field::__field1, __variant) => {
                                _serde::de::VariantAccess::unit_variant(__variant)?;
                                _serde::__private228::Ok(
                                    StaticCredentialConfigurationType::Oauth2AuthorizationCodeFlow,
                                )
                            }
                            (__Field::__field2, __variant) => {
                                _serde::de::VariantAccess::unit_variant(__variant)?;
                                _serde::__private228::Ok(
                                    StaticCredentialConfigurationType::Oauth2JwtBearerAssertionFlow,
                                )
                            }
                            (__Field::__field3, __variant) => {
                                _serde::de::VariantAccess::unit_variant(__variant)?;
                                _serde::__private228::Ok(
                                    StaticCredentialConfigurationType::Custom,
                                )
                            }
                        }
                    }
                }
                #[doc(hidden)]
                const VARIANTS: &'static [&'static str] = &[
                    "no_auth",
                    "oauth2_authorization_code_flow",
                    "oauth2_jwt_bearer_assertion_flow",
                    "custom",
                ];
                _serde::Deserializer::deserialize_enum(
                    __deserializer,
                    "StaticCredentialConfigurationType",
                    VARIANTS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<
                            StaticCredentialConfigurationType,
                        >,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for StaticCredentialConfigurationType {
        #[inline]
        fn clone(&self) -> StaticCredentialConfigurationType {
            match self {
                StaticCredentialConfigurationType::NoAuth => {
                    StaticCredentialConfigurationType::NoAuth
                }
                StaticCredentialConfigurationType::Oauth2AuthorizationCodeFlow => {
                    StaticCredentialConfigurationType::Oauth2AuthorizationCodeFlow
                }
                StaticCredentialConfigurationType::Oauth2JwtBearerAssertionFlow => {
                    StaticCredentialConfigurationType::Oauth2JwtBearerAssertionFlow
                }
                StaticCredentialConfigurationType::Custom => {
                    StaticCredentialConfigurationType::Custom
                }
            }
        }
    }
    pub struct StaticCredentialConfiguration {
        pub inner: StaticCredentialConfigurationVariant,
        pub metadata: Metadata,
    }
    pub struct NoAuthStaticCredentialConfiguration {
        pub metadata: Metadata,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for NoAuthStaticCredentialConfiguration {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "NoAuthStaticCredentialConfiguration",
                    false as usize + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "metadata",
                    &self.metadata,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for NoAuthStaticCredentialConfiguration {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "metadata" => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"metadata" => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<
                        NoAuthStaticCredentialConfiguration,
                    >,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = NoAuthStaticCredentialConfiguration;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct NoAuthStaticCredentialConfiguration",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            Metadata,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct NoAuthStaticCredentialConfiguration with 1 element",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(NoAuthStaticCredentialConfiguration {
                            metadata: __field0,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<Metadata> = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "metadata",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<Metadata>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("metadata")?
                            }
                        };
                        _serde::__private228::Ok(NoAuthStaticCredentialConfiguration {
                            metadata: __field0,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &["metadata"];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "NoAuthStaticCredentialConfiguration",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<
                            NoAuthStaticCredentialConfiguration,
                        >,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for NoAuthStaticCredentialConfiguration {
        #[inline]
        fn clone(&self) -> NoAuthStaticCredentialConfiguration {
            NoAuthStaticCredentialConfiguration {
                metadata: ::core::clone::Clone::clone(&self.metadata),
            }
        }
    }
    pub struct Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
        pub auth_uri: String,
        pub token_uri: String,
        pub userinfo_uri: String,
        pub jwks_uri: String,
        pub issuer: String,
        pub scopes: Vec<String>,
        pub metadata: Metadata,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize
        for Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "Oauth2AuthorizationCodeFlowStaticCredentialConfiguration",
                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "auth_uri",
                    &self.auth_uri,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "token_uri",
                    &self.token_uri,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "userinfo_uri",
                    &self.userinfo_uri,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "jwks_uri",
                    &self.jwks_uri,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "issuer",
                    &self.issuer,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "scopes",
                    &self.scopes,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "metadata",
                    &self.metadata,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de>
        for Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __field6,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            1u64 => _serde::__private228::Ok(__Field::__field1),
                            2u64 => _serde::__private228::Ok(__Field::__field2),
                            3u64 => _serde::__private228::Ok(__Field::__field3),
                            4u64 => _serde::__private228::Ok(__Field::__field4),
                            5u64 => _serde::__private228::Ok(__Field::__field5),
                            6u64 => _serde::__private228::Ok(__Field::__field6),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "auth_uri" => _serde::__private228::Ok(__Field::__field0),
                            "token_uri" => _serde::__private228::Ok(__Field::__field1),
                            "userinfo_uri" => _serde::__private228::Ok(__Field::__field2),
                            "jwks_uri" => _serde::__private228::Ok(__Field::__field3),
                            "issuer" => _serde::__private228::Ok(__Field::__field4),
                            "scopes" => _serde::__private228::Ok(__Field::__field5),
                            "metadata" => _serde::__private228::Ok(__Field::__field6),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"auth_uri" => _serde::__private228::Ok(__Field::__field0),
                            b"token_uri" => _serde::__private228::Ok(__Field::__field1),
                            b"userinfo_uri" => {
                                _serde::__private228::Ok(__Field::__field2)
                            }
                            b"jwks_uri" => _serde::__private228::Ok(__Field::__field3),
                            b"issuer" => _serde::__private228::Ok(__Field::__field4),
                            b"scopes" => _serde::__private228::Ok(__Field::__field5),
                            b"metadata" => _serde::__private228::Ok(__Field::__field6),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<
                        Oauth2AuthorizationCodeFlowStaticCredentialConfiguration,
                    >,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = Oauth2AuthorizationCodeFlowStaticCredentialConfiguration;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct Oauth2AuthorizationCodeFlowStaticCredentialConfiguration",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct Oauth2AuthorizationCodeFlowStaticCredentialConfiguration with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct Oauth2AuthorizationCodeFlowStaticCredentialConfiguration with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct Oauth2AuthorizationCodeFlowStaticCredentialConfiguration with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct Oauth2AuthorizationCodeFlowStaticCredentialConfiguration with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field4 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        4usize,
                                        &"struct Oauth2AuthorizationCodeFlowStaticCredentialConfiguration with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field5 = match _serde::de::SeqAccess::next_element::<
                            Vec<String>,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        5usize,
                                        &"struct Oauth2AuthorizationCodeFlowStaticCredentialConfiguration with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field6 = match _serde::de::SeqAccess::next_element::<
                            Metadata,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        6usize,
                                        &"struct Oauth2AuthorizationCodeFlowStaticCredentialConfiguration with 7 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
                            auth_uri: __field0,
                            token_uri: __field1,
                            userinfo_uri: __field2,
                            jwks_uri: __field3,
                            issuer: __field4,
                            scopes: __field5,
                            metadata: __field6,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field1: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field2: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field3: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field4: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field5: _serde::__private228::Option<Vec<String>> = _serde::__private228::None;
                        let mut __field6: _serde::__private228::Option<Metadata> = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "auth_uri",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private228::Option::is_some(&__field1) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "token_uri",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private228::Option::is_some(&__field2) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "userinfo_uri",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private228::Option::is_some(&__field3) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "jwks_uri",
                                            ),
                                        );
                                    }
                                    __field3 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private228::Option::is_some(&__field4) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("issuer"),
                                        );
                                    }
                                    __field4 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private228::Option::is_some(&__field5) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("scopes"),
                                        );
                                    }
                                    __field5 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Vec<String>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field6 => {
                                    if _serde::__private228::Option::is_some(&__field6) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "metadata",
                                            ),
                                        );
                                    }
                                    __field6 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<Metadata>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("auth_uri")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private228::Some(__field1) => __field1,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("token_uri")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private228::Some(__field2) => __field2,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("userinfo_uri")?
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private228::Some(__field3) => __field3,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("jwks_uri")?
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private228::Some(__field4) => __field4,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("issuer")?
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private228::Some(__field5) => __field5,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("scopes")?
                            }
                        };
                        let __field6 = match __field6 {
                            _serde::__private228::Some(__field6) => __field6,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("metadata")?
                            }
                        };
                        _serde::__private228::Ok(Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
                            auth_uri: __field0,
                            token_uri: __field1,
                            userinfo_uri: __field2,
                            jwks_uri: __field3,
                            issuer: __field4,
                            scopes: __field5,
                            metadata: __field6,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "auth_uri",
                    "token_uri",
                    "userinfo_uri",
                    "jwks_uri",
                    "issuer",
                    "scopes",
                    "metadata",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "Oauth2AuthorizationCodeFlowStaticCredentialConfiguration",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<
                            Oauth2AuthorizationCodeFlowStaticCredentialConfiguration,
                        >,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone
    for Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
        #[inline]
        fn clone(&self) -> Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
            Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
                auth_uri: ::core::clone::Clone::clone(&self.auth_uri),
                token_uri: ::core::clone::Clone::clone(&self.token_uri),
                userinfo_uri: ::core::clone::Clone::clone(&self.userinfo_uri),
                jwks_uri: ::core::clone::Clone::clone(&self.jwks_uri),
                issuer: ::core::clone::Clone::clone(&self.issuer),
                scopes: ::core::clone::Clone::clone(&self.scopes),
                metadata: ::core::clone::Clone::clone(&self.metadata),
            }
        }
    }
    pub struct Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
        pub auth_uri: String,
        pub token_uri: String,
        pub userinfo_uri: String,
        pub jwks_uri: String,
        pub issuer: String,
        pub scopes: Vec<String>,
        pub metadata: Metadata,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize
        for Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration",
                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "auth_uri",
                    &self.auth_uri,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "token_uri",
                    &self.token_uri,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "userinfo_uri",
                    &self.userinfo_uri,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "jwks_uri",
                    &self.jwks_uri,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "issuer",
                    &self.issuer,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "scopes",
                    &self.scopes,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "metadata",
                    &self.metadata,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de>
        for Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __field6,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            1u64 => _serde::__private228::Ok(__Field::__field1),
                            2u64 => _serde::__private228::Ok(__Field::__field2),
                            3u64 => _serde::__private228::Ok(__Field::__field3),
                            4u64 => _serde::__private228::Ok(__Field::__field4),
                            5u64 => _serde::__private228::Ok(__Field::__field5),
                            6u64 => _serde::__private228::Ok(__Field::__field6),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "auth_uri" => _serde::__private228::Ok(__Field::__field0),
                            "token_uri" => _serde::__private228::Ok(__Field::__field1),
                            "userinfo_uri" => _serde::__private228::Ok(__Field::__field2),
                            "jwks_uri" => _serde::__private228::Ok(__Field::__field3),
                            "issuer" => _serde::__private228::Ok(__Field::__field4),
                            "scopes" => _serde::__private228::Ok(__Field::__field5),
                            "metadata" => _serde::__private228::Ok(__Field::__field6),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"auth_uri" => _serde::__private228::Ok(__Field::__field0),
                            b"token_uri" => _serde::__private228::Ok(__Field::__field1),
                            b"userinfo_uri" => {
                                _serde::__private228::Ok(__Field::__field2)
                            }
                            b"jwks_uri" => _serde::__private228::Ok(__Field::__field3),
                            b"issuer" => _serde::__private228::Ok(__Field::__field4),
                            b"scopes" => _serde::__private228::Ok(__Field::__field5),
                            b"metadata" => _serde::__private228::Ok(__Field::__field6),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<
                        Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration,
                    >,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field4 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        4usize,
                                        &"struct Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field5 = match _serde::de::SeqAccess::next_element::<
                            Vec<String>,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        5usize,
                                        &"struct Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field6 = match _serde::de::SeqAccess::next_element::<
                            Metadata,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        6usize,
                                        &"struct Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration with 7 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
                            auth_uri: __field0,
                            token_uri: __field1,
                            userinfo_uri: __field2,
                            jwks_uri: __field3,
                            issuer: __field4,
                            scopes: __field5,
                            metadata: __field6,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field1: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field2: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field3: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field4: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field5: _serde::__private228::Option<Vec<String>> = _serde::__private228::None;
                        let mut __field6: _serde::__private228::Option<Metadata> = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "auth_uri",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private228::Option::is_some(&__field1) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "token_uri",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private228::Option::is_some(&__field2) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "userinfo_uri",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private228::Option::is_some(&__field3) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "jwks_uri",
                                            ),
                                        );
                                    }
                                    __field3 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private228::Option::is_some(&__field4) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("issuer"),
                                        );
                                    }
                                    __field4 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private228::Option::is_some(&__field5) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("scopes"),
                                        );
                                    }
                                    __field5 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Vec<String>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field6 => {
                                    if _serde::__private228::Option::is_some(&__field6) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "metadata",
                                            ),
                                        );
                                    }
                                    __field6 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<Metadata>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("auth_uri")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private228::Some(__field1) => __field1,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("token_uri")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private228::Some(__field2) => __field2,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("userinfo_uri")?
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private228::Some(__field3) => __field3,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("jwks_uri")?
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private228::Some(__field4) => __field4,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("issuer")?
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private228::Some(__field5) => __field5,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("scopes")?
                            }
                        };
                        let __field6 = match __field6 {
                            _serde::__private228::Some(__field6) => __field6,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("metadata")?
                            }
                        };
                        _serde::__private228::Ok(Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
                            auth_uri: __field0,
                            token_uri: __field1,
                            userinfo_uri: __field2,
                            jwks_uri: __field3,
                            issuer: __field4,
                            scopes: __field5,
                            metadata: __field6,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "auth_uri",
                    "token_uri",
                    "userinfo_uri",
                    "jwks_uri",
                    "issuer",
                    "scopes",
                    "metadata",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<
                            Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration,
                        >,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone
    for Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
        #[inline]
        fn clone(&self) -> Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
            Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
                auth_uri: ::core::clone::Clone::clone(&self.auth_uri),
                token_uri: ::core::clone::Clone::clone(&self.token_uri),
                userinfo_uri: ::core::clone::Clone::clone(&self.userinfo_uri),
                jwks_uri: ::core::clone::Clone::clone(&self.jwks_uri),
                issuer: ::core::clone::Clone::clone(&self.issuer),
                scopes: ::core::clone::Clone::clone(&self.scopes),
                metadata: ::core::clone::Clone::clone(&self.metadata),
            }
        }
    }
    pub struct CustomStaticCredentialConfiguration {
        pub metadata: Metadata,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for CustomStaticCredentialConfiguration {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "CustomStaticCredentialConfiguration",
                    false as usize + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "metadata",
                    &self.metadata,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for CustomStaticCredentialConfiguration {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "metadata" => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"metadata" => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<
                        CustomStaticCredentialConfiguration,
                    >,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = CustomStaticCredentialConfiguration;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct CustomStaticCredentialConfiguration",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            Metadata,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct CustomStaticCredentialConfiguration with 1 element",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(CustomStaticCredentialConfiguration {
                            metadata: __field0,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<Metadata> = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "metadata",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<Metadata>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("metadata")?
                            }
                        };
                        _serde::__private228::Ok(CustomStaticCredentialConfiguration {
                            metadata: __field0,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &["metadata"];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "CustomStaticCredentialConfiguration",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<
                            CustomStaticCredentialConfiguration,
                        >,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for CustomStaticCredentialConfiguration {
        #[inline]
        fn clone(&self) -> CustomStaticCredentialConfiguration {
            CustomStaticCredentialConfiguration {
                metadata: ::core::clone::Clone::clone(&self.metadata),
            }
        }
    }
    #[serde(tag = "type")]
    pub enum ResourceServerCredentialVariant {
        NoAuth(NoAuthResourceServerCredential),
        Oauth2AuthorizationCodeFlow(Oauth2AuthorizationCodeFlowResourceServerCredential),
        Oauth2JwtBearerAssertionFlow(
            Oauth2JwtBearerAssertionFlowResourceServerCredential,
        ),
        Custom(CustomResourceServerCredential),
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for ResourceServerCredentialVariant {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    ResourceServerCredentialVariant::NoAuth(ref __field0) => {
                        _serde::__private228::ser::serialize_tagged_newtype(
                            __serializer,
                            "ResourceServerCredentialVariant",
                            "NoAuth",
                            "type",
                            "NoAuth",
                            __field0,
                        )
                    }
                    ResourceServerCredentialVariant::Oauth2AuthorizationCodeFlow(
                        ref __field0,
                    ) => {
                        _serde::__private228::ser::serialize_tagged_newtype(
                            __serializer,
                            "ResourceServerCredentialVariant",
                            "Oauth2AuthorizationCodeFlow",
                            "type",
                            "Oauth2AuthorizationCodeFlow",
                            __field0,
                        )
                    }
                    ResourceServerCredentialVariant::Oauth2JwtBearerAssertionFlow(
                        ref __field0,
                    ) => {
                        _serde::__private228::ser::serialize_tagged_newtype(
                            __serializer,
                            "ResourceServerCredentialVariant",
                            "Oauth2JwtBearerAssertionFlow",
                            "type",
                            "Oauth2JwtBearerAssertionFlow",
                            __field0,
                        )
                    }
                    ResourceServerCredentialVariant::Custom(ref __field0) => {
                        _serde::__private228::ser::serialize_tagged_newtype(
                            __serializer,
                            "ResourceServerCredentialVariant",
                            "Custom",
                            "type",
                            "Custom",
                            __field0,
                        )
                    }
                }
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for ResourceServerCredentialVariant {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "variant identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            1u64 => _serde::__private228::Ok(__Field::__field1),
                            2u64 => _serde::__private228::Ok(__Field::__field2),
                            3u64 => _serde::__private228::Ok(__Field::__field3),
                            _ => {
                                _serde::__private228::Err(
                                    _serde::de::Error::invalid_value(
                                        _serde::de::Unexpected::Unsigned(__value),
                                        &"variant index 0 <= i < 4",
                                    ),
                                )
                            }
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "NoAuth" => _serde::__private228::Ok(__Field::__field0),
                            "Oauth2AuthorizationCodeFlow" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            "Oauth2JwtBearerAssertionFlow" => {
                                _serde::__private228::Ok(__Field::__field2)
                            }
                            "Custom" => _serde::__private228::Ok(__Field::__field3),
                            _ => {
                                _serde::__private228::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"NoAuth" => _serde::__private228::Ok(__Field::__field0),
                            b"Oauth2AuthorizationCodeFlow" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            b"Oauth2JwtBearerAssertionFlow" => {
                                _serde::__private228::Ok(__Field::__field2)
                            }
                            b"Custom" => _serde::__private228::Ok(__Field::__field3),
                            _ => {
                                let __value = &_serde::__private228::from_utf8_lossy(
                                    __value,
                                );
                                _serde::__private228::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                const VARIANTS: &'static [&'static str] = &[
                    "NoAuth",
                    "Oauth2AuthorizationCodeFlow",
                    "Oauth2JwtBearerAssertionFlow",
                    "Custom",
                ];
                let (__tag, __content) = _serde::Deserializer::deserialize_any(
                    __deserializer,
                    _serde::__private228::de::TaggedContentVisitor::<
                        __Field,
                    >::new(
                        "type",
                        "internally tagged enum ResourceServerCredentialVariant",
                    ),
                )?;
                let __deserializer = _serde::__private228::de::ContentDeserializer::<
                    __D::Error,
                >::new(__content);
                match __tag {
                    __Field::__field0 => {
                        _serde::__private228::Result::map(
                            <NoAuthResourceServerCredential as _serde::Deserialize>::deserialize(
                                __deserializer,
                            ),
                            ResourceServerCredentialVariant::NoAuth,
                        )
                    }
                    __Field::__field1 => {
                        _serde::__private228::Result::map(
                            <Oauth2AuthorizationCodeFlowResourceServerCredential as _serde::Deserialize>::deserialize(
                                __deserializer,
                            ),
                            ResourceServerCredentialVariant::Oauth2AuthorizationCodeFlow,
                        )
                    }
                    __Field::__field2 => {
                        _serde::__private228::Result::map(
                            <Oauth2JwtBearerAssertionFlowResourceServerCredential as _serde::Deserialize>::deserialize(
                                __deserializer,
                            ),
                            ResourceServerCredentialVariant::Oauth2JwtBearerAssertionFlow,
                        )
                    }
                    __Field::__field3 => {
                        _serde::__private228::Result::map(
                            <CustomResourceServerCredential as _serde::Deserialize>::deserialize(
                                __deserializer,
                            ),
                            ResourceServerCredentialVariant::Custom,
                        )
                    }
                }
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for ResourceServerCredentialVariant {
        #[inline]
        fn clone(&self) -> ResourceServerCredentialVariant {
            match self {
                ResourceServerCredentialVariant::NoAuth(__self_0) => {
                    ResourceServerCredentialVariant::NoAuth(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                ResourceServerCredentialVariant::Oauth2AuthorizationCodeFlow(
                    __self_0,
                ) => {
                    ResourceServerCredentialVariant::Oauth2AuthorizationCodeFlow(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                ResourceServerCredentialVariant::Oauth2JwtBearerAssertionFlow(
                    __self_0,
                ) => {
                    ResourceServerCredentialVariant::Oauth2JwtBearerAssertionFlow(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                ResourceServerCredentialVariant::Custom(__self_0) => {
                    ResourceServerCredentialVariant::Custom(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
            }
        }
    }
    #[serde(rename_all = "snake_case")]
    pub enum ResourceServerCredentialType {
        NoAuth,
        Oauth2AuthorizationCodeFlow,
        Oauth2JwtBearerAssertionFlow,
        Custom,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for ResourceServerCredentialType {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    ResourceServerCredentialType::NoAuth => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "ResourceServerCredentialType",
                            0u32,
                            "no_auth",
                        )
                    }
                    ResourceServerCredentialType::Oauth2AuthorizationCodeFlow => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "ResourceServerCredentialType",
                            1u32,
                            "oauth2_authorization_code_flow",
                        )
                    }
                    ResourceServerCredentialType::Oauth2JwtBearerAssertionFlow => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "ResourceServerCredentialType",
                            2u32,
                            "oauth2_jwt_bearer_assertion_flow",
                        )
                    }
                    ResourceServerCredentialType::Custom => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "ResourceServerCredentialType",
                            3u32,
                            "custom",
                        )
                    }
                }
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for ResourceServerCredentialType {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "variant identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            1u64 => _serde::__private228::Ok(__Field::__field1),
                            2u64 => _serde::__private228::Ok(__Field::__field2),
                            3u64 => _serde::__private228::Ok(__Field::__field3),
                            _ => {
                                _serde::__private228::Err(
                                    _serde::de::Error::invalid_value(
                                        _serde::de::Unexpected::Unsigned(__value),
                                        &"variant index 0 <= i < 4",
                                    ),
                                )
                            }
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "no_auth" => _serde::__private228::Ok(__Field::__field0),
                            "oauth2_authorization_code_flow" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            "oauth2_jwt_bearer_assertion_flow" => {
                                _serde::__private228::Ok(__Field::__field2)
                            }
                            "custom" => _serde::__private228::Ok(__Field::__field3),
                            _ => {
                                _serde::__private228::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"no_auth" => _serde::__private228::Ok(__Field::__field0),
                            b"oauth2_authorization_code_flow" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            b"oauth2_jwt_bearer_assertion_flow" => {
                                _serde::__private228::Ok(__Field::__field2)
                            }
                            b"custom" => _serde::__private228::Ok(__Field::__field3),
                            _ => {
                                let __value = &_serde::__private228::from_utf8_lossy(
                                    __value,
                                );
                                _serde::__private228::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<
                        ResourceServerCredentialType,
                    >,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = ResourceServerCredentialType;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "enum ResourceServerCredentialType",
                        )
                    }
                    fn visit_enum<__A>(
                        self,
                        __data: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::EnumAccess<'de>,
                    {
                        match _serde::de::EnumAccess::variant(__data)? {
                            (__Field::__field0, __variant) => {
                                _serde::de::VariantAccess::unit_variant(__variant)?;
                                _serde::__private228::Ok(
                                    ResourceServerCredentialType::NoAuth,
                                )
                            }
                            (__Field::__field1, __variant) => {
                                _serde::de::VariantAccess::unit_variant(__variant)?;
                                _serde::__private228::Ok(
                                    ResourceServerCredentialType::Oauth2AuthorizationCodeFlow,
                                )
                            }
                            (__Field::__field2, __variant) => {
                                _serde::de::VariantAccess::unit_variant(__variant)?;
                                _serde::__private228::Ok(
                                    ResourceServerCredentialType::Oauth2JwtBearerAssertionFlow,
                                )
                            }
                            (__Field::__field3, __variant) => {
                                _serde::de::VariantAccess::unit_variant(__variant)?;
                                _serde::__private228::Ok(
                                    ResourceServerCredentialType::Custom,
                                )
                            }
                        }
                    }
                }
                #[doc(hidden)]
                const VARIANTS: &'static [&'static str] = &[
                    "no_auth",
                    "oauth2_authorization_code_flow",
                    "oauth2_jwt_bearer_assertion_flow",
                    "custom",
                ];
                _serde::Deserializer::deserialize_enum(
                    __deserializer,
                    "ResourceServerCredentialType",
                    VARIANTS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<
                            ResourceServerCredentialType,
                        >,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for ResourceServerCredentialType {
        #[inline]
        fn clone(&self) -> ResourceServerCredentialType {
            match self {
                ResourceServerCredentialType::NoAuth => {
                    ResourceServerCredentialType::NoAuth
                }
                ResourceServerCredentialType::Oauth2AuthorizationCodeFlow => {
                    ResourceServerCredentialType::Oauth2AuthorizationCodeFlow
                }
                ResourceServerCredentialType::Oauth2JwtBearerAssertionFlow => {
                    ResourceServerCredentialType::Oauth2JwtBearerAssertionFlow
                }
                ResourceServerCredentialType::Custom => {
                    ResourceServerCredentialType::Custom
                }
            }
        }
    }
    pub type ResourceServerCredential = DatabaseCredential<
        ResourceServerCredentialVariant,
    >;
    #[serde(tag = "type")]
    pub enum UserCredentialVariant {
        NoAuth(NoAuthUserCredential),
        Oauth2AuthorizationCodeFlow(Oauth2AuthorizationCodeFlowUserCredential),
        Oauth2JwtBearerAssertionFlow(Oauth2JwtBearerAssertionFlowUserCredential),
        Custom(CustomUserCredential),
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for UserCredentialVariant {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    UserCredentialVariant::NoAuth(ref __field0) => {
                        _serde::__private228::ser::serialize_tagged_newtype(
                            __serializer,
                            "UserCredentialVariant",
                            "NoAuth",
                            "type",
                            "NoAuth",
                            __field0,
                        )
                    }
                    UserCredentialVariant::Oauth2AuthorizationCodeFlow(ref __field0) => {
                        _serde::__private228::ser::serialize_tagged_newtype(
                            __serializer,
                            "UserCredentialVariant",
                            "Oauth2AuthorizationCodeFlow",
                            "type",
                            "Oauth2AuthorizationCodeFlow",
                            __field0,
                        )
                    }
                    UserCredentialVariant::Oauth2JwtBearerAssertionFlow(ref __field0) => {
                        _serde::__private228::ser::serialize_tagged_newtype(
                            __serializer,
                            "UserCredentialVariant",
                            "Oauth2JwtBearerAssertionFlow",
                            "type",
                            "Oauth2JwtBearerAssertionFlow",
                            __field0,
                        )
                    }
                    UserCredentialVariant::Custom(ref __field0) => {
                        _serde::__private228::ser::serialize_tagged_newtype(
                            __serializer,
                            "UserCredentialVariant",
                            "Custom",
                            "type",
                            "Custom",
                            __field0,
                        )
                    }
                }
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for UserCredentialVariant {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "variant identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            1u64 => _serde::__private228::Ok(__Field::__field1),
                            2u64 => _serde::__private228::Ok(__Field::__field2),
                            3u64 => _serde::__private228::Ok(__Field::__field3),
                            _ => {
                                _serde::__private228::Err(
                                    _serde::de::Error::invalid_value(
                                        _serde::de::Unexpected::Unsigned(__value),
                                        &"variant index 0 <= i < 4",
                                    ),
                                )
                            }
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "NoAuth" => _serde::__private228::Ok(__Field::__field0),
                            "Oauth2AuthorizationCodeFlow" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            "Oauth2JwtBearerAssertionFlow" => {
                                _serde::__private228::Ok(__Field::__field2)
                            }
                            "Custom" => _serde::__private228::Ok(__Field::__field3),
                            _ => {
                                _serde::__private228::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"NoAuth" => _serde::__private228::Ok(__Field::__field0),
                            b"Oauth2AuthorizationCodeFlow" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            b"Oauth2JwtBearerAssertionFlow" => {
                                _serde::__private228::Ok(__Field::__field2)
                            }
                            b"Custom" => _serde::__private228::Ok(__Field::__field3),
                            _ => {
                                let __value = &_serde::__private228::from_utf8_lossy(
                                    __value,
                                );
                                _serde::__private228::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                const VARIANTS: &'static [&'static str] = &[
                    "NoAuth",
                    "Oauth2AuthorizationCodeFlow",
                    "Oauth2JwtBearerAssertionFlow",
                    "Custom",
                ];
                let (__tag, __content) = _serde::Deserializer::deserialize_any(
                    __deserializer,
                    _serde::__private228::de::TaggedContentVisitor::<
                        __Field,
                    >::new("type", "internally tagged enum UserCredentialVariant"),
                )?;
                let __deserializer = _serde::__private228::de::ContentDeserializer::<
                    __D::Error,
                >::new(__content);
                match __tag {
                    __Field::__field0 => {
                        _serde::__private228::Result::map(
                            <NoAuthUserCredential as _serde::Deserialize>::deserialize(
                                __deserializer,
                            ),
                            UserCredentialVariant::NoAuth,
                        )
                    }
                    __Field::__field1 => {
                        _serde::__private228::Result::map(
                            <Oauth2AuthorizationCodeFlowUserCredential as _serde::Deserialize>::deserialize(
                                __deserializer,
                            ),
                            UserCredentialVariant::Oauth2AuthorizationCodeFlow,
                        )
                    }
                    __Field::__field2 => {
                        _serde::__private228::Result::map(
                            <Oauth2JwtBearerAssertionFlowUserCredential as _serde::Deserialize>::deserialize(
                                __deserializer,
                            ),
                            UserCredentialVariant::Oauth2JwtBearerAssertionFlow,
                        )
                    }
                    __Field::__field3 => {
                        _serde::__private228::Result::map(
                            <CustomUserCredential as _serde::Deserialize>::deserialize(
                                __deserializer,
                            ),
                            UserCredentialVariant::Custom,
                        )
                    }
                }
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for UserCredentialVariant {
        #[inline]
        fn clone(&self) -> UserCredentialVariant {
            match self {
                UserCredentialVariant::NoAuth(__self_0) => {
                    UserCredentialVariant::NoAuth(::core::clone::Clone::clone(__self_0))
                }
                UserCredentialVariant::Oauth2AuthorizationCodeFlow(__self_0) => {
                    UserCredentialVariant::Oauth2AuthorizationCodeFlow(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                UserCredentialVariant::Oauth2JwtBearerAssertionFlow(__self_0) => {
                    UserCredentialVariant::Oauth2JwtBearerAssertionFlow(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                UserCredentialVariant::Custom(__self_0) => {
                    UserCredentialVariant::Custom(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    pub type UserCredential = DatabaseCredential<UserCredentialVariant>;
    pub struct NoAuthFullCredential {
        pub static_cred: NoAuthStaticCredentialConfiguration,
        pub resource_server_cred: NoAuthResourceServerCredential,
        pub user_cred: NoAuthUserCredential,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for NoAuthFullCredential {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "NoAuthFullCredential",
                    false as usize + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "static_cred",
                    &self.static_cred,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "resource_server_cred",
                    &self.resource_server_cred,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "user_cred",
                    &self.user_cred,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for NoAuthFullCredential {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            1u64 => _serde::__private228::Ok(__Field::__field1),
                            2u64 => _serde::__private228::Ok(__Field::__field2),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "static_cred" => _serde::__private228::Ok(__Field::__field0),
                            "resource_server_cred" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            "user_cred" => _serde::__private228::Ok(__Field::__field2),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"static_cred" => _serde::__private228::Ok(__Field::__field0),
                            b"resource_server_cred" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            b"user_cred" => _serde::__private228::Ok(__Field::__field2),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<NoAuthFullCredential>,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = NoAuthFullCredential;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct NoAuthFullCredential",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            NoAuthStaticCredentialConfiguration,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct NoAuthFullCredential with 3 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            NoAuthResourceServerCredential,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct NoAuthFullCredential with 3 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            NoAuthUserCredential,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct NoAuthFullCredential with 3 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(NoAuthFullCredential {
                            static_cred: __field0,
                            resource_server_cred: __field1,
                            user_cred: __field2,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<
                            NoAuthStaticCredentialConfiguration,
                        > = _serde::__private228::None;
                        let mut __field1: _serde::__private228::Option<
                            NoAuthResourceServerCredential,
                        > = _serde::__private228::None;
                        let mut __field2: _serde::__private228::Option<
                            NoAuthUserCredential,
                        > = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "static_cred",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            NoAuthStaticCredentialConfiguration,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private228::Option::is_some(&__field1) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "resource_server_cred",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            NoAuthResourceServerCredential,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private228::Option::is_some(&__field2) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "user_cred",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            NoAuthUserCredential,
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("static_cred")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private228::Some(__field1) => __field1,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field(
                                    "resource_server_cred",
                                )?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private228::Some(__field2) => __field2,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("user_cred")?
                            }
                        };
                        _serde::__private228::Ok(NoAuthFullCredential {
                            static_cred: __field0,
                            resource_server_cred: __field1,
                            user_cred: __field2,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "static_cred",
                    "resource_server_cred",
                    "user_cred",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "NoAuthFullCredential",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<
                            NoAuthFullCredential,
                        >,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for NoAuthFullCredential {
        #[inline]
        fn clone(&self) -> NoAuthFullCredential {
            NoAuthFullCredential {
                static_cred: ::core::clone::Clone::clone(&self.static_cred),
                resource_server_cred: ::core::clone::Clone::clone(
                    &self.resource_server_cred,
                ),
                user_cred: ::core::clone::Clone::clone(&self.user_cred),
            }
        }
    }
    pub struct Oauth2AuthorizationCodeFlowFullCredential {
        pub static_cred: Oauth2AuthorizationCodeFlowStaticCredentialConfiguration,
        pub resource_server_cred: Oauth2AuthorizationCodeFlowResourceServerCredential,
        pub user_cred: Oauth2AuthorizationCodeFlowUserCredential,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for Oauth2AuthorizationCodeFlowFullCredential {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "Oauth2AuthorizationCodeFlowFullCredential",
                    false as usize + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "static_cred",
                    &self.static_cred,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "resource_server_cred",
                    &self.resource_server_cred,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "user_cred",
                    &self.user_cred,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de>
        for Oauth2AuthorizationCodeFlowFullCredential {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            1u64 => _serde::__private228::Ok(__Field::__field1),
                            2u64 => _serde::__private228::Ok(__Field::__field2),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "static_cred" => _serde::__private228::Ok(__Field::__field0),
                            "resource_server_cred" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            "user_cred" => _serde::__private228::Ok(__Field::__field2),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"static_cred" => _serde::__private228::Ok(__Field::__field0),
                            b"resource_server_cred" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            b"user_cred" => _serde::__private228::Ok(__Field::__field2),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<
                        Oauth2AuthorizationCodeFlowFullCredential,
                    >,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = Oauth2AuthorizationCodeFlowFullCredential;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct Oauth2AuthorizationCodeFlowFullCredential",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            Oauth2AuthorizationCodeFlowStaticCredentialConfiguration,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct Oauth2AuthorizationCodeFlowFullCredential with 3 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            Oauth2AuthorizationCodeFlowResourceServerCredential,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct Oauth2AuthorizationCodeFlowFullCredential with 3 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            Oauth2AuthorizationCodeFlowUserCredential,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct Oauth2AuthorizationCodeFlowFullCredential with 3 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(Oauth2AuthorizationCodeFlowFullCredential {
                            static_cred: __field0,
                            resource_server_cred: __field1,
                            user_cred: __field2,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<
                            Oauth2AuthorizationCodeFlowStaticCredentialConfiguration,
                        > = _serde::__private228::None;
                        let mut __field1: _serde::__private228::Option<
                            Oauth2AuthorizationCodeFlowResourceServerCredential,
                        > = _serde::__private228::None;
                        let mut __field2: _serde::__private228::Option<
                            Oauth2AuthorizationCodeFlowUserCredential,
                        > = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "static_cred",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Oauth2AuthorizationCodeFlowStaticCredentialConfiguration,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private228::Option::is_some(&__field1) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "resource_server_cred",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Oauth2AuthorizationCodeFlowResourceServerCredential,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private228::Option::is_some(&__field2) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "user_cred",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Oauth2AuthorizationCodeFlowUserCredential,
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("static_cred")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private228::Some(__field1) => __field1,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field(
                                    "resource_server_cred",
                                )?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private228::Some(__field2) => __field2,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("user_cred")?
                            }
                        };
                        _serde::__private228::Ok(Oauth2AuthorizationCodeFlowFullCredential {
                            static_cred: __field0,
                            resource_server_cred: __field1,
                            user_cred: __field2,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "static_cred",
                    "resource_server_cred",
                    "user_cred",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "Oauth2AuthorizationCodeFlowFullCredential",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<
                            Oauth2AuthorizationCodeFlowFullCredential,
                        >,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for Oauth2AuthorizationCodeFlowFullCredential {
        #[inline]
        fn clone(&self) -> Oauth2AuthorizationCodeFlowFullCredential {
            Oauth2AuthorizationCodeFlowFullCredential {
                static_cred: ::core::clone::Clone::clone(&self.static_cred),
                resource_server_cred: ::core::clone::Clone::clone(
                    &self.resource_server_cred,
                ),
                user_cred: ::core::clone::Clone::clone(&self.user_cred),
            }
        }
    }
    pub struct Oauth2JwtBearerAssertionFlowFullCredential {
        pub static_cred: Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration,
        pub resource_server_cred: Oauth2JwtBearerAssertionFlowResourceServerCredential,
        pub user_cred: Oauth2JwtBearerAssertionFlowUserCredential,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for Oauth2JwtBearerAssertionFlowFullCredential {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "Oauth2JwtBearerAssertionFlowFullCredential",
                    false as usize + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "static_cred",
                    &self.static_cred,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "resource_server_cred",
                    &self.resource_server_cred,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "user_cred",
                    &self.user_cred,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de>
        for Oauth2JwtBearerAssertionFlowFullCredential {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            1u64 => _serde::__private228::Ok(__Field::__field1),
                            2u64 => _serde::__private228::Ok(__Field::__field2),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "static_cred" => _serde::__private228::Ok(__Field::__field0),
                            "resource_server_cred" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            "user_cred" => _serde::__private228::Ok(__Field::__field2),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"static_cred" => _serde::__private228::Ok(__Field::__field0),
                            b"resource_server_cred" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            b"user_cred" => _serde::__private228::Ok(__Field::__field2),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<
                        Oauth2JwtBearerAssertionFlowFullCredential,
                    >,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = Oauth2JwtBearerAssertionFlowFullCredential;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct Oauth2JwtBearerAssertionFlowFullCredential",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct Oauth2JwtBearerAssertionFlowFullCredential with 3 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            Oauth2JwtBearerAssertionFlowResourceServerCredential,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct Oauth2JwtBearerAssertionFlowFullCredential with 3 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            Oauth2JwtBearerAssertionFlowUserCredential,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct Oauth2JwtBearerAssertionFlowFullCredential with 3 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(Oauth2JwtBearerAssertionFlowFullCredential {
                            static_cred: __field0,
                            resource_server_cred: __field1,
                            user_cred: __field2,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<
                            Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration,
                        > = _serde::__private228::None;
                        let mut __field1: _serde::__private228::Option<
                            Oauth2JwtBearerAssertionFlowResourceServerCredential,
                        > = _serde::__private228::None;
                        let mut __field2: _serde::__private228::Option<
                            Oauth2JwtBearerAssertionFlowUserCredential,
                        > = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "static_cred",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private228::Option::is_some(&__field1) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "resource_server_cred",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Oauth2JwtBearerAssertionFlowResourceServerCredential,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private228::Option::is_some(&__field2) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "user_cred",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Oauth2JwtBearerAssertionFlowUserCredential,
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("static_cred")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private228::Some(__field1) => __field1,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field(
                                    "resource_server_cred",
                                )?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private228::Some(__field2) => __field2,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("user_cred")?
                            }
                        };
                        _serde::__private228::Ok(Oauth2JwtBearerAssertionFlowFullCredential {
                            static_cred: __field0,
                            resource_server_cred: __field1,
                            user_cred: __field2,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "static_cred",
                    "resource_server_cred",
                    "user_cred",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "Oauth2JwtBearerAssertionFlowFullCredential",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<
                            Oauth2JwtBearerAssertionFlowFullCredential,
                        >,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for Oauth2JwtBearerAssertionFlowFullCredential {
        #[inline]
        fn clone(&self) -> Oauth2JwtBearerAssertionFlowFullCredential {
            Oauth2JwtBearerAssertionFlowFullCredential {
                static_cred: ::core::clone::Clone::clone(&self.static_cred),
                resource_server_cred: ::core::clone::Clone::clone(
                    &self.resource_server_cred,
                ),
                user_cred: ::core::clone::Clone::clone(&self.user_cred),
            }
        }
    }
    pub struct CustomFullCredential {
        pub static_cred: CustomStaticCredentialConfiguration,
        pub resource_server_cred: CustomResourceServerCredential,
        pub user_cred: CustomUserCredential,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for CustomFullCredential {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "CustomFullCredential",
                    false as usize + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "static_cred",
                    &self.static_cred,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "resource_server_cred",
                    &self.resource_server_cred,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "user_cred",
                    &self.user_cred,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for CustomFullCredential {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            1u64 => _serde::__private228::Ok(__Field::__field1),
                            2u64 => _serde::__private228::Ok(__Field::__field2),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "static_cred" => _serde::__private228::Ok(__Field::__field0),
                            "resource_server_cred" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            "user_cred" => _serde::__private228::Ok(__Field::__field2),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"static_cred" => _serde::__private228::Ok(__Field::__field0),
                            b"resource_server_cred" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            b"user_cred" => _serde::__private228::Ok(__Field::__field2),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<CustomFullCredential>,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = CustomFullCredential;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct CustomFullCredential",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            CustomStaticCredentialConfiguration,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct CustomFullCredential with 3 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            CustomResourceServerCredential,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct CustomFullCredential with 3 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            CustomUserCredential,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct CustomFullCredential with 3 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(CustomFullCredential {
                            static_cred: __field0,
                            resource_server_cred: __field1,
                            user_cred: __field2,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<
                            CustomStaticCredentialConfiguration,
                        > = _serde::__private228::None;
                        let mut __field1: _serde::__private228::Option<
                            CustomResourceServerCredential,
                        > = _serde::__private228::None;
                        let mut __field2: _serde::__private228::Option<
                            CustomUserCredential,
                        > = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "static_cred",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            CustomStaticCredentialConfiguration,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private228::Option::is_some(&__field1) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "resource_server_cred",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            CustomResourceServerCredential,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private228::Option::is_some(&__field2) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "user_cred",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            CustomUserCredential,
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("static_cred")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private228::Some(__field1) => __field1,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field(
                                    "resource_server_cred",
                                )?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private228::Some(__field2) => __field2,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("user_cred")?
                            }
                        };
                        _serde::__private228::Ok(CustomFullCredential {
                            static_cred: __field0,
                            resource_server_cred: __field1,
                            user_cred: __field2,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "static_cred",
                    "resource_server_cred",
                    "user_cred",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "CustomFullCredential",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<
                            CustomFullCredential,
                        >,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for CustomFullCredential {
        #[inline]
        fn clone(&self) -> CustomFullCredential {
            CustomFullCredential {
                static_cred: ::core::clone::Clone::clone(&self.static_cred),
                resource_server_cred: ::core::clone::Clone::clone(
                    &self.resource_server_cred,
                ),
                user_cred: ::core::clone::Clone::clone(&self.user_cred),
            }
        }
    }
    pub struct NoAuthResourceServerCredential {
        pub metadata: Metadata,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for NoAuthResourceServerCredential {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "NoAuthResourceServerCredential",
                    false as usize + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "metadata",
                    &self.metadata,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for NoAuthResourceServerCredential {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "metadata" => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"metadata" => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<
                        NoAuthResourceServerCredential,
                    >,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = NoAuthResourceServerCredential;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct NoAuthResourceServerCredential",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            Metadata,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct NoAuthResourceServerCredential with 1 element",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(NoAuthResourceServerCredential {
                            metadata: __field0,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<Metadata> = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "metadata",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<Metadata>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("metadata")?
                            }
                        };
                        _serde::__private228::Ok(NoAuthResourceServerCredential {
                            metadata: __field0,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &["metadata"];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "NoAuthResourceServerCredential",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<
                            NoAuthResourceServerCredential,
                        >,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for NoAuthResourceServerCredential {
        #[inline]
        fn clone(&self) -> NoAuthResourceServerCredential {
            NoAuthResourceServerCredential {
                metadata: ::core::clone::Clone::clone(&self.metadata),
            }
        }
    }
    pub struct Oauth2AuthorizationCodeFlowResourceServerCredential {
        pub client_id: String,
        pub client_secret: String,
        pub redirect_uri: String,
        pub metadata: Metadata,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for Oauth2AuthorizationCodeFlowResourceServerCredential {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "Oauth2AuthorizationCodeFlowResourceServerCredential",
                    false as usize + 1 + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "client_id",
                    &self.client_id,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "client_secret",
                    &self.client_secret,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "redirect_uri",
                    &self.redirect_uri,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "metadata",
                    &self.metadata,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de>
        for Oauth2AuthorizationCodeFlowResourceServerCredential {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            1u64 => _serde::__private228::Ok(__Field::__field1),
                            2u64 => _serde::__private228::Ok(__Field::__field2),
                            3u64 => _serde::__private228::Ok(__Field::__field3),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "client_id" => _serde::__private228::Ok(__Field::__field0),
                            "client_secret" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            "redirect_uri" => _serde::__private228::Ok(__Field::__field2),
                            "metadata" => _serde::__private228::Ok(__Field::__field3),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"client_id" => _serde::__private228::Ok(__Field::__field0),
                            b"client_secret" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            b"redirect_uri" => {
                                _serde::__private228::Ok(__Field::__field2)
                            }
                            b"metadata" => _serde::__private228::Ok(__Field::__field3),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<
                        Oauth2AuthorizationCodeFlowResourceServerCredential,
                    >,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = Oauth2AuthorizationCodeFlowResourceServerCredential;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct Oauth2AuthorizationCodeFlowResourceServerCredential",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct Oauth2AuthorizationCodeFlowResourceServerCredential with 4 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct Oauth2AuthorizationCodeFlowResourceServerCredential with 4 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct Oauth2AuthorizationCodeFlowResourceServerCredential with 4 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            Metadata,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct Oauth2AuthorizationCodeFlowResourceServerCredential with 4 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(Oauth2AuthorizationCodeFlowResourceServerCredential {
                            client_id: __field0,
                            client_secret: __field1,
                            redirect_uri: __field2,
                            metadata: __field3,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field1: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field2: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field3: _serde::__private228::Option<Metadata> = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "client_id",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private228::Option::is_some(&__field1) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "client_secret",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private228::Option::is_some(&__field2) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "redirect_uri",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private228::Option::is_some(&__field3) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "metadata",
                                            ),
                                        );
                                    }
                                    __field3 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<Metadata>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("client_id")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private228::Some(__field1) => __field1,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("client_secret")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private228::Some(__field2) => __field2,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("redirect_uri")?
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private228::Some(__field3) => __field3,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("metadata")?
                            }
                        };
                        _serde::__private228::Ok(Oauth2AuthorizationCodeFlowResourceServerCredential {
                            client_id: __field0,
                            client_secret: __field1,
                            redirect_uri: __field2,
                            metadata: __field3,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "client_id",
                    "client_secret",
                    "redirect_uri",
                    "metadata",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "Oauth2AuthorizationCodeFlowResourceServerCredential",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<
                            Oauth2AuthorizationCodeFlowResourceServerCredential,
                        >,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for Oauth2AuthorizationCodeFlowResourceServerCredential {
        #[inline]
        fn clone(&self) -> Oauth2AuthorizationCodeFlowResourceServerCredential {
            Oauth2AuthorizationCodeFlowResourceServerCredential {
                client_id: ::core::clone::Clone::clone(&self.client_id),
                client_secret: ::core::clone::Clone::clone(&self.client_secret),
                redirect_uri: ::core::clone::Clone::clone(&self.redirect_uri),
                metadata: ::core::clone::Clone::clone(&self.metadata),
            }
        }
    }
    pub struct Oauth2JwtBearerAssertionFlowResourceServerCredential {
        pub client_id: String,
        pub client_secret: String,
        pub redirect_uri: String,
        pub metadata: Metadata,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for Oauth2JwtBearerAssertionFlowResourceServerCredential {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "Oauth2JwtBearerAssertionFlowResourceServerCredential",
                    false as usize + 1 + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "client_id",
                    &self.client_id,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "client_secret",
                    &self.client_secret,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "redirect_uri",
                    &self.redirect_uri,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "metadata",
                    &self.metadata,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de>
        for Oauth2JwtBearerAssertionFlowResourceServerCredential {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            1u64 => _serde::__private228::Ok(__Field::__field1),
                            2u64 => _serde::__private228::Ok(__Field::__field2),
                            3u64 => _serde::__private228::Ok(__Field::__field3),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "client_id" => _serde::__private228::Ok(__Field::__field0),
                            "client_secret" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            "redirect_uri" => _serde::__private228::Ok(__Field::__field2),
                            "metadata" => _serde::__private228::Ok(__Field::__field3),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"client_id" => _serde::__private228::Ok(__Field::__field0),
                            b"client_secret" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            b"redirect_uri" => {
                                _serde::__private228::Ok(__Field::__field2)
                            }
                            b"metadata" => _serde::__private228::Ok(__Field::__field3),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<
                        Oauth2JwtBearerAssertionFlowResourceServerCredential,
                    >,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = Oauth2JwtBearerAssertionFlowResourceServerCredential;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct Oauth2JwtBearerAssertionFlowResourceServerCredential",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct Oauth2JwtBearerAssertionFlowResourceServerCredential with 4 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct Oauth2JwtBearerAssertionFlowResourceServerCredential with 4 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct Oauth2JwtBearerAssertionFlowResourceServerCredential with 4 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            Metadata,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct Oauth2JwtBearerAssertionFlowResourceServerCredential with 4 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(Oauth2JwtBearerAssertionFlowResourceServerCredential {
                            client_id: __field0,
                            client_secret: __field1,
                            redirect_uri: __field2,
                            metadata: __field3,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field1: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field2: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field3: _serde::__private228::Option<Metadata> = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "client_id",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private228::Option::is_some(&__field1) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "client_secret",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private228::Option::is_some(&__field2) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "redirect_uri",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private228::Option::is_some(&__field3) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "metadata",
                                            ),
                                        );
                                    }
                                    __field3 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<Metadata>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("client_id")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private228::Some(__field1) => __field1,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("client_secret")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private228::Some(__field2) => __field2,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("redirect_uri")?
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private228::Some(__field3) => __field3,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("metadata")?
                            }
                        };
                        _serde::__private228::Ok(Oauth2JwtBearerAssertionFlowResourceServerCredential {
                            client_id: __field0,
                            client_secret: __field1,
                            redirect_uri: __field2,
                            metadata: __field3,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "client_id",
                    "client_secret",
                    "redirect_uri",
                    "metadata",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "Oauth2JwtBearerAssertionFlowResourceServerCredential",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<
                            Oauth2JwtBearerAssertionFlowResourceServerCredential,
                        >,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for Oauth2JwtBearerAssertionFlowResourceServerCredential {
        #[inline]
        fn clone(&self) -> Oauth2JwtBearerAssertionFlowResourceServerCredential {
            Oauth2JwtBearerAssertionFlowResourceServerCredential {
                client_id: ::core::clone::Clone::clone(&self.client_id),
                client_secret: ::core::clone::Clone::clone(&self.client_secret),
                redirect_uri: ::core::clone::Clone::clone(&self.redirect_uri),
                metadata: ::core::clone::Clone::clone(&self.metadata),
            }
        }
    }
    pub struct Oauth2AuthorizationCodeFlowUserCredential {
        pub code: String,
        pub access_token: String,
        pub refresh_token: String,
        pub expiry_time: WrappedChronoDateTime,
        pub sub: String,
        pub metadata: Metadata,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for Oauth2AuthorizationCodeFlowUserCredential {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "Oauth2AuthorizationCodeFlowUserCredential",
                    false as usize + 1 + 1 + 1 + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "code",
                    &self.code,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "access_token",
                    &self.access_token,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "refresh_token",
                    &self.refresh_token,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "expiry_time",
                    &self.expiry_time,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "sub",
                    &self.sub,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "metadata",
                    &self.metadata,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de>
        for Oauth2AuthorizationCodeFlowUserCredential {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            1u64 => _serde::__private228::Ok(__Field::__field1),
                            2u64 => _serde::__private228::Ok(__Field::__field2),
                            3u64 => _serde::__private228::Ok(__Field::__field3),
                            4u64 => _serde::__private228::Ok(__Field::__field4),
                            5u64 => _serde::__private228::Ok(__Field::__field5),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "code" => _serde::__private228::Ok(__Field::__field0),
                            "access_token" => _serde::__private228::Ok(__Field::__field1),
                            "refresh_token" => {
                                _serde::__private228::Ok(__Field::__field2)
                            }
                            "expiry_time" => _serde::__private228::Ok(__Field::__field3),
                            "sub" => _serde::__private228::Ok(__Field::__field4),
                            "metadata" => _serde::__private228::Ok(__Field::__field5),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"code" => _serde::__private228::Ok(__Field::__field0),
                            b"access_token" => {
                                _serde::__private228::Ok(__Field::__field1)
                            }
                            b"refresh_token" => {
                                _serde::__private228::Ok(__Field::__field2)
                            }
                            b"expiry_time" => _serde::__private228::Ok(__Field::__field3),
                            b"sub" => _serde::__private228::Ok(__Field::__field4),
                            b"metadata" => _serde::__private228::Ok(__Field::__field5),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<
                        Oauth2AuthorizationCodeFlowUserCredential,
                    >,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = Oauth2AuthorizationCodeFlowUserCredential;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct Oauth2AuthorizationCodeFlowUserCredential",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct Oauth2AuthorizationCodeFlowUserCredential with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct Oauth2AuthorizationCodeFlowUserCredential with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct Oauth2AuthorizationCodeFlowUserCredential with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            WrappedChronoDateTime,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct Oauth2AuthorizationCodeFlowUserCredential with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field4 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        4usize,
                                        &"struct Oauth2AuthorizationCodeFlowUserCredential with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field5 = match _serde::de::SeqAccess::next_element::<
                            Metadata,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        5usize,
                                        &"struct Oauth2AuthorizationCodeFlowUserCredential with 6 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(Oauth2AuthorizationCodeFlowUserCredential {
                            code: __field0,
                            access_token: __field1,
                            refresh_token: __field2,
                            expiry_time: __field3,
                            sub: __field4,
                            metadata: __field5,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field1: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field2: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field3: _serde::__private228::Option<
                            WrappedChronoDateTime,
                        > = _serde::__private228::None;
                        let mut __field4: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field5: _serde::__private228::Option<Metadata> = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("code"),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private228::Option::is_some(&__field1) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "access_token",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private228::Option::is_some(&__field2) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "refresh_token",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private228::Option::is_some(&__field3) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "expiry_time",
                                            ),
                                        );
                                    }
                                    __field3 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            WrappedChronoDateTime,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private228::Option::is_some(&__field4) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("sub"),
                                        );
                                    }
                                    __field4 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private228::Option::is_some(&__field5) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "metadata",
                                            ),
                                        );
                                    }
                                    __field5 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<Metadata>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("code")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private228::Some(__field1) => __field1,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("access_token")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private228::Some(__field2) => __field2,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("refresh_token")?
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private228::Some(__field3) => __field3,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("expiry_time")?
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private228::Some(__field4) => __field4,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("sub")?
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private228::Some(__field5) => __field5,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("metadata")?
                            }
                        };
                        _serde::__private228::Ok(Oauth2AuthorizationCodeFlowUserCredential {
                            code: __field0,
                            access_token: __field1,
                            refresh_token: __field2,
                            expiry_time: __field3,
                            sub: __field4,
                            metadata: __field5,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "code",
                    "access_token",
                    "refresh_token",
                    "expiry_time",
                    "sub",
                    "metadata",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "Oauth2AuthorizationCodeFlowUserCredential",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<
                            Oauth2AuthorizationCodeFlowUserCredential,
                        >,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for Oauth2AuthorizationCodeFlowUserCredential {
        #[inline]
        fn clone(&self) -> Oauth2AuthorizationCodeFlowUserCredential {
            Oauth2AuthorizationCodeFlowUserCredential {
                code: ::core::clone::Clone::clone(&self.code),
                access_token: ::core::clone::Clone::clone(&self.access_token),
                refresh_token: ::core::clone::Clone::clone(&self.refresh_token),
                expiry_time: ::core::clone::Clone::clone(&self.expiry_time),
                sub: ::core::clone::Clone::clone(&self.sub),
                metadata: ::core::clone::Clone::clone(&self.metadata),
            }
        }
    }
    pub struct NoAuthUserCredential {
        pub metadata: Metadata,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for NoAuthUserCredential {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "NoAuthUserCredential",
                    false as usize + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "metadata",
                    &self.metadata,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for NoAuthUserCredential {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "metadata" => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"metadata" => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<NoAuthUserCredential>,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = NoAuthUserCredential;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct NoAuthUserCredential",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            Metadata,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct NoAuthUserCredential with 1 element",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(NoAuthUserCredential {
                            metadata: __field0,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<Metadata> = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "metadata",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<Metadata>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("metadata")?
                            }
                        };
                        _serde::__private228::Ok(NoAuthUserCredential {
                            metadata: __field0,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &["metadata"];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "NoAuthUserCredential",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<
                            NoAuthUserCredential,
                        >,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for NoAuthUserCredential {
        #[inline]
        fn clone(&self) -> NoAuthUserCredential {
            NoAuthUserCredential {
                metadata: ::core::clone::Clone::clone(&self.metadata),
            }
        }
    }
    pub struct Oauth2JwtBearerAssertionFlowUserCredential {
        pub assertion: String,
        pub token: String,
        pub expiry_time: WrappedChronoDateTime,
        pub sub: String,
        pub metadata: Metadata,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for Oauth2JwtBearerAssertionFlowUserCredential {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "Oauth2JwtBearerAssertionFlowUserCredential",
                    false as usize + 1 + 1 + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "assertion",
                    &self.assertion,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "token",
                    &self.token,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "expiry_time",
                    &self.expiry_time,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "sub",
                    &self.sub,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "metadata",
                    &self.metadata,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de>
        for Oauth2JwtBearerAssertionFlowUserCredential {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            1u64 => _serde::__private228::Ok(__Field::__field1),
                            2u64 => _serde::__private228::Ok(__Field::__field2),
                            3u64 => _serde::__private228::Ok(__Field::__field3),
                            4u64 => _serde::__private228::Ok(__Field::__field4),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "assertion" => _serde::__private228::Ok(__Field::__field0),
                            "token" => _serde::__private228::Ok(__Field::__field1),
                            "expiry_time" => _serde::__private228::Ok(__Field::__field2),
                            "sub" => _serde::__private228::Ok(__Field::__field3),
                            "metadata" => _serde::__private228::Ok(__Field::__field4),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"assertion" => _serde::__private228::Ok(__Field::__field0),
                            b"token" => _serde::__private228::Ok(__Field::__field1),
                            b"expiry_time" => _serde::__private228::Ok(__Field::__field2),
                            b"sub" => _serde::__private228::Ok(__Field::__field3),
                            b"metadata" => _serde::__private228::Ok(__Field::__field4),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<
                        Oauth2JwtBearerAssertionFlowUserCredential,
                    >,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = Oauth2JwtBearerAssertionFlowUserCredential;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct Oauth2JwtBearerAssertionFlowUserCredential",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct Oauth2JwtBearerAssertionFlowUserCredential with 5 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct Oauth2JwtBearerAssertionFlowUserCredential with 5 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            WrappedChronoDateTime,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct Oauth2JwtBearerAssertionFlowUserCredential with 5 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct Oauth2JwtBearerAssertionFlowUserCredential with 5 elements",
                                    ),
                                );
                            }
                        };
                        let __field4 = match _serde::de::SeqAccess::next_element::<
                            Metadata,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        4usize,
                                        &"struct Oauth2JwtBearerAssertionFlowUserCredential with 5 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(Oauth2JwtBearerAssertionFlowUserCredential {
                            assertion: __field0,
                            token: __field1,
                            expiry_time: __field2,
                            sub: __field3,
                            metadata: __field4,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field1: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field2: _serde::__private228::Option<
                            WrappedChronoDateTime,
                        > = _serde::__private228::None;
                        let mut __field3: _serde::__private228::Option<String> = _serde::__private228::None;
                        let mut __field4: _serde::__private228::Option<Metadata> = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "assertion",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private228::Option::is_some(&__field1) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("token"),
                                        );
                                    }
                                    __field1 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private228::Option::is_some(&__field2) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "expiry_time",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            WrappedChronoDateTime,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private228::Option::is_some(&__field3) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("sub"),
                                        );
                                    }
                                    __field3 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private228::Option::is_some(&__field4) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "metadata",
                                            ),
                                        );
                                    }
                                    __field4 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<Metadata>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("assertion")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private228::Some(__field1) => __field1,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("token")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private228::Some(__field2) => __field2,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("expiry_time")?
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private228::Some(__field3) => __field3,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("sub")?
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private228::Some(__field4) => __field4,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("metadata")?
                            }
                        };
                        _serde::__private228::Ok(Oauth2JwtBearerAssertionFlowUserCredential {
                            assertion: __field0,
                            token: __field1,
                            expiry_time: __field2,
                            sub: __field3,
                            metadata: __field4,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "assertion",
                    "token",
                    "expiry_time",
                    "sub",
                    "metadata",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "Oauth2JwtBearerAssertionFlowUserCredential",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<
                            Oauth2JwtBearerAssertionFlowUserCredential,
                        >,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for Oauth2JwtBearerAssertionFlowUserCredential {
        #[inline]
        fn clone(&self) -> Oauth2JwtBearerAssertionFlowUserCredential {
            Oauth2JwtBearerAssertionFlowUserCredential {
                assertion: ::core::clone::Clone::clone(&self.assertion),
                token: ::core::clone::Clone::clone(&self.token),
                expiry_time: ::core::clone::Clone::clone(&self.expiry_time),
                sub: ::core::clone::Clone::clone(&self.sub),
                metadata: ::core::clone::Clone::clone(&self.metadata),
            }
        }
    }
    pub struct CustomResourceServerCredential {
        pub metadata: Metadata,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for CustomResourceServerCredential {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "CustomResourceServerCredential",
                    false as usize + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "metadata",
                    &self.metadata,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for CustomResourceServerCredential {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "metadata" => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"metadata" => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<
                        CustomResourceServerCredential,
                    >,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = CustomResourceServerCredential;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct CustomResourceServerCredential",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            Metadata,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct CustomResourceServerCredential with 1 element",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(CustomResourceServerCredential {
                            metadata: __field0,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<Metadata> = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "metadata",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<Metadata>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("metadata")?
                            }
                        };
                        _serde::__private228::Ok(CustomResourceServerCredential {
                            metadata: __field0,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &["metadata"];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "CustomResourceServerCredential",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<
                            CustomResourceServerCredential,
                        >,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for CustomResourceServerCredential {
        #[inline]
        fn clone(&self) -> CustomResourceServerCredential {
            CustomResourceServerCredential {
                metadata: ::core::clone::Clone::clone(&self.metadata),
            }
        }
    }
    pub struct CustomUserCredential {
        pub metadata: Metadata,
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for CustomUserCredential {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "CustomUserCredential",
                    false as usize + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "metadata",
                    &self.metadata,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for CustomUserCredential {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "metadata" => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"metadata" => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<CustomUserCredential>,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = CustomUserCredential;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct CustomUserCredential",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            Metadata,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct CustomUserCredential with 1 element",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(CustomUserCredential {
                            metadata: __field0,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<Metadata> = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "metadata",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<Metadata>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("metadata")?
                            }
                        };
                        _serde::__private228::Ok(CustomUserCredential {
                            metadata: __field0,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &["metadata"];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "CustomUserCredential",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<
                            CustomUserCredential,
                        >,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for CustomUserCredential {
        #[inline]
        fn clone(&self) -> CustomUserCredential {
            CustomUserCredential {
                metadata: ::core::clone::Clone::clone(&self.metadata),
            }
        }
    }
    pub trait CredentialInjectorLike {
        fn inject_credentials(&self, request: &mut Request);
    }
    pub trait ProviderControllerLike {
        type ProviderInstance;
        async fn save_resource_server_credential(
            input: ResourceServerCredentialVariant,
        ) -> Result<ResourceServerCredential, CommonError>;
        async fn save_user_credential(
            input: UserCredentialVariant,
        ) -> Result<UserCredential, CommonError>;
        async fn get_static_credentials(
            variant: StaticCredentialConfigurationType,
        ) -> Result<StaticCredentialConfiguration, CommonError>;
        fn id() -> String;
        fn documentation_url() -> String;
        fn name() -> String;
    }
}
mod providers {
    pub mod google_mail {
        use serde::{Serialize, Deserialize};
        use crate::logic::*;
        use shared::{
            error::CommonError, primitives::{WrappedChronoDateTime, WrappedUuidV4},
        };
        pub struct GoogleMailController;
        impl ProviderControllerLike for GoogleMailController {
            type ProviderInstance = GoogleMailInstance;
            async fn save_resource_server_credential(
                input: ResourceServerCredentialVariant,
            ) -> Result<ResourceServerCredential, CommonError> {
                match input {
                    ResourceServerCredentialVariant::Oauth2AuthorizationCodeFlow(_) => {}
                    ResourceServerCredentialVariant::Oauth2JwtBearerAssertionFlow(_) => {}
                    _ => {
                        return Err(CommonError::InvalidRequest {
                            msg: "Unsupported credential type for google_mail".into(),
                            source: None,
                        });
                    }
                };
                Ok(ResourceServerCredential {
                    id: WrappedUuidV4::new(),
                    created_at: WrappedChronoDateTime::now(),
                    updated_at: WrappedChronoDateTime::now(),
                    inner: input,
                    metadata: Metadata::new(),
                })
            }
            async fn save_user_credential(
                input: UserCredentialVariant,
            ) -> Result<UserCredential, CommonError> {
                match input {
                    UserCredentialVariant::Oauth2AuthorizationCodeFlow(_) => {}
                    UserCredentialVariant::Oauth2JwtBearerAssertionFlow(_) => {}
                    _ => {
                        return Err(CommonError::InvalidRequest {
                            msg: "Unsupported user credential type for google_mail"
                                .into(),
                            source: None,
                        });
                    }
                };
                Ok(UserCredential {
                    id: WrappedUuidV4::new(),
                    created_at: WrappedChronoDateTime::now(),
                    updated_at: WrappedChronoDateTime::now(),
                    inner: input,
                    metadata: Metadata::new(),
                })
            }
            async fn get_static_credentials(
                variant: StaticCredentialConfigurationType,
            ) -> Result<StaticCredentialConfiguration, CommonError> {
                match variant {
                    StaticCredentialConfigurationType::Oauth2AuthorizationCodeFlow => {
                        let creds = Oauth2AuthorizationCodeFlowStaticCredentialConfiguration {
                            auth_uri: "https://accounts.google.com/o/oauth2/auth"
                                .to_string(),
                            token_uri: "https://oauth2.googleapis.com/token".to_string(),
                            userinfo_uri: "https://www.googleapis.com/oauth2/v3/userinfo"
                                .to_string(),
                            jwks_uri: "https://www.googleapis.com/oauth2/v3/jwks"
                                .to_string(),
                            issuer: "https://accounts.google.com".to_string(),
                            scopes: <[_]>::into_vec(
                                ::alloc::boxed::box_new([
                                    "https://www.googleapis.com/auth/gmail.readonly".to_string(),
                                ]),
                            ),
                            metadata: Metadata::new(),
                        };
                        return Ok(StaticCredentialConfiguration {
                            inner: StaticCredentialConfigurationVariant::Oauth2AuthorizationCodeFlow(
                                creds,
                            ),
                            metadata: Metadata::new(),
                        });
                    }
                    StaticCredentialConfigurationType::Oauth2JwtBearerAssertionFlow => {
                        let creds = Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration {
                            auth_uri: "https://accounts.google.com/o/oauth2/auth"
                                .to_string(),
                            token_uri: "https://oauth2.googleapis.com/token".to_string(),
                            userinfo_uri: "https://www.googleapis.com/oauth2/v3/userinfo"
                                .to_string(),
                            jwks_uri: "https://www.googleapis.com/oauth2/v3/jwks"
                                .to_string(),
                            issuer: "https://accounts.google.com".to_string(),
                            scopes: <[_]>::into_vec(
                                ::alloc::boxed::box_new([
                                    "https://www.googleapis.com/auth/gmail.readonly".to_string(),
                                ]),
                            ),
                            metadata: Metadata::new(),
                        };
                        return Ok(StaticCredentialConfiguration {
                            inner: StaticCredentialConfigurationVariant::Oauth2JwtBearerAssertionFlow(
                                creds,
                            ),
                            metadata: Metadata::new(),
                        });
                    }
                    _ => {
                        Err(CommonError::InvalidRequest {
                            msg: "No static credentials configured for google_mail"
                                .into(),
                            source: None,
                        })
                    }
                }
            }
            fn id() -> String {
                "google_mail".to_string()
            }
            fn name() -> String {
                "Google Mail".to_string()
            }
            fn documentation_url() -> String {
                "https://developers.google.com/gmail/api/guides/concepts".to_string()
            }
        }
        #[serde(tag = "type", rename_all = "snake_case")]
        pub enum GoogleMailVariant {
            Oauth2AuthorizationCodeFlow(Oauth2AuthorizationCodeFlowFullCredential),
            Oauth2JwtBearerAssertionFlow(Oauth2JwtBearerAssertionFlowFullCredential),
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for GoogleMailVariant {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private228::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    match *self {
                        GoogleMailVariant::Oauth2AuthorizationCodeFlow(ref __field0) => {
                            _serde::__private228::ser::serialize_tagged_newtype(
                                __serializer,
                                "GoogleMailVariant",
                                "Oauth2AuthorizationCodeFlow",
                                "type",
                                "oauth2_authorization_code_flow",
                                __field0,
                            )
                        }
                        GoogleMailVariant::Oauth2JwtBearerAssertionFlow(ref __field0) => {
                            _serde::__private228::ser::serialize_tagged_newtype(
                                __serializer,
                                "GoogleMailVariant",
                                "Oauth2JwtBearerAssertionFlow",
                                "type",
                                "oauth2_jwt_bearer_assertion_flow",
                                __field0,
                            )
                        }
                    }
                }
            }
        };
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for GoogleMailVariant {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private228::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "variant identifier",
                            )
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private228::Ok(__Field::__field0),
                                1u64 => _serde::__private228::Ok(__Field::__field1),
                                _ => {
                                    _serde::__private228::Err(
                                        _serde::de::Error::invalid_value(
                                            _serde::de::Unexpected::Unsigned(__value),
                                            &"variant index 0 <= i < 2",
                                        ),
                                    )
                                }
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "oauth2_authorization_code_flow" => {
                                    _serde::__private228::Ok(__Field::__field0)
                                }
                                "oauth2_jwt_bearer_assertion_flow" => {
                                    _serde::__private228::Ok(__Field::__field1)
                                }
                                _ => {
                                    _serde::__private228::Err(
                                        _serde::de::Error::unknown_variant(__value, VARIANTS),
                                    )
                                }
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"oauth2_authorization_code_flow" => {
                                    _serde::__private228::Ok(__Field::__field0)
                                }
                                b"oauth2_jwt_bearer_assertion_flow" => {
                                    _serde::__private228::Ok(__Field::__field1)
                                }
                                _ => {
                                    let __value = &_serde::__private228::from_utf8_lossy(
                                        __value,
                                    );
                                    _serde::__private228::Err(
                                        _serde::de::Error::unknown_variant(__value, VARIANTS),
                                    )
                                }
                            }
                        }
                    }
                    #[automatically_derived]
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private228::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    const VARIANTS: &'static [&'static str] = &[
                        "oauth2_authorization_code_flow",
                        "oauth2_jwt_bearer_assertion_flow",
                    ];
                    let (__tag, __content) = _serde::Deserializer::deserialize_any(
                        __deserializer,
                        _serde::__private228::de::TaggedContentVisitor::<
                            __Field,
                        >::new("type", "internally tagged enum GoogleMailVariant"),
                    )?;
                    let __deserializer = _serde::__private228::de::ContentDeserializer::<
                        __D::Error,
                    >::new(__content);
                    match __tag {
                        __Field::__field0 => {
                            _serde::__private228::Result::map(
                                <Oauth2AuthorizationCodeFlowFullCredential as _serde::Deserialize>::deserialize(
                                    __deserializer,
                                ),
                                GoogleMailVariant::Oauth2AuthorizationCodeFlow,
                            )
                        }
                        __Field::__field1 => {
                            _serde::__private228::Result::map(
                                <Oauth2JwtBearerAssertionFlowFullCredential as _serde::Deserialize>::deserialize(
                                    __deserializer,
                                ),
                                GoogleMailVariant::Oauth2JwtBearerAssertionFlow,
                            )
                        }
                    }
                }
            }
        };
        #[serde(transparent)]
        pub struct GoogleMailInstance(pub GoogleMailVariant);
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for GoogleMailInstance {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private228::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    _serde::Serialize::serialize(&self.0, __serializer)
                }
            }
        };
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for GoogleMailInstance {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private228::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::__private228::Result::map(
                        _serde::Deserialize::deserialize(__deserializer),
                        |__transparent| GoogleMailInstance {
                            0: __transparent,
                        },
                    )
                }
            }
        };
    }
    pub const MAIL_CATEGORY: &str = "mail";
    pub enum ProviderController {
        GoogleMail(google_mail::GoogleMailController),
    }
}
