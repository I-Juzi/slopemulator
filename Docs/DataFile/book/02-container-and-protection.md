# 2. Container format and protection, end to end

Build: client 0.0.381.8884 (data), `Client.exe` 0.0.381 x86 (code, IDA copy `ida/lanes/cx/Client.exe.i64`). Server 348 addresses are from `ida/datafile/gd348/GameDaemon.exe.i64`.
Rule: no key, salt, table or signature bytes are in this book. Where a value is needed, the text says `keyhere`.

## Quick find

| Question | Section | Address or file offset |
| --- | --- | --- |
| Which layers are there, in which order? | 2.1 | — |
| `.dat` archive header | 2.2.1 | file offset 0–216; type `DataPackFile::_DataPackHeader` |
| `.dat` file index | 2.2.2 | file offset 217; `DataPackFile::ReadFromFile` `0x7068b0` |
| `.dat` entry header (80 bytes) | 2.2.3 | type `PackFile::_fileHeader`; `PackFile::SetHeader` `0x7057e0` |
| `.dat` encryption: algorithm and key source | 2.3 | `PackEnc::Initialize` `0x7044d0`; `Aes128Table_Decrypt` `0x615960` |
| `.dat` compression | 2.2.4 | `DataPackFile::LoadFileContents` `0x706440` |
| `.dat` RSA signature | 2.4 | `PackSignature::CheckSignature` `0x7060b0`; header offset 27 |
| `datafile.bin` header | 2.5.1 | file offset 0–98 (x86) / 0–118 (x64) |
| Are `_compressed` / `_encrypted` used? | 2.5.2 | `0x624440` (no) |
| Table frame (11 bytes) | 2.5.3 | file offset 99 + alias size (x86) |
| Alias block | 2.5.4 | `DrAliasMapImpl::ReadFromBuffer` `0x625bd0` |
| Table compression (zlib blocks) | 2.6 | body byte 0; `DrCacheData::decompressChunk` `0x62fed0` |
| How do we undo and redo each layer? | 2.7 | Rust `bnsdata::dat`, `bnsdata::table` |
| Server binary XML (`LMXBOSLB`) XOR layer | 2.8 | `XmlReader_sv64.dll` 2020 `0x1800068d0` |

## 2.1 The layers

On a retail client, the bytes go through these layers when the client loads them (top = on disk):

```
 xml.dat / local.dat  on disk                        "UOSEDALB" v3, 217-byte header
 ├─ L1  RSA-1024 signature over the whole archive    check at start           0x7060b0
 ├─ L2  AES-128 (ECB) on the index and on entries    16-byte built-in key     0x7043f0
 ├─ L3  zlib on the index and on entries             per flag                 0x706440
 │
 └─ entry "datafile.bin"                             "TADBOSLB" v5, 99/119-byte header
    ├─ L4  no transform on the container             flags exist, not used    0x624440
    ├─     alias block (main file only)                                       0x625bd0
    └─ tables (11-byte frame each)
       └─ L5  per table: flag byte, then plain body or zlib blocks            0x6228b0, 0x62fed0
          └─ records (DrEl) + string pools: chapter 3
```

Order on load: verify signature (L1) → decrypt (L2) → inflate (L3) → parse container (L4) → inflate table blocks on demand (L5).
Order on build: L5 → L4 → L3 → L2 → L1 (signature last, because it covers the final archive bytes).

There is **no encryption inside `datafile.bin`**. The high entropy of its body (about 7.93 bits per byte, journal) comes from L5 zlib blocks and from UTF-16 text. Earlier notes guessed that the container was encrypted; that guess is wrong. ✅ Operator walk (`_encrypted` = 0, `_compressed` = 0 in all four files) and client code (2.5.2).

## 2.2 Layer 1–3 container: the `.dat` archive (`DataPackFile`)

### 2.2.1 Archive header (217 bytes, packed, little-endian)

Source: PDB type `DataPackFile::_DataPackHeader` (size `0xD9`) in the 381 IDA copy ✅; `DataPackFile::ReadFromFile` `0x7068b0` reads `0xD9` bytes at offset 0 ✅; `IsCorrectHeader` `0x7062f0` ✅.

