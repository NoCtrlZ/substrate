// This file is part of Substrate.

// Copyright (C) 2017-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Storage types to use, implements storage traits such as StorageMap and others.
//! These types needs a prefix to be given, this prefix can be generated by `#[pallet]` macro
//! with `#[pallet::storage]` attribute (see macro documentation for usage).

use codec::{FullEncode, FullCodec};
use crate::traits::{GetDefault, StorageInstance};
use frame_metadata::{DefaultByte, DefaultByteGetter, StorageEntryModifier};

/// Trait implementing how the storage optional value is converted into the queried type.
/// It is implemented by:
/// * `OptionQuery` which convert an optional value to an optional value, user when querying
///   storage will get an optional value.
/// * `ValueQuery` which convert an optional value to a value, user when querying storage will get
///   a value.
pub trait QueryKindTrait<Value> {
	/// Metadata for the storage kind.
	const METADATA: StorageEntryModifier;

	/// Type returned on query
	type Query: FullCodec + 'static;

	/// Convert an optional value (i.e. some if trie contains the value or none otherwise) to the
	/// query.
	fn from_optional_value_to_query<OnEmpty>(v: Option<Value>) -> Self::Query where
		OnEmpty: crate::traits::Get<Self::Query>;

	/// Convert a query to an optional value.
	fn from_query_to_optional_value(v: Self::Query) -> Option<Value>;
}

/// Implement QueryKindTrait with query being `Option<Value>`
pub struct OptionQuery;
impl<Value: FullCodec + 'static> QueryKindTrait<Value> for OptionQuery where
{
	const METADATA: StorageEntryModifier = StorageEntryModifier::Optional;
	type Query = Option<Value>;
	fn from_optional_value_to_query<OnEmpty>(v: Option<Value>) -> Self::Query where
		OnEmpty: crate::traits::Get<Self::Query>
	{
		if v.is_none() {
			OnEmpty::get()
		} else {
			v
		}
	}
	fn from_query_to_optional_value(v: Self::Query) -> Option<Value> {
		v
	}
}

/// Implement QueryKindTrait with query being `Value`
pub struct ValueQuery;
impl<Value: FullCodec + 'static> QueryKindTrait<Value> for ValueQuery where
{
	const METADATA: StorageEntryModifier = StorageEntryModifier::Default;
	type Query = Value;
	fn from_optional_value_to_query<OnEmpty>(v: Option<Value>) -> Self::Query where
		OnEmpty: crate::traits::Get<Self::Query>
	{
		v.unwrap_or_else(|| OnEmpty::get())
	}
	fn from_query_to_optional_value(v: Self::Query) -> Option<Value> {
		Some(v)
	}
}

/// A type that implements StorageValue when generics are correctly set:
/// * Prefix must implement StorageInstance, a ready-to-use structure implementing StorageInstance
///   is generated by `#[pallet::storage]` in pallet macro.
/// * Value must implement FullCodec
/// * QueryKind must implmeent QueryKindTrait
/// * OnEmpty must implement Get<QueryKind::Query> + 'static
///
/// By default query kind is OptionQuery and OnEmpty returns Default Query (i.e. None for
/// OptionQuery or default value for ValueQuery).
pub struct StorageValueType<Prefix, Value, QueryKind=OptionQuery, OnEmpty=GetDefault>(
	core::marker::PhantomData<(Prefix, Value, QueryKind, OnEmpty)>
);

impl<Prefix, Value, QueryKind, OnEmpty> super::generator::StorageValue<Value> for
	StorageValueType<Prefix, Value, QueryKind, OnEmpty>
where
	Prefix: StorageInstance,
	Value: FullCodec,
	QueryKind: QueryKindTrait<Value>,
	OnEmpty: crate::traits::Get<QueryKind::Query> + 'static,
{
	type Query = QueryKind::Query;
	fn module_prefix() -> &'static [u8] {
		<Prefix::I as crate::traits::Instance>::PREFIX.as_bytes()
	}
	fn storage_prefix() -> &'static [u8] {
		Prefix::STORAGE_PREFIX.as_bytes()
	}
	fn from_optional_value_to_query(v: Option<Value>) -> Self::Query {
		QueryKind::from_optional_value_to_query::<OnEmpty>(v)
	}
	fn from_query_to_optional_value(v: Self::Query) -> Option<Value> {
		QueryKind::from_query_to_optional_value(v)
	}
}

