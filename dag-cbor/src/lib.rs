use std::collections::BTreeMap;
use std::fmt;

use serde::{de, ser, Deserialize};
use serde_bytes;
use serde_cbor::tags::{current_cbor_tag, Tagged};

const CBOR_TAG_CID: u64 = 42;

pub fn encode(ipld: &Ipld) -> Result<Vec<u8>, serde_cbor::Error> {
    serde_cbor::to_vec(&ipld)
}

pub fn decode(data: &[u8]) -> Result<Ipld, serde_cbor::Error> {
    serde_cbor::from_slice(&data)
}

struct IpldVisitor;

impl<'de> de::Visitor<'de> for IpldVisitor {
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
        let mut vec = Vec::new();

        while let Some(elem) = visitor.next_element()? {
            vec.push(elem);
        }

        Ok(Ipld::List(vec))
    }

    #[inline]
    fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: de::MapAccess<'de>,
    {
        let mut values = BTreeMap::new();

        while let Some((key, value)) = visitor.next_entry()? {
            values.insert(key, value);
        }

        Ok(Ipld::Map(values))
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
                let link = match Ipld::deserialize(deserializer) {
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

#[derive(Debug, Clone, PartialEq)]
pub enum Ipld {
    Null,
    Bool(bool),
    Integer(i128),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    List(Vec<Ipld>),
    Map(BTreeMap<String, Ipld>),
    Link(Vec<u8>),
}

impl ser::Serialize for Ipld {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match &self {
            Ipld::Null => ser.serialize_none(),
            Ipld::Bool(bool) => ser.serialize_bool(*bool),
            Ipld::Integer(i128) => ser.serialize_i128(*i128),
            Ipld::Float(f64) => ser.serialize_f64(*f64),
            Ipld::String(string) => ser.serialize_str(string),
            Ipld::Bytes(bytes) => ser.serialize_bytes(bytes),
            Ipld::List(list) => ser.collect_seq(list),
            Ipld::Map(map) => ser.collect_map(map),
            Ipld::Link(link) => {
                let value = serde_bytes::Bytes::new(&link);
                Tagged::new(Some(CBOR_TAG_CID), &value).serialize(ser)
            }
        }
    }
}

impl<'de> de::Deserialize<'de> for Ipld {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(IpldVisitor)
    }
}