| Offset | Size | Field | Rule |
| ---: | ---: | --- | --- |
| 0 | 8 | `_dataPackIdentifier` | ASCII `UOSEDALB` (`DataPackFile::DATAPACKFILE_IDENTIFIER` `0x14fc8b8`). ✅ |
| 8 | 1 | `_dataPackVersion` | must be 3, else the archive is rejected. ✅ `0x7062f0` |
| 9 | 8 | `_gameVersion` | not checked by the reader ✅ (no reference in `0x7068b0`) |
| 17 | 4 | `_totalSize` | not checked by the reader ✅ |
| 21 | 4 | `_totalFileCount` | number of index entries ✅ |
| 25 | 1 | `_compressed` | 1 = the index is zlib-compressed ✅ |
| 26 | 1 | `_encrypted` | 1 = the index is AES-encrypted ✅ |
| 27 | 128 | `_signature` | RSA-1024 signature (2.4) ✅ |
| 155 | 62 | `_reserved` | — |
| 217 | — | end | — |

Diagram:

```
0         8  9                17      21      25 26 27                         155               217
+---------+--+----------------+-------+-------+--+--+---------------------------+----------------+
|UOSEDALB |03| game version   |total  |files  |Z |E | signature (128)           | reserved (62)  |
+---------+--+----------------+-------+-------+--+--+---------------------------+----------------+
```

❓ This is the x86 layout. The 64-bit archives can have other field widths. No 64-bit reader was checked.
❓ No real `.dat` file is in the worktree, so this book has no hex dump of a `UOSEDALB` header. The operator files are already extracted `.bin` files.

### 2.2.2 File index

Directly after the header (all ✅ `0x7068b0`):

| Offset | Size | Field |
| ---: | ---: | --- |
| 217 | 4 | `S` = stored index size |
| 221 | 4 | `P` = plain index size |
| 225 | `S` | index bytes |
| 225 + `S` | 4 | `contentsPos` = start of the contents area |

Decode of the index bytes:

1. If header byte 26 (`_encrypted`) ≠ 0: AES-128 decrypt `S` bytes (2.3). The output size is `S`.
2. If header byte 25 (`_compressed`) ≠ 0: zlib `uncompress` to `P` bytes.
3. Read `_totalFileCount` entries:

```
entry:  u32 N                 name length in UTF-16 code units
        N * 2 bytes           name, UTF-16LE, no terminator         e.g. "datafile.bin"
        80 bytes              PackFile::_fileHeader (2.2.3)
```

When `N` = 0, the entry is only the 4-byte length; no name and no header follow. ✅ (`if (v19 != 0)` branch in `0x7068b0`).

4. The reader puts each `name → PackFile` pair in a `std::map<std::wstring, PackFile*>` (case of the compare: ❓). Lookups use `FindFirstMatchingFile` `0x707120` with a filter string.
5. Check: `contentsPos` must equal the number of bytes read so far (217 + 4 + 4 + `S` + 4). If not, the archive is rejected. ✅

### 2.2.3 Entry header (`PackFile::_fileHeader`, 80 bytes)

Source: PDB type `PackFile::_fileHeader` ✅; use in `LoadFileContents` `0x706440` ✅.

| Offset | Size | Field | Use |
| ---: | ---: | --- | --- |
| 0 | 1 | `_version` | the constructor sets 2 before the copy; not checked ✅ |
| 1 | 1 | `_zipped` | 1 = zlib |
| 2 | 1 | `_encrypted` | 1 = AES-128 |
| 3 | 1 | `_encryptType` | not read by `LoadFileContents` ✅; meaning ❓ |
| 4 | 4 | `_originalSize` | size after all decoding |
| 8 | 4 | `_zippedSize` | size of the zlib stream |
| 12 | 4 | `_encryptedSize` | size of the stored (encrypted) bytes |
| 16 | 4 | `_offset` | relative to `contentsPos` |
| 20 | 60 | `_reserved` | — |

### 2.2.4 Entry decode (`LoadFileContents` `0x706440`)

```
stored = encrypted ? _encryptedSize : (zipped ? _zippedSize : _originalSize)
seek(contentsPos + _offset); read `stored` bytes             (fail if short)
if encrypted: AES-128-ECB decrypt `stored` bytes -> buffer of _encryptedSize
if zipped:    zlib uncompress(input = first _zippedSize bytes, output = _originalSize)
result -> PackFile._fileBuffer (+0x50), _fileBufferSize (+0x54)
```

