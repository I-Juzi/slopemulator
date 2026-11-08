//! Operator-supplied, width-specific JSON definitions. No definitions are embedded.
use crate::{Error, Result};
use serde::{Deserialize, Deserializer};
use std::{collections::BTreeMap, path::Path};

fn null_vec<'de, D, T>(d: D) -> std::result::Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(d)?.unwrap_or_default())
}

#[derive(Clone, Debug, Deserialize)]
pub struct Attribute {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub offset: usize,
    /// Inline stride, not the total size of all repeats.
    pub size: usize,
    pub repeat: usize,
    #[serde(default)]
    pub key: bool,
    #[serde(default)]
    pub required: i32,
    #[serde(default)]
    pub deprecated: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub seq: Option<Vec<String>>,
    #[serde(default)]
    pub reftable: i32,
    #[serde(default)]
    pub refel: i32,
    #[serde(default)]
    pub min: i64,
    #[serde(default)]
    pub max: i64,
    #[serde(default)]
    pub fmin: f32,
    #[serde(default)]
    pub fmax: f32,
    #[serde(default)]
    pub defvalue: serde_json::Value,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Subtype {
    pub name: String,
    pub size: usize,
    pub attrcount: usize,
    #[serde(default, deserialize_with = "null_vec")]
    pub attr: Vec<Attribute>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Body {
    pub size: usize,
    pub keyoffset: usize,
    pub attrcount: usize,
    pub subcount: usize,
    #[serde(default, deserialize_with = "null_vec")]
    pub attr: Vec<Attribute>,
    #[serde(default, deserialize_with = "null_vec")]
    pub sub: Vec<Subtype>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Element {
    pub name: String,
    #[serde(rename = "type")]
    pub type_id: i8,
    pub body: Body,
    #[serde(default, deserialize_with = "null_vec")]
    pub child: Vec<i32>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct TableDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub type_id: u16,
    pub major_ver: u16,
    pub minor_ver: u16,
    pub autokey: bool,
    pub maxid: i64,
    pub module: u32,
    pub elcount: usize,
    pub el: Vec<Element>,
}

/// Base fields followed by subtype fields. Offsets use the DrEl record base.
pub struct Layout<'a> {
    pub size: usize,
    pub keyoffset: usize,
    pub base_fields: &'a [Attribute],
    pub subtype_fields: &'a [Attribute],
}
impl Layout<'_> {
    pub fn fields(&self) -> impl Iterator<Item = &Attribute> {
        self.base_fields.iter().chain(self.subtype_fields)
    }
}
impl TableDefinition {
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        let value: Self = serde_json::from_slice(bytes)
            .map_err(|_| Error::Invalid("invalid table definition JSON"))?;
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<()> {
        if self.elcount != self.el.len() {
            return Err(Error::Invalid("definition element count mismatch"));
        }
        let mut ids = std::collections::BTreeSet::new();
        for el in &self.el {
            if !ids.insert(el.type_id)
                || el.body.attrcount != el.body.attr.len()
                || el.body.subcount != el.body.sub.len()
                || el.body.sub.iter().any(|s| s.attrcount != s.attr.len())
            {
                return Err(Error::Invalid("definition count or element ID mismatch"));
            }
        }
        Ok(())
    }
    pub fn layout(&self, element_type: i8, subtype: i16) -> Result<Layout<'_>> {
        let body = &self
            .el
            .iter()
            .find(|e| e.type_id == element_type)
            .ok_or(Error::Invalid("unknown element type"))?
            .body;
        let (size, subtype_fields) = if subtype < 0 {
            (body.size, &[][..])
        } else {
            let sub = body
                .sub
                .get(subtype as usize)
                .ok_or(Error::Invalid("unknown record subtype"))?;
            (sub.size, sub.attr.as_slice())
        };
        Ok(Layout {
            size,
            keyoffset: body.keyoffset,
            base_fields: &body.attr,
            subtype_fields,
        })
    }
}

#[derive(Debug, Default)]
pub struct Definitions(pub BTreeMap<u16, TableDefinition>);
impl Definitions {
    pub fn from_definitions(
        definitions: impl IntoIterator<Item = TableDefinition>,
    ) -> Result<Self> {
        let mut result = Self::default();
        for definition in definitions {
            definition.validate()?;
            if result.0.insert(definition.type_id, definition).is_some() {
                return Err(Error::Invalid("duplicate table definition ID"));
            }
        }
        Ok(result)
    }
    /// Read immediate `.json` files only. The caller must select the correct width.
    pub fn load_folder(path: impl AsRef<Path>) -> Result<Self> {
        let mut paths = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() && entry.path().extension().is_some_and(|s| s == "json")
            {
                paths.push(entry.path());
            }
        }
        paths.sort();
        let mut definitions = Vec::new();
        for path in paths {
            definitions.push(TableDefinition::from_json(&std::fs::read(path)?)?);
        }
        if definitions.is_empty() {
            return Err(Error::Invalid("definition folder has no JSON files"));
        }
        Self::from_definitions(definitions)
    }
    pub fn get(&self, type_id: u16) -> Result<&TableDefinition> {
        self.0
            .get(&type_id)
            .ok_or(Error::Invalid("missing table definition"))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub(crate) fn definition() -> TableDefinition {
        TableDefinition::from_json(
            br#"{
          "name":"synthetic", "type":7, "major_ver":1, "minor_ver":2,
          "autokey":false, "maxid":0, "module":0, "elcount":1,
          "el":[{"name":"record", "type":1, "body":{
            "size":16, "keyoffset":8, "attrcount":1, "subcount":1,
            "attr":[{"name":"id", "type":"int64", "offset":8, "size":8, "repeat":1, "key":true}],
            "sub":[{"name":"extended", "size":20, "attrcount":1,
              "attr":[{"name":"value", "type":"int32", "offset":16, "size":4, "repeat":1}]}]
          }, "child":null}]
        }"#,
        )
        .unwrap()
    }
    #[test]
    fn synthetic_map_and_subtype_lookup() {
        let definitions = Definitions::from_definitions([definition()]).unwrap();
        let def = definitions.get(7).unwrap();
        assert_eq!(def.layout(1, -1).unwrap().size, 16);
        let sub = def.layout(1, 0).unwrap();
        assert_eq!(sub.size, 20);
        assert_eq!(
            sub.fields().map(|a| a.name.as_str()).collect::<Vec<_>>(),
            ["id", "value"]
        );
        assert!(def.layout(2, -1).is_err());
        assert!(def.layout(1, 1).is_err());
        assert!(definitions.get(8).is_err());
        assert!(Definitions::from_definitions([definition(), definition()]).is_err());
    }
    #[test]
    fn null_arrays_and_bad_json() {
        let mut def = definition();
        def.el[0].body.attrcount = 0;
        assert!(def.validate().is_err());
        assert!(TableDefinition::from_json(b"{}").is_err());
        let bytes = br#"{"name":"empty","type":1,"major_ver":0,"minor_ver":0,"autokey":true,"maxid":0,"module":0,"elcount":1,"el":[{"name":"r","type":1,"body":{"size":6,"keyoffset":0,"attrcount":0,"subcount":0,"attr":null,"sub":null}}]}"#;
        assert_eq!(
            TableDefinition::from_json(bytes)
                .unwrap()
                .layout(1, -1)
                .unwrap()
                .size,
            6
        );
    }
}
