use super::*;
use crate::{layout::Layout, pe::Pe};
use bnsdata::tabledef::Definitions;
use serde_json::json;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

const DATA_RAW: usize = 0x400;
const DATA_RVA: usize = 0x2000;
fn put(bytes: &mut [u8], at: usize, value: u64, n: usize) {
    bytes[at..at + n].copy_from_slice(&value.to_le_bytes()[..n]);
}
struct Fake {
    bytes: Vec<u8>,
    base: u64,
    l: &'static Layout,
    cursor: usize,
    headers: usize,
    array: u64,
    table: usize,
    body: usize,
    attr: usize,
}
impl Fake {
    fn new(wide: bool, count: usize) -> Self {
        let mut bytes = vec![0; 0x4400];
        bytes[..2].copy_from_slice(b"MZ");
        put(&mut bytes, 0x3c, 0x80, 4);
        bytes[0x80..0x84].copy_from_slice(b"PE\0\0");
        put(&mut bytes, 0x84, if wide { 0x8664 } else { 0x14c }, 2);
        put(&mut bytes, 0x86, 2, 2);
        let optsz = if wide { 240 } else { 224 };
        put(&mut bytes, 0x94, optsz, 2);
        let opt = 0x98;
        put(&mut bytes, opt, if wide { 0x20b } else { 0x10b }, 2);
        put(&mut bytes, opt + 16, 0x1000, 4);
        let base = if wide { 0x140000000 } else { 0x400000 };
        put(
            &mut bytes,
            opt + if wide { 24 } else { 28 },
            base,
            if wide { 8 } else { 4 },
        );
        put(&mut bytes, opt + 32, 0x1000, 4);
        put(&mut bytes, opt + 36, 0x200, 4);
        put(&mut bytes, opt + 56, 0x6000, 4);
        put(&mut bytes, opt + 60, 0x200, 4);
        let headers = opt + optsz as usize;
        bytes[headers..headers + 5].copy_from_slice(b".text");
        put(&mut bytes, headers + 8, 0x200, 4);
        put(&mut bytes, headers + 12, 0x1000, 4);
        put(&mut bytes, headers + 16, 0x200, 4);
        put(&mut bytes, headers + 20, 0x200, 4);
        put(&mut bytes, headers + 36, 0x60000020, 4);
        let h = headers + 40;
        bytes[h..h + 5].copy_from_slice(b".data");
        put(&mut bytes, h + 8, 0x4000, 4);
        put(&mut bytes, h + 12, DATA_RVA as u64, 4);
        put(&mut bytes, h + 16, 0x4000, 4);
        put(&mut bytes, h + 20, DATA_RAW as u64, 4);
        put(&mut bytes, h + 36, 0xc0000040, 4);
        let mut f = Self {
            bytes,
            base,
            l: if wide { &layout::X64P4 } else { &layout::X86 },
            cursor: DATA_RAW,
            headers,
            array: 0,
            table: 0,
            body: 0,
            attr: 0,
        };
        let array = f.alloc((count + 2) * f.l.p);
        f.array = f.va(array);
        let name = f.string("record");
        let field = f.string("field");
        let hashname = f.string("hash-name");
        let default = f.string("Δ");
        let zero = f.string("zero");
        let one = f.string("one");
        let subname = f.string("extended");
        let nh = f.alloc(16);
        f.u(nh, 0, 0x12345678, 4);
        f.ptr(nh, 4, hashname);
        let seq = f.alloc(2 * f.l.p);
        f.ptr(seq, 0, zero);
        f.ptr(seq, f.l.p, one);
        let defaultrec = f.alloc(6);
        f.u(defaultrec, 0, 255, 1);
        f.u(defaultrec, 2, 65534, 2);
        f.u(defaultrec, 4, 24, 2);
        let attr = f.alloc(f.l.attr.stride);
        f.attr = attr;
        let a = &f.l.attr;
        f.ptr(attr, a.name, field);
        f.ptr(attr, a.namehash, nh);
        f.u(attr, a.kind, 3, 1);
        f.u(attr, a.key, 1, 1);
        f.u(attr, a.repeat, 2, 1);
        f.u(attr, a.required, 7, 1);
        f.ptr(attr, a.defvalue, default);
        f.u(attr, a.min, (-123i64) as u64, 8);
        f.u(attr, a.max, 456, 8);
        f.u(attr, a.fmin, 0.1f32.to_bits() as u64, 4);
        f.u(attr, a.fmax, 1.5f32.to_bits() as u64, 4);
        f.ptr(attr, a.seq, seq);
        f.u(attr, a.seqcount, 2, 2);
        f.u(attr, a.reftable, 65534, 2);
        f.u(attr, a.refel, 255, 1);
        f.u(attr, a.deprecated, 1, 1);
        f.u(attr, a.offset, 8, 2);
        f.u(attr, a.size, 4, 2);
        f.u(attr, a.propfield, 0x89abcdef, 4);
        let sub = f.alloc(f.l.sub.stride);
        let s = &f.l.sub;
        f.ptr(sub, s.name, subname);
        f.ptr(sub, s.attr, attr);
        f.ptr(sub, s.defrec, defaultrec);
        f.u(sub, s.attrcount, 1, 2);
        f.u(sub, s.size, 28, 2);
        let body = f.alloc(f.l.body.stride);
        f.body = body;
        let b = &f.l.body;
        f.ptr(body, b.attr, attr);
        f.ptr(body, b.sub, sub);
        f.ptr(body, b.defrec, defaultrec);
        f.u(body, b.attrcount, 1, 2);
        f.u(body, b.subcount, 1, 2);
        f.u(body, b.size, 24, 2);
        f.u(body, b.keyoffset, 8, 2);
        let el = f.alloc(f.l.el.stride);
        let e = &f.l.el;
        f.ptr(el, e.name, name);
        f.u(el, e.kind, 1, if wide { 4 } else { 1 });
        f.ptr(el, e.body, body);
        f.u(el, e.child + 1, 4, 1);
        for i in 1..=count {
            let table_name = f.string(&format!("table_{i}"));
            let td = f.alloc(48);
            if i == 1 {
                f.table = td;
            }
            let t = &f.l.td;
            f.ptr(td, t.name, table_name);
            f.u(td, t.kind, i as u64, 2);
            f.u(td, t.major, 2, 2);
            f.u(td, t.minor, 3, 2);
            f.ptr(td, t.el, el);
            f.u(td, t.elcount, 1, 1);
            // High bits must be ignored, as in exportdefs.py's u32 read.
            f.u(td, t.maxid, 0x1234567887654321, 8);
            f.u(td, t.autokey, 1, 1);
            f.u(td, t.module, 42, 4);
            f.ptr(array, i * f.l.p, td);
        }
        f
    }
    fn alloc(&mut self, n: usize) -> usize {
        self.cursor = (self.cursor + 7) & !7;
        let at = self.cursor;
        self.cursor += n;
        assert!(self.cursor <= self.bytes.len());
        at
    }
    fn va(&self, at: usize) -> u64 {
        self.base + DATA_RVA as u64 + (at - DATA_RAW) as u64
    }
    fn string(&mut self, s: &str) -> usize {
        let units: Vec<_> = s.encode_utf16().chain([0]).collect();
        let at = self.alloc(units.len() * 2);
        for (i, u) in units.iter().enumerate() {
            put(&mut self.bytes, at + i * 2, u64::from(*u), 2);
        }
        at
    }
    fn u(&mut self, at: usize, off: usize, value: u64, n: usize) {
        put(&mut self.bytes, at + off, value, n);
    }
    fn ptr(&mut self, at: usize, off: usize, target: usize) {
        self.u(at, off, self.va(target), self.l.p);
    }
    fn parse(&self) -> Pe<'_> {
        Pe::parse(&self.bytes).unwrap()
    }
}

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "bnsschema-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn expected_table() -> Value {
    let attr = json!({"name":"field", "namehash":{"namehash":0x12345678u32,"name":"hash-name"},
        "type":"int32", "key":true,"repeat":2,"required":7,"defvalue":"Δ", "min":-123,"max":456,
        "fmin":f64::from(0.1f32),"fmax":1.5,"seq":["zero","one"],"seqcount":2,"reftable":-2,
        "refel":255,"deprecated":true,"hidden":false,"offset":8,"size":4,"propfield":0x89abcdefu32});
    let record = json!({"type":-1,"subtype":-2,"size":24});
    let mut child = vec![0; 32];
    child[1] = 4;
    json!({"name":"table_1","type":1,"major_ver":2,"minor_ver":3,"maxid":0x87654321u32,"autokey":true,"module":42,
        "elcount":1,"el":[{"name":"record","type":1,"child":child,"body":{
            "attr":[attr.clone()],"sub":[{"name":"extended","attr":[attr],"defaultRecord":record.clone(),"attrcount":1,"size":28}],
            "defaultRecord":record,"attrcount":1,"subcount":1,"size":24,"keyoffset":8}}]})
}