All steps ✅ `0x706440`. Encrypted data is padded to whole 16-byte blocks: the decrypt loop always works on 16-byte steps (2.3), and zlib reads only `_zippedSize` bytes, so any padding is ignored. ❓ The padding byte value is not known (any value works for the reader).

## 2.3 Layer 2: AES-128 encryption of the `.dat` archive

| Item | Fact | Source |
| --- | --- | --- |
| Object | One global `PackEnc` (`DataPackFile::_packEncoder`, initializer `0x13987f0`) | ✅ |
| Setup | `PackEnc::Initialize` `0x7044d0` runs once (guard byte `0x1a4d553`) | ✅ |
| Key size | 16 bytes, made in a loop of 16 steps | ✅ `0x704530`–`0x704573` |
| Key source | Built into `Client.exe`: a table of 4 u32 values at `0x19ff150` and a table of 4 bytes at `0x19ff14c`. Byte `i` (0–15) = low byte of (`u32[i mod 4]` shifted right by `i` bits) XOR `byte[i mod 4]`. Values: `keyhere`. | ✅ disassembly `0x704530`–`0x704568` |
| Processors | `DataProcessor::Create(0)` = `DataEncrypt`, `Create(1)` = `DataDecrypt`; both set crypto type 3 and get the 16-byte key | ✅ `0x607490` jump table, `0x7045d5`, `0x704613` |
| Cipher | Crypto type 3 = `Aes128` (`Crypto::Create` `0x60b010`, case 3) | ✅ |
| Mode | ECB: `Aes128Table_Decrypt` `0x615960` decrypts each 16-byte block alone, no IV, no chaining; output size = number of bytes processed | ✅ |
| Decrypt call | `PackEnc::Decrypt` `0x7043f0` → decrypt processor `vtbl + 20`; it requires source size ≥ destination size | ✅ |
| Same key for all archives | Yes: one global encoder for `config.dat`, `xml.dat`, `local.dat` | ✅ (`_packEncoder` is static) |

Other crypto types in the same factory (not used by the data path): 1 `SimpleBlock`, 33 `SimpleStream`, 65 `RsaPublic`, 81 `RsaPrivate`. ✅ `0x60b010`. The network session crypto (`PPUtil::SetupSessionCrypto` `0x60f8f0`) uses the same library; that is a separate topic.

## 2.4 Layer 1: RSA signature of the `.dat` archive

`PackXmlReaderIO::Initialize` `0x7076b0` opens the archive, reads the index, and then calls `PackSignature::CheckSignature` `0x7060b0`. If the check fails, it closes the archive and returns false. ✅ ❓ What `BNSClient::Initialize` does after a false return (stop or continue) was not checked.

Inputs: the three `SignatureKey` blobs from `BNSClient::Initialize` (salt 32 bytes, key 610 bytes, checksum 32 bytes; values `keyhere`). ✅ `0x116fe9c`.

Steps:

