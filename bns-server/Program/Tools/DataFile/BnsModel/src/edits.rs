use super::*;
use bnsdata::table::ScalarValue;

pub(super) fn set_scalar(
    record: &mut Record,
    def: &TableDefinition,
    field: &str,
    repeat: usize,
    value: &Value,
) -> Result<()> {
    let value = match value {
        Value::Int(v) => ScalarValue::Int(*v),
        Value::Float(v) => ScalarValue::Float(*v),
        Value::Bool(v) => ScalarValue::Bool(*v),
        Value::Enum { index, .. } => ScalarValue::Enum(*index),
        Value::Ref { key, .. } => ScalarValue::Ref(key.clone()),
        _ => return Err(bad("value is not an editable scalar")),
    };
    record.set_scalar(def, field, repeat, value)
}
impl Project {
    fn value(&self, key: &FieldKey) -> Result<Value> {
        self.current_fields(key.physical, key.record)?
            .into_iter()
            .find(|f| f.name == key.field && f.repeat_index == key.repeat)
            .map(|f| f.value)
            .ok_or(bad("field not found"))
    }
    fn check_value(&self, key: &FieldKey, new: &Value) -> Result<()> {
        let old = self.value(key)?;
        if matches!(&old, Value::Raw { .. }) {
            return Err(bad("raw fields are read-only"));
        }
        match (&old, new) {
            (Value::Text(_), Value::Text(units))
            | (Value::Native { .. }, Value::Native { text: units, .. }) => {
                if units.contains(&0) {
                    return Err(bad("text contains NUL"));
                }
                if matches!((&old,new), (Value::Native { a, .. },Value::Native { a: b, .. }) if a != b)
                {
                    return Err(bad("native word a cannot change"));
                }
            }
            (_, Value::Text(_) | Value::Native { .. } | Value::Raw { .. }) => {
                return Err(bad("value type is not editable"))
            }
            (Value::Ref { table: a, .. }, Value::Ref { table: b, .. }) if a != b => {
                return Err(bad("reference table cannot change"))
            }
            _ => {
                let mut record = self.physical[key.physical]
                    .record_pool(key.record)?
                    .0
                    .clone();
                set_scalar(
                    &mut record,
                    self.definition(key.physical)?,
                    &key.field,
                    key.repeat,
                    new,
                )?;
            }
        }
        Ok(())
    }
    fn value_problems(
        &self,
        table: TableId,
        key: &FieldKey,
        value: &Value,
    ) -> Result<Vec<Problem>> {
        let a = self.attribute(key.physical, key.record, &key.field, key.repeat)?;
        let mut problems = Vec::new();
        let mut add = |severity, message: &str| {
            problems.push(Problem {
                table,
                record: key.record,
                field: key.field.clone(),
                repeat: key.repeat,
                severity,
                message: message.to_owned(),
            })
        };
        match value {
            Value::Int(v) if (a.min != 0 || a.max != 0) && (*v < a.min || *v > a.max) => {
                add(Severity::Error, "integer outside schema range")
            }
            Value::Float(v)
                if (a.fmin != 0.0 || a.fmax != 0.0)
                    && (!v.is_finite() || *v < a.fmin || *v > a.fmax) =>
            {
                add(Severity::Error, "float outside schema range")
            }
            Value::Enum { index, .. }
                if usize::try_from(*index)
                    .ok()
                    .is_none_or(|i| a.seq.as_ref().is_none_or(|s| i >= s.len())) =>
            {
                add(Severity::Error, "enum index outside names")
            }
            Value::Ref {
                table, key: bytes, ..
            } if bytes.iter().any(|b| *b != 0)
                && self.resolve(key.physical, *table, bytes)?.is_none() =>
            {
                add(Severity::Warning, "unresolved reference")
            }
            Value::Text(v) | Value::Native { text: v, .. } if a.required != 0 && v.is_empty() => {
                add(Severity::Error, "required text is empty")
            }
            _ => (),
        }
        Ok(problems)
    }
    /// Check the source edit. Mirror checks also run in apply, before either view changes.
    pub fn validate(&self, edit: &Edit) -> Result<Vec<Problem>> {
        let key = FieldKey {
            physical: self.physical_id(edit.table)?,
            record: edit.record,
            field: edit.field.clone(),
            repeat: edit.repeat,
        };
        if self.value(&key)? != edit.old {
            return Err(bad("stale edit old value"));
        }
        self.check_value(&key, &edit.new)?;
        self.value_problems(edit.table, &key, &edit.new)
    }
    fn mirror(&self, p: usize) -> Result<Option<usize>> {
        let source = &self.sources[self.physical[p].file].container;
        let other_file = self
            .sources
            .iter()
            .position(|s| s.container.role == source.role && s.container.width != source.width);
        let Some(file) = other_file else {
            return Ok(None);
        };
        let name = &self.definition(p)?.name;
        self.physical
            .iter()
            .enumerate()
            .find_map(|(i, q)| {
                (q.file == file && self.definition(i).is_ok_and(|d| &d.name == name)).then_some(i)
            })
            .map(Some)
            .ok_or(bad("mirror table is missing"))
    }
    pub fn apply(&mut self, edit: Edit) -> Result<()> {
        let p = self.physical_id(edit.table)?;
        let key = FieldKey {
            physical: p,
            record: edit.record,
            field: edit.field.clone(),
            repeat: edit.repeat,
        };
        if self.value(&key)? != edit.old {
            return Err(bad("stale edit old value"));
        }
        if edit.old == edit.new {
            return Ok(());
        }
        let mut writes = Vec::new();
        for target in std::iter::once(p).chain(self.mirror(p)?) {
            let key = FieldKey {
                physical: target,
                ..key.clone()
            };
            let old = self.value(&key).map_err(|error| {
                if target == p {
                    error
                } else {
                    bad("mirror record or field is missing")
                }
            })?;
            // Native a belongs to the physical record, not the selected width.
            let new = match (&old, &edit.new) {
                (Value::Native { a, .. }, Value::Native { text, .. }) if target != p => {
                    Value::Native {
                        a: *a,
                        text: text.clone(),
                    }
                }
                _ => edit.new.clone(),
            };
            self.check_value(&key, &new).map_err(|error| {
                if target == p {
                    error
                } else {
                    bad("mirror field type or storage check failed")
                }
            })?;
            if self
                .value_problems(edit.table, &key, &new)?
                .iter()
                .any(|p| p.severity == Severity::Error)
            {
                return Err(bad(if target == p {
                    "edit fails schema validation"
                } else {
                    "mirror edit fails schema validation"
                }));
            }
            writes.push(Write { key, old, new });
        }
        let transaction = Transaction { writes };
        self.commit(&transaction, true)?;
        self.undo.push(transaction);
        self.redo.clear();
        Ok(())
    }
    fn commit(&mut self, transaction: &Transaction, forward: bool) -> Result<()> {
        // Read all baselines before changing the overlay.
        let mut values = Vec::new();
        for write in &transaction.writes {
            let value = if forward { &write.new } else { &write.old };
            let baseline = self
                .base_fields(write.key.physical, write.key.record)?
                .into_iter()
                .find(|f| f.name == write.key.field && f.repeat_index == write.key.repeat)
                .ok_or(bad("history field missing"))?
                .value;
            values.push((
                write.key.clone(),
                (value != &baseline).then(|| value.clone()),
            ));
        }
        for (key, value) in values {
            if let Some(value) = value {
                self.overlay.insert(key, value);
            } else {
                self.overlay.remove(&key);
            }
        }
        self.keys.borrow_mut().clear();
        Ok(())
    }
    pub fn undo(&mut self) -> Result<bool> {
        let Some(t) = self.undo.last().cloned() else {
            return Ok(false);
        };
        self.commit(&t, false)?;
        self.undo.pop();
        self.redo.push(t);
        Ok(true)
    }
    pub fn redo(&mut self) -> Result<bool> {
        let Some(t) = self.redo.last().cloned() else {
            return Ok(false);
        };
        self.commit(&t, true)?;
        self.redo.pop();
        self.undo.push(t);
        Ok(true)
    }
    /// Net changes in the logical view, not the undo event log.
    pub fn changes(&self) -> Result<Vec<Edit>> {
        let mut result = Vec::new();
        for (table, &p) in self.logical.iter().enumerate() {
            for (key, new) in self.overlay.iter().filter(|(k, _)| k.physical == p) {
                let old = self
                    .base_fields(p, key.record)?
                    .into_iter()
                    .find(|f| f.name == key.field && f.repeat_index == key.repeat)
                    .ok_or(bad("change field missing"))?
                    .value;
                result.push(Edit {
                    table: TableId(table),
                    record: key.record,
                    field: key.field.clone(),
                    repeat: key.repeat,
                    old,
                    new: new.clone(),
                });
            }
        }
        Ok(result)
    }
    /// Discard one net change. This operation can itself be undone.
    pub fn discard(&mut self, change: &Edit) -> Result<()> {
        if !self.changes()?.contains(change) {
            return Err(bad("stale or unknown change"));
        }
        let p = self.physical_id(change.table)?;
        let mut writes = Vec::new();
        for target in std::iter::once(p).chain(self.mirror(p)?) {
            let key = FieldKey {
                physical: target,
                record: change.record,
                field: change.field.clone(),
                repeat: change.repeat,
            };
            let old = self.value(&key)?;
            let new = self
                .base_fields(target, change.record)?
                .into_iter()
                .find(|f| f.name == change.field && f.repeat_index == change.repeat)
                .ok_or(bad("discard field missing"))?
                .value;
            writes.push(Write { key, old, new });
        }
        let t = Transaction { writes };
        self.commit(&t, true)?;
        self.undo.push(t);
        self.redo.clear();
        Ok(())
    }
    pub fn problems(&self) -> Result<Vec<Problem>> {
        let mut result = Vec::new();
        let changed: std::collections::BTreeSet<_> = self
            .overlay
            .keys()
            .map(|k| (k.physical, k.record))
            .collect();
        for (p, index) in changed {
            let table = self
                .logical
                .iter()
                .position(|i| *i == p)
                .or_else(|| {
                    self.logical.iter().position(|&i| {
                        self.definition(i)
                            .is_ok_and(|a| self.definition(p).is_ok_and(|b| a.name == b.name))
                    })
                })
                .map(TableId)
                .ok_or(bad("changed table has no logical view"))?;
            for field in self.current_fields(p, index)? {
                let key = FieldKey {
                    physical: p,
                    record: index,
                    field: field.name,
                    repeat: field.repeat_index,
                };
                result.extend(self.value_problems(table, &key, &field.value)?);
            }
        }
        Ok(result)
    }
}
