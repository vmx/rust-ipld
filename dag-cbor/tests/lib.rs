use std::collections::BTreeMap;

use ipld_core::Ipld;
use ipld_dag_cbor;

#[test]
fn encode_struct() {
    // Create a contact object that looks like:
    // Contact { name: "Hello World", details: CID }
    let mut map = BTreeMap::new();
    map.insert("name".to_string(), Ipld::String("Hello World!".to_string()));
    map.insert("details".to_string(), Ipld::Link(vec![7, 8, 9]));
    let contact = Ipld::Map(map);

    let contact_encoded = ipld_dag_cbor::encode(&contact).unwrap();
    println!("encoded: {:02x?}", contact_encoded);
    let expected_encoded = vec![
        0xa2, 0x67, 0x64, 0x65, 0x74, 0x61, 0x69, 0x6c, 0x73, 0xd8, 0x2a, 0x43, 0x07, 0x08, 0x09,
        0x64, 0x6e, 0x61, 0x6d, 0x65, 0x6c, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x57, 0x6f, 0x72,
        0x6c, 0x64, 0x21,
    ];
    println!("expected: {:02x?}", expected_encoded);
    assert_eq!(contact_encoded, expected_encoded);

    let contact_decoded: Ipld = ipld_dag_cbor::decode(&contact_encoded).unwrap();
    assert_eq!(contact_decoded, contact);
}
