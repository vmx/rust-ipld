use std::collections::BTreeMap;

use ipld_core::Ipld;
use ipld_dag_json;

#[test]
fn encode_struct() {
  // Create a contact object that looks like:
  // Contact { name: "Hello World", details: CID }
  let mut map = BTreeMap::new();
  map.insert("name".to_string(), Ipld::String("Hello World!".to_string()));
  map.insert("details".to_string(), Ipld::Link(vec![7, 8, 9]));
  let contact = Ipld::Map(map);

  let contact_encoded = ipld_dag_json::encode(&contact).unwrap();
  println!("encoded: {:02x?}", contact_encoded);
  println!(
    "encoded string {}",
    std::str::from_utf8(&contact_encoded).unwrap()
  );

  assert_eq!(
    std::str::from_utf8(&contact_encoded).unwrap(),
    r#"{"details":{"/":"BwgJ"},"name":"Hello World!"}"#
  );

  let contact_decoded: Ipld = ipld_dag_json::decode(&contact_encoded).unwrap();
  assert_eq!(contact_decoded, contact);
}
