# LMXBOSLB version 3

## Step 2 implementation and validation

The operator validated this layout on 60 random server files. Each structure-only walk ended at the exact file size. This supersedes the earlier real-file validation note below. This job did not open game data files.

The Rust `lmx` module parses and writes the tree. It preserves UTF-16 code units, ordered attributes, child order, flag bytes, line numbers, the three version-padding bytes, and all 64 reserved bytes. `binary_size()` uses the formulas below. The key-free `walk()` returns element, text-node, attribute, string, UTF-16-unit, and byte counts. It does not decode string payloads.

Readers reject unknown child tags, truncation, impossible counts, size mismatches, trailing bytes, and nesting beyond 256 levels below the root. Writers apply the same depth limit and reject sizes that exceed u32. These limits are implementation safeguards, not new format fields.

`load_key()` reads the path in `BNSDATA_KEY_FILE`. The file must contain exactly 16 raw bytes or 32 ASCII hex characters, without a newline. The parser and writer take the resulting 16-byte key as an explicit argument. The helper fails clearly when the environment variable is absent. No built-in key or fallback exists.

The ignored `server_roundtrip` test uses `BNSDATA_SERVER_XML` and `BNSDATA_KEY_FILE`. It opens the source read-only. It compares the written bytes in memory and prints counts only. Normal tests use generated keys and synthetic documents.

## Sources and scope

This specification describes the server binary XML format. All integers use little-endian byte order. There is no field alignment in the body.

| Operation | 2019 PDB DLL address | 2020 target DLL address |
| --- | --- | --- |
| Check file signature | `0x1800066e0` | `0x180006b70` |
| Check memory signature and exact size | `XmlReaderImpl::IsBinary` | `0x18000a920` |
| Write document | `0x180006970` | `0x180006e40` |
| Read document | `0x180006ab0` | ❓ not checked |
| Write string | `0x180006470` | `0x1800068d0` |
| Read string | `0x1800065f0` | ❓ not checked |
| Write common node | `0x180007240` | `0x180007800` |
| Read common node | `0x180007380` | ❓ not checked |
| Write element | `0x1800081c0` | `0x180008830` |
| Read element | `0x180008280` | ❓ not checked |
| Write/read text | `0x180008700` / `0x180008760` | ❓ not checked |
| Size document/node/element/text | `0x180006900` / `0x1800071b0` / `0x180007f00` / `0x180008660` | ❓ not checked |

Builds: the supplied `ida/datafile/xml2019/XmlReader_sv64.dll.i64` has PDB names. The supplied `ida/datafile/xml2020/XmlReader_sv64.dll.i64` is the GameDaemon 2020 target. The inspected 2020 writers match the 2019 layout. Text-node confirmation in the target remains open.

## Header

| Offset | Width | Meaning |
| --- | --- | --- |
| 0 | 8 | ASCII `LMXBOSLB` |
| 8 | 1 | Version, 3 |
| 9 | 3 | Zero padding from the writer |
| 12 | 4 | Total binary size, including this header |
| 16 | 64 | Reserved; the writer clears these bytes |
| 80 | 1 | Document valid flag |
| 81 | variable | Document name string |
| next | variable | Root element, without a type tag |

Important: the version is a byte, not a u32. The u32 value 3 in observed headers includes three zero padding bytes. Source: 2020 `0x180006e40` writes `LOBYTE(Src[1])`; `0x18000a920` checks `BYTE8` and the u32 size at 12.

The memory checker requires more than 80 bytes and requires the supplied size to equal the header size. The file-signature checker does not enforce that size condition.

## Strings and obfuscation

A string has a u32 UTF-16 code-unit count, then exactly twice that count in bytes. The payload uses UTF-16LE. There is no stored terminator. A zero count uses four bytes. The in-memory reader appends a terminator.

Each payload byte is XORed with a repeating 16-byte table. The table position starts at zero for EACH string. Counts and node metadata are not XORed. Source: 2019 `string_to_buffer` at `0x180006470`, `memory_from_buffer` at `0x180006550`; 2020 `0x1800068d0`.

Key provenance: the 2019 DLL has static symbol `cipherKeys` at `0x1801ba930`. The target DLL references a 16-byte table at `0x1800dc4b0`. There is no derivation in these helpers. ❓ Table equality across builds was not checked. No table bytes are included here. The Rust API requires a 16-byte runtime key file selected by `BNSDATA_KEY_FILE`. Tests use generated keys only.

## Nodes

The document owns one root element. It is not a common node record.

An element stores, in order:

1. u32 attribute count.
2. Each attribute as a name string, then a value string, in original order.
3. The common node record.

Attributes have no type tag. Attribute hashes are in-memory lookup data, not serialized fields. Source: 2019 `SetAttrName` at `0x180007c00` and the element writers above. The decompiler applies a misleading base-pointer type to some member accesses. The target writer uses offsets 80 and 88 relative to the node base for the two string arrays.

A text node stores a text string, then the common node record.

The common node record stores, in order:

1. u8 valid flag.
2. Name string.
3. u32 child count.
4. u32 source line number.
5. Each child as a u32 type tag followed by its complete record.

Child tags: 1 = element; 2 = text. Source: 2019 `0x180007380`. Other tags are not specified. The reader must reject them. It must not guess a record length.

## BinarySize

Let `S(s) = 4 + 2 * number_of_UTF16_code_units(s)`.

- Common node: `9 + S(name) + sum(4 + child_size)`.
- Element: `4 + sum(S(attribute_name) + S(attribute_value)) + common_node_size`.
- Text: `S(text) + common_node_size`.
- Document: `81 + S(document_name) + root_element_size`.

The original document size method returns zero when there is no root. A Rust document must have a root. Preserve reserved header bytes, flag bytes, string code units, line numbers, attribute order, and child order for byte-identical round trips. Do not replace raw UTF-16 with a lossy Unicode conversion.

## Validation state

✅ The inspected 2019 and 2020 header, string, common-node, and element writers agree.

❓ Three real-file header reads were requested through the shell. The tool required approval and returned no file results. No real-file validation is claimed.

❓ No client acceptance check was identified in this XML code. This is not proof that the client accepts a changed file.
