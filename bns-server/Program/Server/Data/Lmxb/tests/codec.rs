use lmxb::{
    decode, element_count, encode, to_xml_string, Document, Element, Node, Text, MAX_DEPTH,
};

fn element(name: &str, children: Vec<Node>) -> Element {
    Element {
        name_flag: 1,
        name: name.into(),
        attributes: Vec::new(),
        source_line: 1,
        children,
    }
}
fn text(value: &str) -> Node {
    Node::Text(Text {
        name_flag: 1,
        name: "text".into(),
        source_line: 2,
        value: value.into(),
    })
}
fn sample() -> Document {
    let mut root = element(
        "table",
        vec![
            text("\n\t"),
            Node::Element(element("record", vec![text("한글🦀 & < >\r")])),
            text("\n"),
        ],
    );
    root.attributes = vec![
        ("empty".into(), "".into()),
        ("value".into(), "\"'&<>\t\r\n".into()),
    ];
    Document {
        reserved: [0; 64],
        source_flag: 1,
        source: "generated\\sample.xml".into(),
        root,
    }
}
#[test]
fn generated_round_trip() {
    let doc = sample();
    let bytes = encode(&doc);
    assert_eq!(&bytes[..8], b"LMXBOSLB");
    assert_eq!(
        u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize,
        bytes.len()
    );
    let decoded = decode(&bytes).unwrap();
    assert_eq!(decoded, doc);
    assert_eq!(encode(&decoded), bytes);
    assert_eq!(element_count(&decoded), 2);
    assert_eq!(to_xml_string(&doc).unwrap(), "<table empty=\"\" value=\"&quot;'&amp;&lt;&gt;&#9;&#13;&#10;\">\n\t<record>한글🦀 &amp; &lt; &gt;&#13;</record>\n</table>");
}
#[test]
fn generated_wire_layout_matches_spec() {
    let doc = Document {
        reserved: [0; 64],
        source_flag: 1,
        source: String::new(),
        root: element("r", vec![]),
    };
    let bytes = encode(&doc);
    assert_eq!(bytes.len(), 104);
    assert_eq!(&bytes[8..16], &[3, 0, 0, 0, 104, 0, 0, 0]);
    assert_eq!(&bytes[16..80], &[0; 64]);
    assert_eq!(
        &bytes[80..],
        &[
            1, // source flag
            0, 0, 0, 0, // empty source string
            0, 0, 0, 0, // no attributes; root has no kind
            1, // name flag
            1, 0, 0, 0, // one UTF-16 unit
            0xd6, 0x9f, // UTF-16LE 'r' XOR first key pair
            0, 0, 0, 0, // no children
            1, 0, 0, 0, // source line
        ]
    );
    assert_eq!(decode(&bytes).unwrap(), doc);
}
#[test]
fn empty_root_and_opaque_metadata() {
    let mut doc = sample();
    doc.root = element("root", vec![]);
    doc.source.clear();
    doc.reserved = [0x7a; 64];
    doc.source_flag = 0xa5;
    doc.root.name_flag = 0x42;
    let b = encode(&doc);
    assert_eq!(decode(&b).unwrap(), doc);
    assert_eq!(encode(&decode(&b).unwrap()), b);
    assert_eq!(to_xml_string(&doc).unwrap(), "<root/>");
}
fn repair_size(b: &mut [u8]) {
    let n = b.len() as u32;
    if b.len() >= 16 {
        b[12..16].copy_from_slice(&n.to_le_bytes());
    }
}
#[test]
fn every_truncation_rejected_without_panic() {
    let b = encode(&sample());
    for i in 0..b.len() {
        assert!(decode(&b[..i]).is_err(), "prefix {i}");
        let mut prefix = b[..i].to_vec();
        repair_size(&mut prefix);
        assert!(decode(&prefix).is_err(), "repaired prefix {i}");
    }
}
#[test]
fn garbled_generated_input_never_panics() {
    let original = encode(&sample());
    for i in 0..original.len() {
        for mask in [1, 0x80, 0xff] {
            let mut b = original.clone();
            b[i] ^= mask;
            if let Ok(doc) = decode(&b) {
                assert_eq!(encode(&doc), b);
            }
        }
    }
    let mut state = 0x123456789abcdefu64;
    for _ in 0..4096 {
        let mut b = original.clone();
        for _ in 0..8 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let i = state as usize % b.len();
            b[i] = (state >> 32) as u8;
        }
        if let Ok(doc) = decode(&b) {
            assert_eq!(encode(&doc), b);
        }
    }
}
#[test]
fn bad_counts_utf16_kind_and_trailing_bytes() {
    let mut doc = sample();
    doc.source = "x".into();
    doc.root = element("root", vec![text("x")]);
    let b = encode(&doc);
    for (offset, value) in [
        (8, 4u32),
        (81, u32::MAX),
        (87, u32::MAX),
        (104, u32::MAX),
        (112, 99u32),
    ] {
        let mut bad = b.clone();
        bad[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(decode(&bad).is_err(), "offset {offset}");
    }
    let mut bad = b.clone();
    bad[85] = 0xa4;
    bad[86] = 0x47; // XOR decodes to lone high surrogate D800.
    assert!(decode(&bad).unwrap_err().message.contains("UTF-16"));
    let mut bad = b.clone();
    bad.push(0);
    repair_size(&mut bad);
    assert!(decode(&bad).is_err());
    // Text's child count: kind at 112, value at 116, name flag 122,
    // name at 123 (4 + 8 bytes), children at 135.
    let mut bad = b;
    bad[135..139].copy_from_slice(&1u32.to_le_bytes());
    assert_eq!(decode(&bad).unwrap_err().message, "text node has children");
}
#[test]
fn depth_limit_is_checked() {
    let mut root = element("e", vec![]);
    for _ in 1..MAX_DEPTH {
        root = element("e", vec![Node::Element(root)]);
    }
    let mut doc = Document {
        reserved: [0; 64],
        source_flag: 1,
        source: String::new(),
        root,
    };
    assert!(decode(&encode(&doc)).is_ok());
    doc.root = element("e", vec![Node::Element(doc.root)]);
    assert_eq!(
        decode(&encode(&doc)).unwrap_err().message,
        "node depth limit"
    );
}
#[test]
fn invalid_xml_is_reported() {
    let mut doc = sample();
    doc.root.name = "bad name".into();
    assert!(to_xml_string(&doc).is_err());
    doc.root.name = "root".into();
    doc.root.attributes.push(doc.root.attributes[0].clone());
    assert!(to_xml_string(&doc).is_err());
    doc.root.attributes.clear();
    doc.root.children = vec![text("\0")];
    assert!(to_xml_string(&doc).is_err());
}
#[test]
fn many_generated_shapes() {
    for width in 0..64 {
        let mut doc = sample();
        doc.root.children = (0..width)
            .map(|i| {
                let mut e = element("item", vec![text(&format!("value {i} 🦀"))]);
                e.source_line = i + 1;
                e.attributes.push(("id".into(), i.to_string()));
                Node::Element(e)
            })
            .collect();
        let b = encode(&doc);
        assert_eq!(encode(&decode(&b).unwrap()), b);
        assert_eq!(element_count(&doc), width as usize + 1);
    }
}
