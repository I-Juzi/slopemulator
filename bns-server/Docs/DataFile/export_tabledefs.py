"""Build 348 metadata export. Run only inside the supplied IDA database.

This reads schema names and layouts, not default records, values, or key material.
It does not change or save the IDA database. The output path must not exist.
The documented offsets come from the supplied PDB types.
"""
import json

ARRAY_ADDRESS = 0x140E99970


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

    symbols = {ea: name for ea, name in db.names.get_all() if "@@3UDrTableDef@@B" in name}
    kinds = {member.value: member.name for member in
             db.types.get_enum_members(db.types.get_by_name("DrAttrType"))}
    array = []
    # The named DrTableDef symbols bound the allowed pointer targets.
    for index in range(len(symbols) + 2):
        pointer = num(ARRAY_ADDRESS + 8 * index, 8)
        if index == 0 and pointer == 0:
            array.append(None)
        elif pointer in symbols:
            array.append(hex(pointer))
        else:
            break
    tables = []
    for address, symbol in symbols.items():
        elcount = num(address + 24, 1)
        if elcount > 127:
            raise ValueError("unexpected element count")
        element_base = num(address + 16, 8)
        elements = []
        for index in range(elcount):
            el = element_base + 56 * index
            body = num(el + 16, 8)
            element = {"address": hex(el), "name": text(num(el, 8)),
                       "type": num(el + 8, 1), "body_address": hex(body)}
            if body:
                attrbase = num(body, 8)
                attrcount = num(body + 24, 2)
                if attrcount > 16384:
                    raise ValueError("unexpected attribute count")
                fields = []
                for field_index in range(attrcount):
                    attr = attrbase + 88 * field_index
                    type_id = num(attr + 16, 1)
                    fields.append({"address": hex(attr), "name": text(num(attr, 8)),
                                   "type_id": type_id, "type": kinds.get(type_id, "UNKNOWN"),
                                   "offset64": num(attr + 72, 2), "size64": num(attr + 74, 2),
                                   "repeat": num(attr + 18, 1), "key": bool(num(attr + 17, 1)),
                                   "reference_table": num(attr + 66, 2),
                                   "reference_element": num(attr + 68, 1)})
                element.update({"size64": num(body + 28, 2),
                                "key_offset64": num(body + 30, 2),
                                "subdef_address": hex(num(body + 8, 8)),
                                "subdef_count": num(body + 26, 2), "fields": fields})
            # Subtype layouts use the same DrAttrDef stride and field offsets.
            subtypes = []
            if body:
                subbase = num(body + 8, 8)
                for subtype_index in range(num(body + 26, 2)):
                    sub = subbase + 32 * subtype_index
                    subattrs = num(sub + 8, 8)
                    subfields = []
                    for field_index in range(num(sub + 24, 2)):
                        attr = subattrs + 88 * field_index
                        type_id = num(attr + 16, 1)
                        subfields.append({"address": hex(attr), "name": text(num(attr, 8)),
                                          "type_id": type_id, "type": kinds.get(type_id, "UNKNOWN"),
                                          "offset64": num(attr + 72, 2), "size64": num(attr + 74, 2),
                                          "repeat": num(attr + 18, 1), "key": bool(num(attr + 17, 1)),
                                          "reference_table": num(attr + 66, 2),
                                          "reference_element": num(attr + 68, 1)})
                    subtypes.append({"index": subtype_index, "address": hex(sub),
                                     "name": text(num(sub, 8)), "size64": num(sub + 26, 2),
                                     "fields": subfields})
            element["subtypes"] = subtypes
            elements.append(element)
        tables.append({"address": hex(address), "symbol": symbol,
                       "type_id": num(address + 8, 2), "name": text(num(address, 8)),
                       "major": num(address + 10, 2), "minor": num(address + 12, 2),
                       "kind": 1 if num(address + 40, 1) else 2,
                       "autokey": bool(num(address + 40, 1)),
                       "element_array_address": hex(element_base), "elements": elements})
    tables.sort(key=lambda item: item["type_id"])
    return {"source": "ida/datafile/gd348/GameDaemon.exe.i64", "build": 348,
            "target_0_0_381": "unverified; do not use these definitions as target schemas",
            "pointer_width": 64, "offsets_and_sizes_32": None,
            "definition_array_address": hex(ARRAY_ADDRESS), "definition_array_stride": 8,
            "definition_array_entries_observed": array,
            "record_strides": {"DrLoaderDef": 96, "DrTableDef": 48,
                               "DrElDef": 56, "DrElBodyDef": 32, "DrAttrDef": 88},
            "attribute_types": kinds, "tables": tables}


def export(db, path):
    result = collect(db)
    with open(path, "x", encoding="utf-8") as output:
        json.dump(result, output, ensure_ascii=False, indent=2)
        output.write("\n")
    return result
