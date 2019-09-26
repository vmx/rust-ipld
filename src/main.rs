use serde::de;
use serde::ser;
use serde::{Deserialize, Serialize};

use serde_bytes;

use serde_cbor;

use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, PartialEq)]
struct Cid(Vec<u8>);

impl ser::Serialize for Cid {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let tag = 42u64;
        let value = serde_bytes::ByteBuf::from(&self.0[..]);
        s.serialize_newtype_struct(serde_cbor::CBOR_TAG_STRUCT_NAME, &(tag, value))
    }
}

struct CidVisitor;

impl<'de> de::Visitor<'de> for CidVisitor {
    type Value = Cid;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "a sequence of tag and value")
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, self)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let tag: u64 = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let value: serde_cbor::Value = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;

        match (tag, value) {
            // Only return the value if tag and value type match
            (42, serde_cbor::Value::Bytes(bytes)) => Ok(Cid(bytes)),
            _ => {
                let error = format!("tag: {:?}", tag);
                let unexpected = de::Unexpected::Other(&error);
                Err(de::Error::invalid_value(unexpected, &self))
            }
        }
    }
}

impl<'de> de::Deserialize<'de> for Cid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let visitor = CidVisitor;
        deserializer.deserialize_newtype_struct(serde_cbor::CBOR_TAG_STRUCT_NAME, visitor)
    }
}

#[derive(Debug, Clone)]
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
        struct TagValueVisitor;
        impl<'de> de::Visitor<'de> for TagValueVisitor {
            type Value = Ipld;

            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("any valid CBOR tag dklsfjskldjf")
            }

            #[inline]
            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                let tag: u64 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let value: Ipld = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                match tag {
                    // It's a link
                    42 => match value {
                        Ipld::Bytes(data) => Ok(Ipld::Link(data)),
                        _ => Err(de::Error::custom(format!(
                            "Expected Bytes, found {:?}.",
                            value
                        ))),
                    },
                    // It's some other tag, so just return its value
                    _ => Ok(value),
                }
            }
        }

        deserializer.deserialize_tuple(2, TagValueVisitor)
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

#[derive(Debug, Serialize, Deserialize)]
struct Contact {
    name: String,
    details: Cid,
}

fn main() {
    let contact = Contact {
        name: "Hello World!".to_string(),
        details: Cid(vec![7, 8, 9]),
    };
    println!("Contact: {:?}", contact);
    let contact_encoded = serde_cbor::to_vec(&contact).unwrap();
    println!("Encoded contact: {:?}", contact_encoded);
    let contact_decoded_to_struct: Contact = serde_cbor::from_slice(&contact_encoded).unwrap();
    println!(
        "Decoded contact to original struct: {:?}",
        contact_decoded_to_struct
    );
    let contact_decoded_to_ipld: Ipld = serde_cbor::from_slice(&contact_encoded).unwrap();
    println!("Decoded contact to IPLD: {:?}", contact_decoded_to_ipld);
}