#[test]
fn synthetic_roundtrip_and_exact_json_both_widths() {
    for wide in [false, true] {
        let fake = Fake::new(wide, 2);
        let (found, values) = analyze(&fake.bytes, Some(fake.array)).unwrap();
        assert_eq!(found.tables, 2);
        assert_eq!(found.layout, if wide { "x64p4" } else { "x86" });
        assert_eq!(
            found.width,
            if wide {
                Width32or64::Width64
            } else {
                Width32or64::Width32
            }
        );
        assert_eq!(values[0], expected_table());
        let temp = Temp::new();
        let exe = temp.0.join("fake.exe");
        fs::write(&exe, &fake.bytes).unwrap();
        let out = temp.0.join("defs");
        assert_eq!(export(&exe, Some(fake.array), &out).unwrap(), found);
        let definitions = Definitions::load_folder(&out).unwrap();
        assert_eq!(definitions.0.len(), 2);
        assert_eq!(definitions.get(1).unwrap().el[0].body.attr.len(), 1);
        assert_eq!(definitions.get(2).unwrap().el[0].body.sub[0].attr.len(), 1);
        let old = fs::read(out.join("table_1.json")).unwrap();
        assert!(
            matches!(export(&exe, Some(fake.array), &out), Err(Error::Io(e)) if e.kind() == std::io::ErrorKind::AlreadyExists)
        );
        assert_eq!(fs::read(out.join("table_1.json")).unwrap(), old);
        assert!(inspect(&exe).is_err()); // Automatic mode requires 16 tables.
    }
}

