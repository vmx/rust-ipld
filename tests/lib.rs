use std::collections::BTreeMap;

use ipld_dag_cbor::{Cid, Ipld};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Contact {
    name: String,
    details: Cid,
}

#[test]
fn encode_struct() {
    let contact = Contact {
        name: "Hello World!".to_string(),
        details: Cid(vec![7, 8, 9]),
    };

    let contact_encoded = serde_cbor::to_vec(&contact).unwrap();
    let expected_encoded = vec![
        0xa2, 0x64, 0x6e, 0x61, 0x6d, 0x65, 0x6c, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x57, 0x6f,
        0x72, 0x6c, 0x64, 0x21, 0x67, 0x64, 0x65, 0x74, 0x61, 0x69, 0x6c, 0x73, 0xd8, 0x2a, 0x43,
        0x07, 0x08, 0x09,
    ];
    assert_eq!(contact_encoded, expected_encoded);

    let contact_decoded: Ipld = serde_cbor::from_slice(&contact_encoded).unwrap();
    let mut expected_decoded_map = BTreeMap::new();
    expected_decoded_map.insert("details".to_string(), Ipld::Link(vec![7, 8, 9]));
    expected_decoded_map.insert("name".to_string(), Ipld::String("Hello World!".to_string()));
    let expected_decoded = Ipld::Map(expected_decoded_map);
    assert_eq!(contact_decoded, expected_decoded);
}