/// A type that implements StorageMap when generics are correctly set:
/// * Prefix must implement StorageInstance, a ready-to-use structure implementing StorageInstance
///   is generated by `#[pallet::storage]` in pallet macro.
/// * Hasher must implement StorageHasher
/// * Key must implement FullCodec
/// * Value must implement FullCodec
/// * QueryKind must implmeent QueryKindTrait
/// * OnEmpty must implement Get<QueryKind::Query> + 'static
///
/// By default query kind is OptionQuery and OnEmpty returns Default Query (i.e. None for
/// OptionQuery or default value for ValueQuery).
pub struct StorageMapType<Prefix, Hasher, Key, Value, QueryKind=OptionQuery, OnEmpty=GetDefault>(
	core::marker::PhantomData<(Prefix, Hasher, Key, Value, QueryKind, OnEmpty)>
);

impl<Prefix, Hasher, Key, Value, QueryKind, OnEmpty> super::generator::StorageMap<Key, Value> for
	StorageMapType<Prefix, Hasher, Key, Value, QueryKind, OnEmpty>
where
	Prefix: StorageInstance,
	Hasher: crate::hash::StorageHasher,
	Key: FullEncode,
	Value: FullCodec,
	QueryKind: QueryKindTrait<Value>,
	OnEmpty: crate::traits::Get<QueryKind::Query> + 'static,
{
	type Query = QueryKind::Query;
	type Hasher = Hasher;
	fn module_prefix() -> &'static [u8] {
		<Prefix::I as crate::traits::Instance>::PREFIX.as_bytes()
	}
	fn storage_prefix() -> &'static [u8] {
		Prefix::STORAGE_PREFIX.as_bytes()
	}
	fn from_optional_value_to_query(v: Option<Value>) -> Self::Query {
		QueryKind::from_optional_value_to_query::<OnEmpty>(v)
	}
	fn from_query_to_optional_value(v: Self::Query) -> Option<Value> {
		QueryKind::from_query_to_optional_value(v)
	}
}

/// A type that implements StorageDoubleMap when generics are correctly set:
/// * Prefix must implement StorageInstance, a ready-to-use structure implementing StorageInstance
///   is generated by `#[pallet::storage]` in pallet macro.
/// * Hasher1 must implement StorageHasher
/// * Hasher2 must implement StorageHasher
/// * Key1 must implement FullCodec
/// * Key2 must implement FullCodec
/// * Value must implement FullCodec
/// * QueryKind must implmeent QueryKindTrait
/// * OnEmpty must implement Get<QueryKind::Query> + 'static
///
/// By default query kind is OptionQuery and OnEmpty returns Default Query (i.e. None for
/// OptionQuery or default value for ValueQuery).
pub struct StorageDoubleMapType<
	Prefix, Hasher1, Key1, Hasher2, Key2, Value, QueryKind=OptionQuery, OnEmpty=GetDefault
>(
	core::marker::PhantomData<(Prefix, Hasher1, Key1, Hasher2, Key2, Value, QueryKind, OnEmpty)>
);

impl<Prefix, Hasher1, Key1, Hasher2, Key2, Value, QueryKind, OnEmpty>
	super::generator::StorageDoubleMap<Key1, Key2, Value> for
	StorageDoubleMapType<Prefix, Hasher1, Key1, Hasher2, Key2, Value, QueryKind, OnEmpty>
where
	Prefix: StorageInstance,
	Hasher1: crate::hash::StorageHasher,
	Hasher2: crate::hash::StorageHasher,
	Key1: FullEncode,
	Key2: FullEncode,
	Value: FullCodec,
	QueryKind: QueryKindTrait<Value>,
	OnEmpty: crate::traits::Get<QueryKind::Query> + 'static
{
	type Query = QueryKind::Query;
	type Hasher1 = Hasher1;
	type Hasher2 = Hasher2;
	fn module_prefix() -> &'static [u8] {
		<Prefix::I as crate::traits::Instance>::PREFIX.as_bytes()
	}
	fn storage_prefix() -> &'static [u8] {
		Prefix::STORAGE_PREFIX.as_bytes()
	}
	fn from_optional_value_to_query(v: Option<Value>) -> Self::Query {
		QueryKind::from_optional_value_to_query::<OnEmpty>(v)
	}
	fn from_query_to_optional_value(v: Self::Query) -> Option<Value> {
		QueryKind::from_query_to_optional_value(v)
	}
}