1. `VerifySignatureKey` `0x705ba0`: decode the three blobs with `Transcode` `0x705900` (a per-byte XOR with a 6-byte built-in table plus fixed bit swaps; table `keyhere`). Hash with SHA-256 and compare the result with the decoded checksum. ✅ calls; ❓ the exact bytes that go into the hash (two vector inserts, order not traced).
2. Load the decoded key with `d2i_PrivateKey(6 = EVP_PKEY_RSA, ...)` and take the RSA key. ✅ The client holds an RSA **private** key blob.
3. `GetFileSignature` `0x705a10`: read the whole archive file. Copy the 128 bytes at offset 27 (the header `_signature`). In the copy of the file, set those 128 bytes to 0, then write a fixed 32-byte value (`0x149089c`, `keyhere`) at offset 27. Compute SHA-256 of the whole changed file. ✅
4. `RSA_private_decrypt(128, signature, out, key)` and compare the first 32 bytes of `out` with the SHA-256 value. Equal → pass. ✅ ❓ padding mode argument (not traced; PKCS#1 v1.5 is the OpenSSL default).

```
archive bytes ──copy──> [0..26] [27..154 := 0, then 27..58 := fixed 32 bytes] [155..end]
                                     │
                                SHA-256 ──────────────┐
header[27..154] ── RSA_private_decrypt(key) ──> 32 B ─┴─ equal?
```

Consequence: the signature covers the whole archive, header and contents. Any change to `datafile.bin` inside `xml.dat` needs a new signature. Because the verify key is an RSA key pair that the client holds, the same key can make a valid signature (❓ not tested). The private-server client avoids this layer: `binreplace` gives the loose `.bin` file to the client (chapter 1, 1.7).

## 2.5 Layer 4: the `TADBOSLB` container (`datafile.bin`, `localfile.bin`)

### 2.5.1 Header

Packed, little-endian. The version is one byte, not u32. x86 layout: PDB type `BinaryDataFile::_BinaryDataFileHeader` in the 381 IDA copy, size `0x63` = 99 ✅. x64 layout: server 348 PDB type and readers ✅ (`02-tadboslb.md`); operator walk ✅ (header 119 bytes).

| Field | x86 offset | x64 offset | Size x86 / x64 | Client 381 use (`0x624440`) |
| --- | ---: | ---: | --- | --- |
| `_binaryDataFileIdentifier` `TADBOSLB` | 0 | 0 | 8 | must match ✅ |
| `_binaryDataFileVersion` | 8 | 8 | 1 | must be 5 ✅ |
| `_gameVersion` (4 × u16: 0, 0, 381, 8884) | 9 | 9 | 8 | not read ✅ |
| `_totalTableSize` | 17 | 17 | 4 / 8 | must be ≠ 0 to read tables ✅ |
| `_totalTableCount` | 21 | 25 | 4 / 8 | loop count ✅ |
| `_aliasSize` | 25 | 33 | 4 / 8 | alias block length ✅ |
| `_aliasCount` | 29 | 41 | 4 / 8 | alias count ✅ |
| `_maxBufferSize` | 33 | 49 | 4 / 8 | not read ✅ |
| `_saveTime` (Unix time, i32) | 37 | 57 | 4 | not read ✅ |
| `_compressed` | 41 | 61 | 1 | not read ✅ |
| `_encrypted` | 42 | 62 | 1 | not read ✅ |
| `_reserved` | 43 | 63 | 56 | not read ✅ |
| end | 99 | 119 | — | — |

"Not read" means that the decompiled reader `0x624440` uses no other header byte than offsets 0–8, 17, 21, 25, 29. The x64 client reader is ❓ (no IDA copy).

Header of `datafile.bin` (x86), **rebuilt** from the verified field values in the journal (not a raw dump; the operator file is outside the worktree). `??` = value not recorded. No key bytes exist in this header.

```
offset  00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F
000000  54 41 44 42 4F 53 4C 42  05 00 00 00 00 7D 01 B4   TADBOSLB.....}..
000010  22 01 D7 44 07 5C 01 00  00 35 A2 ED 01 E4 5D 1A   "..D.\...5....].
000020  00 ?? ?? ?? ?? BA 6C E1  5E 00 00 00 00 00 00 00   .....l.^........
000030  .. reserved (56 bytes, value not recorded ❓) ..
000060  .. .. ..|<- 0x63 = 99: alias block starts
```

| Bytes | Field | Value |
| --- | --- | --- |
| `05` @8 | version | 5 ✅ |
| `00 00 00 00 7D 01 B4 22` @9 | game version | 0.0.381.8884 ✅ (journal, BnsBinTool cross-check) |
| `01 D7 44 07` @17 | total table size | 121,952,001 ✅ value; bytes computed |
| `5C 01 00 00` @21 | table count | 348 ✅ value; bytes computed |
| `35 A2 ED 01` @25 | alias size | 32,350,773 ✅ value; bytes computed |
| `E4 5D 1A 00` @29 | alias count | 1,727,972 ✅ value; bytes computed |
| `BA 6C E1 5E` @37 | save time | 2020-06-10 23:28:58 UTC ✅ value; bytes computed |
| `00` @41, `00` @42 | compressed, encrypted | 0, 0 ✅ |

`localfile.bin` has the same header shape. It repeats the main file's alias size and count, but its body has no alias block. ✅ Operator walk.

### 2.5.2 What the client checks

From `0x624440` (client read-from-memory; arguments: buffer, size, loader definitions, count, isLocal):

1. Size ≥ 99, signature and version 5. Else fail. ✅
2. Main file (isLocal = 0): save the 99-byte header in the global `0x19a44d8`. Local file: fail if the saved alias count is 0 (main file not loaded). ✅
3. Main file with alias count ≠ 0 and alias size ≠ 0: read the alias block through the alias map (`vtbl + 24`). ✅
4. When table count ≠ 0 and total table size ≠ 0: read the tables (2.5.3). ✅
5. No checksum, no hash, no signature, no game-version compare. ✅ (no such calls in `0x624440`)

So the client accepts a changed `datafile.bin` when the frame, the versions and the records are correct. The values of `_totalTableSize` and `_maxBufferSize` are not checked by this reader (only "≠ 0"). ✅ for the x86 reader. Our writer keeps them unchanged (2.7).

### 2.5.3 Table frame

After the header (and after the alias block in the main file), the tables follow back to back:

```
+------+---------+---------+---------+-------------+------------------+
| kind | type    | major   | minor   | body size B | body (B bytes)   |
| u8   | u16     | u16     | u16     | u32         | chapter 3        |
+------+---------+---------+---------+-------------+------------------+
  0      1         3         5         7             11
```

| Rule | Source |
| --- | --- |
| kind 1 = auto-key table (`DrAutoKeyTable`), kind 2 = multi-key table (`DrMultiKeyTable`) | ✅ `0x624440`; server `0x1406fa370` |
| The client finds the definition whose type (u16 at definition + 4) equals `type`. It walks a list of 52-byte loader entries. | ✅ `0x624440` |
| Unknown type: the client skips `11 + B` bytes and goes on. | ✅ `0x624440` (`LABEL_32`) |
| major/minor ≠ compiled definition: the whole load fails. | ✅ `0x624440` |
| The table object reads its body with `ReadFromBuffer`. The body size is the table's own length; no global offset table exists. | ✅ |
| `UseLegacyElHeader(flag)` is called on each table. The flag from `loadBinaryDataFile` is 0. No legacy 8-byte prefix occurs in the 381 files. | ✅ call; ✅ walk (`06-client-body.md`); ❓ who sets 1 |

### 2.5.4 Alias block

The alias block maps text names (aliases such as `item_xxx`) to 8-byte record keys. Main file only. Size and count come from the header.

The 381 client reader `DrAliasMapImpl::ReadFromBuffer` `0x625bd0` has three forms. Flags in the alias map object choose the form:

| Form | Layout | Source |
| --- | --- | --- |
| String table | `DrAliasStringTable::ReadFromBuffer` | ✅ call in `0x625bd0`; server layout: u32 count, 16-byte items, u32 pool size, pool (`02-tadboslb.md`, server `0x1406f3900`) |
| Map (legacy) | per alias: u64 key, u8 length L, L UTF-16 units; at most 200 units (copy into a 402-byte buffer) | ✅ `0x625bd0`; server `0x1406f5fa0` |
| Multi-dictionary array | u32, u32, u32 count M, M × 12-byte items, u32 pool size, pool bytes. The first u32 of each item is an offset into the pool; the reader adds the pool base. | ✅ `0x625bd0` lines that read `buffer + 12` and relocate every 12-byte item |

❓ Which form the 381 files use. Our writer keeps the alias block as opaque bytes, so this question does not block editing records. Alias count 1,727,972 and size 32,350,773 (x86) give about 18.7 bytes per alias. ✅ numbers.

## 2.6 Layer 5: table compression

Each client table body starts with one byte: 0 = plain, 1 = block-compressed. The server 348 body has no such byte. ✅ `06-client-body.md` (walk), client `DrMultiKeyTable::ReadFromBuffer` `0x6228b0` (plain path reads count at body + 1, fixed size at + 5, extra size at + 9, fixed region at + 14; compressed path calls `DrCacheData::ReadFromBuffer` `0x62f870` with body + 1).

Compressed body:

```
+1  u32 block count
+5  u16 constant 8 (all tables) ❓ meaning
+7  blocks:
    +------------+-----------+-----+---------------+-----+-----+-----------------+
    | first key  | last key  | cs  | zlib (cs)     | ds  | rc  | rc x u16 offset |
    | i64        | i64       | u16 | "78 01" ...   | u16 | u32 |                 |
    +------------+-----------+-----+---------------+-----+-----+-----------------+
```

- Plain zlib (RFC 1950) with the Adler-32 trailer. Every block inflates to exactly `ds` ≤ 65,535 bytes. ✅ walk of 16,332 blocks (datafile x86).
- The client keeps the compressed blocks in memory and inflates one block on first use (`DrCacheData::decompressChunk` `0x62fed0`, zlib `uncompress`, buffer size from the u16 at chunk + 24). After inflate it relocates the records (`RelocateMemoryToDecompress`). ✅
- Counts per file: 29 compressed tables in each main file, 1 (`text`) in each local file. ✅ `06-client-body.md`.

## 2.7 How we undo and redo each layer

| Layer | Undo (read) | Redo (write) | Tool | State |
| --- | --- | --- | --- | --- |
| L1 RSA signature | Not needed to read (only checked) | Sign the final archive with the client RSA key (`keyhere`), method in 2.4 | none in repo | ❓ not done; not needed with `binreplace` |
| L2 AES-128-ECB | Make the key as in 2.3 (`keyhere`), decrypt 16-byte blocks | Encrypt 16-byte blocks; pad the zlib stream to a multiple of 16 | none in repo | ❓ not done |
| L3 zlib (`.dat`) | `uncompress` to `_originalSize` | deflate; set `_zippedSize`, `_encryptedSize`, `_originalSize`, `_offset`, index sizes, `contentsPos` | none in repo | ❓ not done |
| L4 `TADBOSLB` | `bnsdata::dat` `Container::parse(bytes, width, role)` | `Container::write()`: updates table count, body lengths, alias size; keeps `_totalTableSize`, `_maxBufferSize`, save time, flags, reserved | `Program/Tools/DataFile/BnsData/src/dat.rs` | ✅ byte-identical on all four files (`tests/real_files.rs` `client_roundtrip`) |
| L5 table zlib | `bnsdata::table` `TableBody::parse` | `TableBody::write`: unchanged tables keep the original zlib bytes; changed tables are re-blocked (≤ 65,535 bytes) and re-compressed | `.../BnsData/src/table.rs` | ✅ round trip; ❓ client acceptance of new zlib output |
| Records and strings | `table::decode_fields_with_pool` | `RecordEditor::set_text`, pool rebuild | `.../BnsData/src/table/fields.rs`, `text.rs` | ✅ (chapter 3) |
| Whole project | `bnsmodel` `files::open`, `Project::record` | `files::pack` | `Program/Tools/DataFile/BnsModel` | ✅ edit → pack → reopen test |

Width and role: the `TADBOSLB` header has no field that says x86 or x64, or main or local. The caller selects them (`BNSDATA_DAT_WIDTH=32|64`, `BNSDATA_DAT_ROLE=main|local`). ✅ `02-tadboslb.md`. A file name rule works in practice: `*64.bin` = x64, `localfile*` = local.

Conflict resolved: BnsBinTool (no licence, cross-check only) reads `_binaryDataFileVersion` as u8 and the game version as 4 × u16; our first notes read u32 version. The PDB type and the client check (`a2[8] == 5`) win. ✅

## 2.8 Related: the server binary XML layer (`LMXBOSLB`)

The server keeps the same data as binary XML, one file per data type. It is not part of the client chain, but the same build makes both. ✅ `Docs/DataFile/01-lmxboslb.md`.

| Item | Fact | Source |
| --- | --- | --- |
| Header | `LMXBOSLB`, version byte 3, 3 zero bytes, u32 total size at 12, 64 reserved bytes, u8 valid flag at 80 | ✅ 2020 `XmlReader_sv64.dll` `0x180006e40`, `0x18000a920` |
| Protection | Each string payload is XORed with a 16-byte table; the table position restarts at 0 for each string. Counts and tree data are not XORed. | ✅ 2020 `0x1800068d0`; 2019 PDB `cipherKeys` `0x1801ba930` |
| Key | Static 16-byte table in the DLL (`0x1800dc4b0`, 2020). Value `keyhere`; our tools read it from `BNSDATA_KEY_FILE`. | ✅ |
| Undo/redo | `bnsdata::lmx` `Document::parse(bytes, key)` / `write(key)`; `lmxb::decode` / `encode` (server crate) | ✅ round trip on server files |

## Journal links

Merged from: `Docs/DataFile/02-tadboslb.md` (all), `06-client-body.md` ("Body = 1 flag byte", "Compressed variant", "Writer rules"), `01-lmxboslb.md` ("Header", "Strings and obfuscation"), journal sections "Client files received", "External tool: BnsBinTool", "Operator-side validation of the dat1 spec", "dat2 results". New in this chapter: all `.dat` archive facts (2.2–2.4) and the client checks in 2.5.2 (IDA, 2026-11-09).
