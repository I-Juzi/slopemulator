"""Read-only schema export for the supplied 2020 GameDaemon IDB.

Run with IDA's db object: export(db, output_path). The output must not exist.
No default records, default-value contents, or game key material are read.
Offsets describe the 64-bit server only. See 04-tabledefs-2020.md for evidence.
"""
import json
from pathlib import Path

ARRAY_ADDRESS = 0x140E73670
ARRAY_COUNT = 278


def collect(db):
    def raw(ea, size):
        value = db.bytes.get_bytes_at(ea, size)
        if value is None or len(value) != size:
            raise ValueError(f"unreadable schema metadata at {ea:#x}")
        return value

    def num(ea, size):
        return int.from_bytes(raw(ea, size), "little")

    def text(ea):
        if not ea:
            return None
        data = bytearray()
        for index in range(512):
            unit = raw(ea + 2 * index, 2)
            if unit == b"\0\0":
                return data.decode("utf-16le", errors="strict")
            data.extend(unit)
        raise ValueError("schema name exceeds bound")

    # Enum labels are metadata from the earlier PDB export, not target values.
    with Path(__file__).with_name("tabledefs.json").open(encoding="utf-8") as source:
        kinds = json.load(source)["attribute_types"]

    def fields(base, count):
        if count > 16384 or (count and not base):
            raise ValueError("invalid attribute metadata")
        result = []
        for index in range(count):
            attr = base + 88 * index
            type_id = num(attr + 16, 1)
            result.append({
                "address": hex(attr), "name": text(num(attr, 8)),
                "type_id": type_id, "type": kinds.get(str(type_id), "UNKNOWN"),
                "offset64": num(attr + 72, 2), "size64": num(attr + 74, 2),
                "repeat": num(attr + 18, 1), "key": bool(num(attr + 17, 1)),
                "required": bool(num(attr + 19, 1)),
                "reference_table": num(attr + 66, 2),
                "reference_element": num(attr + 68, 1),
                "deprecated": bool(num(attr + 69, 1)),
                "hidden": bool(num(attr + 70, 1)),
            })
        return result

    tables = []
    slots = []
    for slot in range(ARRAY_COUNT):
        address = num(ARRAY_ADDRESS + 8 * slot, 8)
        slots.append(hex(address) if address else None)
        if not address:
            continue
        type_id = num(address + 8, 2)
        if type_id != slot:
            raise ValueError("table ID does not match array slot")
        elcount = num(address + 24, 1)
        if not 0 < elcount < 128:
            raise ValueError("invalid element count")
        element_base = num(address + 16, 8)
        elements = []
        for index in range(elcount):
            el = element_base + 56 * index
            body = num(el + 16, 8)
            element = {"address": hex(el), "name": text(num(el, 8)),
                       "type": num(el + 8, 1),
                       "child_types": list(raw(el + 24, 32)),
                       "body_address": hex(body), "subtypes": []}
            if body:
                attrcount = num(body + 24, 2)
                subcount = num(body + 26, 2)
                if subcount > 16384:
                    raise ValueError("invalid subtype count")
                element.update({"size64": num(body + 28, 2),
                                "key_offset64": num(body + 30, 2),
                                "fields": fields(num(body, 8), attrcount)})
                subbase = num(body + 8, 8)
                for subtype_index in range(subcount):
                    sub = subbase + 32 * subtype_index
                    element["subtypes"].append({
                        "index": subtype_index, "address": hex(sub),
                        "name": text(num(sub, 8)), "size64": num(sub + 26, 2),
                        "fields": fields(num(sub + 8, 8), num(sub + 24, 2)),
                    })
                for layout in [element] + element["subtypes"]:
                    for field in layout["fields"]:
                        end = field["offset64"] + field["size64"] * field["repeat"]
                        if end > layout["size64"]:
                            raise ValueError(f"field exceeds record at {field['address']}")
            elements.append(element)
        auto = num(address + 40, 1)
        if auto not in (0, 1):
            raise ValueError("invalid autokey flag")
        tables.append({"address": hex(address), "type_id": type_id,
                       "name": text(num(address, 8)),
                       "major": num(address + 10, 2), "minor": num(address + 12, 2),
                       "kind": 1 if auto else 2, "autokey": bool(auto),
                       "element_array_address": hex(element_base), "elements": elements})
    return {"source": "ida/datafile/gd2020/GameDaemon.exe.i64",
            "build": "2020-07-04 server package; internal schema lineage not client 381",
            "pointer_width": 64, "offsets_and_sizes_32": None,
            "definition_array_address": hex(ARRAY_ADDRESS),
            "definition_array_count": ARRAY_COUNT,
            "definition_array_entries_observed": slots,
            "record_strides": {"DrTableDef": 48, "DrElDef": 56,
                               "DrElBodyDef": 32, "DrSubDef": 32, "DrAttrDef": 88},
            "attribute_type_labels_source": "build 348 PDB; target numeric IDs exported unchanged",
            "attribute_types": kinds, "tables": tables}


def export(db, path):
    result = collect(db)
    with open(path, "x", encoding="utf-8") as output:
        json.dump(result, output, ensure_ascii=False, indent=2)
        output.write("\n")
    return result