/// Part of storage metadata for storage value.
pub trait StorageValueMetadata {
	const MODIFIER: StorageEntryModifier;
	const NAME: &'static str;
	const DEFAULT: DefaultByteGetter;
}

struct OnEmptyGetter<Value, OnEmpty>(core::marker::PhantomData<(Value, OnEmpty)>);
impl<Value: FullCodec, OnEmpty: crate::traits::Get<Value>> DefaultByte for OnEmptyGetter<Value, OnEmpty> {
	fn default_byte(&self) -> sp_std::vec::Vec<u8> {
		OnEmpty::get().encode()
	}
}
unsafe impl <Value, OnEmpty: crate::traits::Get<Value>> Send for OnEmptyGetter<Value, OnEmpty> {}
unsafe impl <Value, OnEmpty: crate::traits::Get<Value>> Sync for OnEmptyGetter<Value, OnEmpty> {}

impl<Prefix, Value, QueryKind, OnEmpty> StorageValueMetadata
	for StorageValueType<Prefix, Value, QueryKind, OnEmpty> where
	Prefix: StorageInstance,
	Value: FullCodec,
	QueryKind: QueryKindTrait<Value>,
	OnEmpty: crate::traits::Get<QueryKind::Query> + 'static,
{
	const MODIFIER: StorageEntryModifier = QueryKind::METADATA;
	const NAME: &'static str = Prefix::STORAGE_PREFIX;
	const DEFAULT: DefaultByteGetter =
		DefaultByteGetter(&OnEmptyGetter::<QueryKind::Query, OnEmpty>(core::marker::PhantomData));
}

/// Part of storage metadata for a storage map.
///
/// NOTE: Generic hasher is supported.
pub trait StorageMapMetadata {
	const MODIFIER: StorageEntryModifier;
	const NAME: &'static str;
	const DEFAULT: DefaultByteGetter;
	const HASHER: frame_metadata::StorageHasher;
}

impl<Prefix, Hasher, Key, Value, QueryKind, OnEmpty> StorageMapMetadata
	for StorageMapType<Prefix, Hasher, Key, Value, QueryKind, OnEmpty> where
	Prefix: StorageInstance,
	Hasher: crate::hash::StorageHasher,
	Key: FullEncode,
	Value: FullCodec,
	QueryKind: QueryKindTrait<Value>,
	OnEmpty: crate::traits::Get<QueryKind::Query> + 'static,
{
	const MODIFIER: StorageEntryModifier = QueryKind::METADATA;
	const HASHER: frame_metadata::StorageHasher = Hasher::METADATA;
	const NAME: &'static str = Prefix::STORAGE_PREFIX;
	const DEFAULT: DefaultByteGetter =
		DefaultByteGetter(&OnEmptyGetter::<QueryKind::Query, OnEmpty>(core::marker::PhantomData));
}

/// Part of storage metadata for a storage double map.
///
/// NOTE: Generic hashers is supported.
pub trait StorageDoubleMapMetadata {
	const MODIFIER: StorageEntryModifier;
	const NAME: &'static str;
	const DEFAULT: DefaultByteGetter;
	const HASHER1: frame_metadata::StorageHasher;
	const HASHER2: frame_metadata::StorageHasher;
}

impl<Prefix, Hasher1, Hasher2, Key1, Key2, Value, QueryKind, OnEmpty> StorageDoubleMapMetadata
	for StorageDoubleMapType<Prefix, Hasher1, Key1, Hasher2, Key2, Value, QueryKind, OnEmpty> where
	Prefix: StorageInstance,
	Hasher1: crate::hash::StorageHasher,
	Hasher2: crate::hash::StorageHasher,
	Key1: FullEncode,
	Key2: FullEncode,
	Value: FullCodec,
	QueryKind: QueryKindTrait<Value>,
	OnEmpty: crate::traits::Get<QueryKind::Query> + 'static
{
	const MODIFIER: StorageEntryModifier = QueryKind::METADATA;
	const HASHER1: frame_metadata::StorageHasher = Hasher1::METADATA;
	const HASHER2: frame_metadata::StorageHasher = Hasher2::METADATA;
	const NAME: &'static str = Prefix::STORAGE_PREFIX;
	const DEFAULT: DefaultByteGetter =
		DefaultByteGetter(&OnEmptyGetter::<QueryKind::Query, OnEmpty>(core::marker::PhantomData));
}
