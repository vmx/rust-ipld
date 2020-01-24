use std::collections::BTreeMap;
use std::fmt;
use std::iter::FromIterator;

use serde::{de, ser, Deserialize, Serialize};
use serde_bytes;
use serde_cbor::tags::{current_cbor_tag, Tagged};

use ipld_core::Ipld;

const CBOR_TAG_CID: u64 = 42;

pub fn encode(ipld: &Ipld) -> Result<Vec<u8>, serde_cbor::Error> {
    let mut vec = Vec::new();
    let mut ser = serde_cbor::Serializer::new(&mut vec);
    serialize(&ipld, &mut ser)?;
    Ok(vec)
}

pub fn decode(data: &[u8]) -> Result<Ipld, serde_cbor::Error> {
    let mut de = serde_cbor::Deserializer::from_slice(&data);
    deserialize(&mut de)
}

// Needed for `visit_seq` and `visit_map` in Deserializer
/// We cannot directly implement `serde::Deserializer` for `Ipld` as it is a remote type.
/// Instead wrap it into a newtype struct and implement `serde::Deserialize` for that one.
/// All the deserializer does is calling the `deserialize()` function we defined which returns
/// an unwrapped `Ipld` instance. Wrap that `Ipld` instance in `Wrapper` and return it.
/// Users of this wrapper will then unwrap it again so that they can return the expected `Ipld`
/// instance.
struct WrapperOwned(Ipld);
impl<'de> Deserialize<'de> for WrapperOwned {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let deserialized = deserialize(deserializer);
        // Better version of Ok(Wrapper(deserialized.unwrap()))
        deserialized.map(Self)
    }
}
// Needed for `collect_seq` and `collect_map` in Deserializer
struct Wrapper<'a>(&'a Ipld);
impl<'a> Serialize for Wrapper<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serialize(&self.0, serializer)
    }
}

struct IpldCborVisitor;
impl<'de> de::Visitor<'de> for IpldCborVisitor {
    type Value = Ipld;

    fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("any valid CBOR value")
    }

    #[inline]
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(String::from(value))
    }

    #[inline]
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::String(value))
    }
    #[inline]
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_byte_buf(v.to_owned())
    }

    #[inline]
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Bytes(v))
    }

    #[inline]
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Integer(v.into()))
    }

    #[inline]
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Integer(v.into()))
    }

    #[inline]
    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Integer(v))
    }

    #[inline]
    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Bool(v))
    }

    #[inline]
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_unit()
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Null)
    }

    #[inline]
    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: de::SeqAccess<'de>,
    {
        let mut vec: Vec<WrapperOwned> = Vec::new();

        while let Some(elem) = visitor.next_element()? {
            vec.push(elem);
        }

        let unwrapped = vec.into_iter().map(|WrapperOwned(ipld)| ipld).collect();
        Ok(Ipld::List(unwrapped))
    }

    #[inline]
    fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: de::MapAccess<'de>,
    {
        let mut values: Vec<(String, WrapperOwned)> = Vec::new();

        while let Some((key, value)) = visitor.next_entry()? {
            values.push((key, value));
        }

        let unwrapped = values
            .into_iter()
            .map(|(key, WrapperOwned(value))| (key, value));
        Ok(Ipld::Map(BTreeMap::from_iter(unwrapped)))
    }

    #[inline]
    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ipld::Float(v))
    }

    #[inline]
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        match current_cbor_tag() {
            Some(CBOR_TAG_CID) => {
                let link = match deserialize(deserializer) {
                    Ok(Ipld::Bytes(link)) => link,
                    _ => return Err(de::Error::custom("bytes expected")),
                };
                Ok(Ipld::Link(link))
            }
            Some(tag) => Err(de::Error::custom(format!("unexpected tag ({})", tag))),
            _ => Err(de::Error::custom("tag expected")),
        }
    }
}

fn serialize<S>(ipld: &Ipld, ser: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    match &ipld {
        Ipld::Null => ser.serialize_none(),
        Ipld::Bool(bool) => ser.serialize_bool(*bool),
        Ipld::Integer(i128) => ser.serialize_i128(*i128),
        Ipld::Float(f64) => ser.serialize_f64(*f64),
        Ipld::String(string) => ser.serialize_str(&string),
        Ipld::Bytes(bytes) => ser.serialize_bytes(&bytes),
        Ipld::List(list) => {
            let wrapped = list.iter().map(|ipld| Wrapper(ipld));
            ser.collect_seq(wrapped)
        }
        Ipld::Map(map) => {
            let wrapped = map.iter().map(|(key, ipld)| (key, Wrapper(ipld)));
            ser.collect_map(wrapped)
        }
        Ipld::Link(link) => {
            let value = serde_bytes::Bytes::new(&link);
            Tagged::new(Some(CBOR_TAG_CID), &value).serialize(ser)
        }
    }
}

fn deserialize<'de, D>(deserializer: D) -> Result<Ipld, D::Error>
where
    D: de::Deserializer<'de>,
{
    deserializer.deserialize_any(IpldCborVisitor)
}
