use base64;
use ipld_core::Ipld;
use serde::ser::{self, Serialize};
use serde_json;
use std::collections::BTreeMap;

pub enum EncodeError {
    Error,
}

pub fn encode(ipld: &Ipld) -> Result<Vec<u8>, serde_json::Error> {
    let mut writer = Vec::with_capacity(128);
    let mut ser = serde_json::ser::Serializer::new(&mut writer);
    serialize(&ipld, &mut ser)?;
    Ok(writer)
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
            let value = base64::encode(&link);
            let mut map = BTreeMap::new();
            map.insert("/", value);

            ser.collect_map(map)
        }
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
