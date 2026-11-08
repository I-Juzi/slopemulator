//! TableDefs JSON, with the same keys and reads as the Python reference.
use crate::{find::Array, layout::Layout, pe::Pe, Error};
use serde_json::{json, Value};
use std::collections::BTreeSet;

struct Reader<'a, 'b> {
    pe: &'a Pe<'b>,
    l: &'static Layout,
    unknown: BTreeSet<u8>,
    remaining: usize,
}
impl Reader<'_, '_> {
    fn u(&self, v: u64, o: usize, n: usize) -> Result<u64, Error> {
        self.pe.uint(v, o, n)
    }
    fn p(&self, v: u64, o: usize) -> Result<u64, Error> {
        self.u(v, o, self.l.p)
    }
    fn name(&self, v: u64, o: usize) -> Result<Option<String>, Error> {
        self.pe.wstr(self.p(v, o)?)
    }
    fn budget(&mut self, n: usize) -> Result<(), Error> {
        self.remaining = self
            .remaining
            .checked_sub(n)
            .ok_or_else(|| Error::Format("schema exceeds 2000000 entries".into()))?;
        Ok(())
    }
    fn span(&mut self, v: u64, n: usize, stride: usize) -> Result<(), Error> {
        self.budget(n)?;
        if n != 0 {
            let size = n
                .checked_mul(stride)
                .ok_or_else(|| Error::Format("structure span overflow".into()))?;
            self.pe.data(v, size)?;
        }
        Ok(())
    }
    fn signed_count(&self, v: u64, o: usize) -> Result<usize, Error> {
        let n = self.u(v, o, 2)? as i16;
        usize::try_from(n).map_err(|_| Error::Format("negative body count".into()))
    }
    fn float(&self, v: u64, o: usize) -> Result<f32, Error> {
        let f = f32::from_bits(self.u(v, o, 4)? as u32);
        if f.is_finite() {
            Ok(f)
        } else {
            Err(Error::Format("non-finite attribute float".into()))
        }
    }
    fn defrec(&self, v: u64) -> Result<Value, Error> {
        if self.pe.va_to_offset(v).is_none() {
            return Ok(Value::Null);
        }
        Ok(
            json!({"type": self.u(v, 0, 1)? as i8, "subtype": self.u(v, 2, 2)? as i16, "size": self.u(v, 4, 2)?}),
        )
    }
    fn attrs(&mut self, v: u64, n: usize) -> Result<Value, Error> {
        let a = &self.l.attr;
        self.span(v, n, a.stride)?;
        let mut out = Vec::new();
        for i in 0..n {
            let v = v
                .checked_add((i * a.stride) as u64)
                .ok_or_else(|| Error::Format("attribute VA overflow".into()))?;
            let t = self.u(v, a.kind, 1)? as u8;
            let kind = match type_name(t) {
                Some(name) => name.to_owned(),
                None => {
                    self.unknown.insert(t);
                    format!("type_{t}")
                }
            };
            let hv = self.p(v, a.namehash)?;
            let namehash = if self.pe.va_to_offset(hv).is_none() {
                Value::Null
            } else {
                json!({"namehash": self.u(hv, self.l.nh.hash, 4)?, "name": self.name(hv, self.l.nh.name)?})
            };
            let seqv = self.p(v, a.seq)?;
            let seqn = self.u(v, a.seqcount, 2)? as usize;
            let seq = if self.pe.va_to_offset(seqv).is_none() {
                Value::Null
            } else {
                self.span(seqv, seqn, self.l.p)?;
                let mut values = Vec::new();
                for k in 0..seqn {
                    values.push(self.pe.wstr(self.p(seqv, k * self.l.p)?)?);
                }
                json!(values)
            };
            out.push(json!({
                "name": self.name(v, a.name)?, "namehash": namehash, "type": kind,
                "key": self.u(v, a.key, 1)? != 0, "repeat": self.u(v, a.repeat, 1)?,
                "required": self.u(v, a.required, 1)?, "defvalue": self.name(v, a.defvalue)?,
                "min": self.u(v, a.min, 8)? as i64, "max": self.u(v, a.max, 8)? as i64,
                "fmin": self.float(v, a.fmin)?, "fmax": self.float(v, a.fmax)?,
                "seq": seq, "seqcount": seqn, "reftable": self.u(v, a.reftable, 2)? as i16,
                "refel": self.u(v, a.refel, 1)?, "deprecated": self.u(v, a.deprecated, 1)? != 0,
                "hidden": self.u(v, a.hidden, 1)? != 0, "offset": self.u(v, a.offset, 2)?,
                "size": self.u(v, a.size, 2)?, "propfield": self.u(v, a.propfield, 4)?
            }));
        }
        Ok(if out.is_empty() {
            Value::Null
        } else {
            json!(out)
        })
    }
    fn table(&mut self, v: u64) -> Result<Value, Error> {
        let (td, el, bd, sb) = (&self.l.td, &self.l.el, &self.l.body, &self.l.sub);
        let elv = self.p(v, td.el)?;
        let elc = self.u(v, td.elcount, 1)? as usize;
        self.span(elv, elc, el.stride)?;
        let mut elements = Vec::new();
        for i in 0..elc {
            let ev = elv
                .checked_add((i * el.stride) as u64)
                .ok_or_else(|| Error::Format("element VA overflow".into()))?;
            let bv = self.p(ev, el.body)?;
            self.pe.data(bv, bd.stride)?;
            let ac = self.signed_count(bv, bd.attrcount)?;
            let sc = self.signed_count(bv, bd.subcount)?;
            let sv = self.p(bv, bd.sub)?;
            self.span(sv, sc, sb.stride)?;
            let mut subs = Vec::new();
            for j in 0..sc {
                let v = sv
                    .checked_add((j * sb.stride) as u64)
                    .ok_or_else(|| Error::Format("subtype VA overflow".into()))?;
                let sac = self.u(v, sb.attrcount, 2)? as usize;
                let attr = self.attrs(self.p(v, sb.attr)?, sac)?;
                subs.push(json!({"name": self.name(v, sb.name)?, "attr": attr,
                    "defaultRecord": self.defrec(self.p(v, sb.defrec)?)?, "attrcount": sac, "size": self.u(v, sb.size, 2)?}));
            }
            let attr = self.attrs(self.p(bv, bd.attr)?, ac)?;
            let mut child = Vec::new();
            for k in 0..32 {
                child.push(self.u(ev, el.child + k, 1)?);
            }
            elements.push(json!({"name": self.name(ev, el.name)?,
                "type": self.u(ev, el.kind, if self.l.p == 8 { 4 } else { 1 })?,
                "body": {"attr": attr, "sub": if subs.is_empty() { Value::Null } else { json!(subs) },
                    "defaultRecord": self.defrec(self.p(bv, bd.defrec)?)?, "attrcount": ac, "subcount": sc,
                    "size": self.u(bv, bd.size, 2)?, "keyoffset": self.u(bv, bd.keyoffset, 2)?}, "child": child}));
        }
        Ok(
            json!({"name": self.name(v, td.name)?, "type": self.u(v, td.kind, 2)? as i16,
            "major_ver": self.u(v, td.major, 2)? as i16, "minor_ver": self.u(v, td.minor, 2)? as i16,
            "el": elements, "elcount": elc, "maxid": self.u(v, td.maxid, 4)?,
            "autokey": self.u(v, td.autokey, 1)? != 0, "module": self.u(v, td.module, 4)?}),
        )
    }
}
pub fn type_name(code: u8) -> Option<&'static str> {
    Some(match code {
        1 => "int8",
        2 => "int16",
        3 => "int32",
        4 => "int64",
        5 => "float32",
        6 => "bool",
        7 => "string",
        8 => "seq",
        9 => "seq16",
        10 => "ref",
        11 => "tref",
        12 => "sub",
        15 => "vector32",
        16 => "icolor",
        18 => "box",
        20 => "msec",
        21 => "distance",
        22 => "velocity",
        23 => "prop_seq",
        24 => "prop_field",
        25 => "script_obj",
        26 => "native",
        28 => "icon",
        30 => "time64",
        _ => return None,
    })
}
pub fn tables(
    pe: &Pe<'_>,
    l: &'static Layout,
    array: &Array,
) -> Result<(Vec<Value>, Vec<u8>), Error> {
    let mut reader = Reader {
        pe,
        l,
        unknown: BTreeSet::new(),
        remaining: 2_000_000,
    };
    let mut tables = Vec::new();
    for &v in &array.definitions {
        tables.push(reader.table(v)?);
    }
    Ok((tables, reader.unknown.into_iter().collect()))
}