#[test]
fn automatic_scan_and_longest_run() {
    for wide in [false, true] {
        let mut fake = Fake::new(wide, 17);
        let first = fake.array;
        let longer = fake.alloc(19 * fake.l.p);
        for i in 1..=17 {
            let target = fake.parse().uint(first, i * fake.l.p, fake.l.p).unwrap();
            fake.u(longer, i * fake.l.p, target, fake.l.p);
        }
        // Put the shorter candidate first to test longest-run selection.
        fake.u(DATA_RAW, 17 * fake.l.p, 0, fake.l.p);
        let found = analyze(&fake.bytes, None).unwrap().0;
        assert_eq!((found.array_va, found.tables), (fake.va(longer), 17));
        let temp = Temp::new();
        let exe = temp.0.join("fake.exe");
        fs::write(&exe, &fake.bytes).unwrap();
        assert_eq!(inspect(&exe).unwrap(), found);
        assert_eq!(export(&exe, None, &temp.0.join("defs")).unwrap(), found);
    }
}

#[test]
fn automatic_scan_requires_initialized_non_executable_data() {
    for wide in [false, true] {
        let mut fake = Fake::new(wide, 16);
        let h = fake.headers + 40;
        for characteristics in [0xc0000000, 0xe0000040] {
            put(&mut fake.bytes, h + 36, characteristics, 4);
            assert!(matches!(
                analyze(&fake.bytes, None),
                Err(Error::NotFound(_))
            ));
            // An explicit address can refer to other section kinds, after the gate.
            assert_eq!(analyze(&fake.bytes, Some(fake.array)).unwrap().0.tables, 16);
        }
        put(&mut fake.bytes, h + 36, 0xc0000040, 4);
        fake.u(fake.table, fake.l.td.kind, 2, 2);
        assert!(matches!(
            analyze(&fake.bytes, None),
            Err(Error::NotFound(_))
        ));
        fake.u(fake.table, fake.l.td.kind, 1, 2);
        fake.u(DATA_RAW, 16 * fake.l.p, 0, fake.l.p);
        assert!(matches!(
            analyze(&fake.bytes, None),
            Err(Error::NotFound(_))
        ));
        assert_eq!(analyze(&fake.bytes, Some(fake.array)).unwrap().0.tables, 15);
    }
}

