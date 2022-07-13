mod circuit;
#[allow(dead_code)]
mod common;
mod sponge;
pub mod rescue_prime;
#[cfg(test)]
mod tests;
mod traits;

use std::convert::TryInto;

pub use circuit::sponge::{
    circuit_generic_hash, circuit_generic_round_function, CircuitGenericSponge, circuit_generic_round_function_conditional
};
use serde::{ser::{SerializeTuple}, Serialize};
pub use traits::{HashParams, CustomGate};
pub use sponge::{generic_hash, generic_round_function, GenericSponge};
pub use rescue_prime::{params::RescuePrimeParams, rescue_prime_hash};
pub use common::domain_strategy::DomainStrategy;


pub trait BigArraySerde<'de>: Sized {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer;
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>;
}

// some wrappers that make array wrappers serializable themselves (resursively)

pub struct BigArrayRefWrapper<'de, B: BigArraySerde<'de>>(&'de B);

impl<'de, B: BigArraySerde<'de>> serde::Serialize for BigArrayRefWrapper<'de, B> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        self.0.serialize(serializer)
    }
}

pub struct BigArrayWrapper<'de, B: BigArraySerde<'de>>(B, std::marker::PhantomData<& 'de ()>);

impl<'de, B: BigArraySerde<'de>> serde::Serialize for BigArrayWrapper<'de, B> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        self.0.serialize(serializer)
    }
}

impl<'de, B: BigArraySerde<'de>> serde::Deserialize<'de> for BigArrayWrapper<'de, B> {
fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let new = B::deserialize(deserializer)?;

        Ok(Self(new, std::marker::PhantomData))
    }
}

struct ArrayVisitor<T, const M: usize> {
    element: std::marker::PhantomData<T>,
}

impl<'de, T, const M: usize> serde::de::Visitor<'de> for ArrayVisitor<T, M>
    where T: serde::Deserialize<'de>
{
    type Value = [T; M];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(&format!("an array of length {}", M))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<[T; M], A::Error>
        where A: serde::de::SeqAccess<'de>
    {
        let mut arr = Vec::with_capacity(M);
        for i in 0..M {
            let el = seq.next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
            arr.push(el);
        }
        let arr: [T; M] = arr.try_into().map_err(|_| serde::de::Error::invalid_length(M, &self))?;

        Ok(arr)
    }
}

impl<'de, T, const N: usize> BigArraySerde<'de> for [T; N]
    where T: serde::Serialize + serde::Deserialize<'de>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        let mut seq = serializer.serialize_tuple(N)?;
        for elem in &self[..] {
            seq.serialize_element(elem)?;
        }
        seq.end()
    }

    fn deserialize<D>(deserializer: D) -> Result<[T; N], D::Error>
        where D: serde::Deserializer<'de>
    {
        let visitor = ArrayVisitor::<_, N> { element: std::marker::PhantomData };
        deserializer.deserialize_tuple(N, visitor)
    }
}

fn serialize_vec_of_arrays<T: serde::Serialize + serde::de::DeserializeOwned, const N: usize, S>(t: &Vec<[T; N]>, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
    let cast: Vec<_> = t.iter().map(|el| BigArrayRefWrapper(el)).collect();
    cast.serialize(serializer)
}

fn deserialize_vec_of_arrays<'de, D, T: serde::Serialize + serde::de::DeserializeOwned, const N: usize>(deserializer: D) -> Result<Vec<[T; N]>, D::Error> where D: serde::Deserializer<'de> {
    use serde::Deserialize;

    let result: Vec<BigArrayWrapper<'de, [T; N]>> = <Vec<BigArrayWrapper<'de, [T; N]>>>::deserialize(deserializer)?;
    let result: Vec<_> = result.into_iter().map(|el| el.0).collect();

    Ok(result)
}

fn serialize_array_of_arrays<T: serde::Serialize + serde::de::DeserializeOwned, const N: usize, const M: usize, S>(t: &[[T; N]; M], serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
    let mut seq = serializer.serialize_tuple(M)?;
    for el in t.iter() {
        let w = BigArrayRefWrapper(el);
        seq.serialize_element(&w)?;
    }

    seq.end()
}

fn deserialize_array_of_arrays<'de, D, T: serde::Serialize + serde::de::DeserializeOwned, const N: usize, const M: usize>(deserializer: D) -> Result<[[T; N]; M], D::Error> where D: serde::Deserializer<'de> {
    let visitor = ArrayVisitor::<BigArrayWrapper<'de, [T; N]>, M> { element: std::marker::PhantomData };
    let result = deserializer.deserialize_tuple(M, visitor)?;

    let subarray: [[T; N]; M] = match std::iter::IntoIterator::into_iter(result).map(|el| el.0).collect::<Vec<_>>().try_into() {
        Ok(a) => a,
        Err(..) => panic!("length must patch")
    };

    Ok(subarray)
}