#[test]
fn protection_names_case_insensitive() {
    for name in protect::PACKER_SECTIONS {
        let mut fake = Fake::new(false, 1);
        let h = fake.headers + 40;
        fake.bytes[h..h + 8].fill(0);
        let name = name.to_ascii_uppercase();
        fake.bytes[h..h + name.len()].copy_from_slice(name.as_bytes());
        assert!(
            matches!(analyze(&fake.bytes, Some(fake.array)), Err(Error::Refused(s)) if s.starts_with("packer/protector section:"))
        );
    }
}
#[test]
fn protection_entry_rules_and_virtual_code() {
    let mut outside = Fake::new(false, 1);
    put(&mut outside.bytes, 0x98 + 16, DATA_RVA as u64, 4);
    assert_eq!(
        protect::check(&outside.parse()).unwrap_err().0,
        "entry point is not in the first executable section"
    );
    put(&mut outside.bytes, 0x98 + 16, 0x9000, 4);
    assert_eq!(
        protect::check(&outside.parse()).unwrap_err().0,
        "entry point is not in a section"
    );
    let mut writable = Fake::new(false, 1);
    put(&mut writable.bytes, writable.headers + 36, 0xe0000020, 4);
    assert_eq!(
        protect::check(&writable.parse()).unwrap_err().0,
        "entry-point section is writable"
    );
    let mut virtual_code = Fake::new(false, 1);
    put(&mut virtual_code.bytes, virtual_code.headers + 16, 0, 4);
    assert!(protect::check(&virtual_code.parse())
        .unwrap_err()
        .0
        .starts_with("executable section has no raw bytes:"));
    // A second executable section cannot hold the entry point.
    let mut second = Fake::new(false, 1);
    put(&mut second.bytes, second.headers + 40 + 36, 0x60000040, 4);
    put(&mut second.bytes, 0x98 + 16, DATA_RVA as u64, 4);
    assert!(protect::check(&second.parse()).is_err());
}
#[test]
fn protection_high_entropy_all_section_bytes() {
    let mut fake = Fake::new(true, 1);
    fake.bytes.resize(DATA_RAW + 65536, 0);
    let h = fake.headers + 40;
    put(&mut fake.bytes, h + 8, 65536, 4);
    put(&mut fake.bytes, h + 16, 65536, 4);
    for (i, b) in fake.bytes[DATA_RAW..].iter_mut().enumerate() {
        *b = i as u8;
    }
    assert!((fake.parse().sections[1].entropy - 8.0).abs() < 1e-10);
    assert!(
        matches!(analyze(&fake.bytes, None), Err(Error::Refused(s)) if s.starts_with("high-entropy section:"))
    );
    assert_eq!(pe::entropy(&[]), 0.0);
}
#[test]
fn refusal_precedes_detection_and_output() {
    let mut fake = Fake::new(false, 1);
    fake.bytes[fake.headers + 40..fake.headers + 48].copy_from_slice(b".themida");
    let temp = Temp::new();
    let exe = temp.0.join("fake.exe");
    fs::write(&exe, fake.bytes).unwrap();
    let out = temp.0.join("must-not-exist");
    assert!(matches!(
        export(&exe, Some(u64::MAX), &out),
        Err(Error::Refused(_))
    ));
    assert!(!out.exists());
    assert!(matches!(inspect(&exe), Err(Error::Refused(_))));
}
#[test]
fn unknown_types_and_null_empty_arrays() {
    let mut fake = Fake::new(true, 1);
    fake.u(fake.attr, fake.l.attr.kind, 33, 1);
    let (found, values) = analyze(&fake.bytes, Some(fake.array)).unwrap();
    assert_eq!(found.unknown_types, [33]);
    assert_eq!(values[0]["el"][0]["body"]["attr"][0]["type"], "type_33");
    fake.u(fake.body, fake.l.body.attrcount, 0, 2);
    fake.u(fake.body, fake.l.body.subcount, 0, 2);
    fake.u(fake.body, fake.l.body.defrec, 0, fake.l.p);
    let (_, values) = analyze(&fake.bytes, Some(fake.array)).unwrap();
    let body = &values[0]["el"][0]["body"];
    assert!(body["attr"].is_null());
    assert!(body["sub"].is_null());
    assert!(body["defaultRecord"].is_null());
}
#[test]
fn bounds_strings_names_counts_and_floats() {
    for wide in [false, true] {
        let mut fake = Fake::new(wide, 1);
        let pe = fake.parse();
        assert_eq!(pe.va_to_offset(fake.array), Some(DATA_RAW));
        assert_eq!(pe.va_to_offset(fake.base + DATA_RVA as u64 + 0x4000), None);
        assert!(pe.uint(u64::MAX, 1, 8).is_err());
        assert!(analyze(&fake.bytes, Some(u64::MAX)).is_err());
        fake.u(fake.body, fake.l.body.attrcount, 0xffff, 2);
        assert!(matches!(
            analyze(&fake.bytes, Some(fake.array)),
            Err(Error::Format(_))
        ));
        fake.u(fake.body, fake.l.body.attrcount, 1, 2);
        fake.u(fake.attr, fake.l.attr.fmin, f32::NAN.to_bits() as u64, 4);
        assert!(matches!(
            analyze(&fake.bytes, Some(fake.array)),
            Err(Error::Format(_))
        ));
        fake.u(fake.attr, fake.l.attr.fmin, 0, 4);
        let unsafe_name = fake.string("../bad");
        fake.ptr(fake.table, fake.l.td.name, unsafe_name);
        assert!(analyze(&fake.bytes, Some(fake.array)).is_err());
        let invalid = fake.alloc(4);
        fake.u(invalid, 0, 0xd800, 2);
        assert!(fake.parse().wstr(fake.va(invalid)).is_err());
        let long = fake.alloc(8196);
        for i in 0..4097 {
            fake.u(long, i * 2, 97, 2);
        }
        assert!(fake.parse().wstr(fake.va(long)).is_err());
        fake.u(long, 4096 * 2, 0, 2);
        assert_eq!(
            fake.parse().wstr(fake.va(long)).unwrap().unwrap().len(),
            4096
        );
        let last = fake.bytes.len() - 2;
        fake.u(last, 0, 97, 2);
        assert!(fake.parse().wstr(fake.va(last)).is_err());
    }
}
#[test]
fn truncation_at_every_length_never_panics() {
    for wide in [false, true] {
        let fake = Fake::new(wide, 2);
        for len in 0..fake.bytes.len() {
            let result = std::panic::catch_unwind(|| analyze(&fake.bytes[..len], Some(fake.array)));
            assert!(result.is_ok(), "panic at length {len}");
            assert!(result.unwrap().is_err(), "accepted truncated file at {len}");
        }
        assert!(analyze(&fake.bytes, Some(fake.array)).is_ok());
    }
}
#[test]
fn malformed_mutations_never_panic() {
    for wide in [false, true] {
        let fake = Fake::new(wide, 2);
        for i in 0..fake.cursor {
            let mut bytes = fake.bytes.clone();
            bytes[i] ^= 0xff;
            assert!(
                std::panic::catch_unwind(|| analyze(&bytes, Some(fake.array))).is_ok(),
                "panic at byte {i}"
            );
        }
    }
}

fn field_count(table: &bnsdata::tabledef::TableDefinition) -> usize {
    table
        .el
        .iter()
        .map(|e| e.body.attr.len() + e.body.sub.iter().map(|s| s.attr.len()).sum::<usize>())
        .sum()
}
#[test]
#[ignore = "operator files: BNSSCHEMA_EXE and BNSSCHEMA_REF"]
fn real_export_compare_counts_only() {
    let exe = PathBuf::from(std::env::var_os("BNSSCHEMA_EXE").expect("set BNSSCHEMA_EXE"));
    let reference = PathBuf::from(std::env::var_os("BNSSCHEMA_REF").expect("set BNSSCHEMA_REF"));
    let temp = Temp::new();
    let out = temp.0.join("defs");
    let found = export(&exe, None, &out).unwrap();
    let actual = Definitions::load_folder(&out).unwrap();
    let reference = Definitions::load_folder(reference).unwrap();
    assert_eq!(actual.0.len(), reference.0.len());
    let mut fields = 0;
    for (id, table) in &actual.0 {
        let expected = reference.get(*id).unwrap();
        assert_eq!(field_count(table), field_count(expected));
        fields += field_count(table);
    }
    println!(
        "tables={} fields={} unknown_type_count={}",
        found.tables,
        fields,
        found.unknown_types.len()
    );
}
