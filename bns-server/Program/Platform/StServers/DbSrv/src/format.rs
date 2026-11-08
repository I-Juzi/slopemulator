//! SQL value → reply text, as the ODBC driver converts to SQL_C_CHAR or
//! SQL_C_WCHAR (sub_1400D35E0 reads every column as text).
//!
//! | SQL type | Text |
//! | --- | --- |
//! | integer types | decimal |
//! | bit | `0` / `1` |
//! | decimal, numeric | `scale` digits; no `0` before the point (`.50`, `-.50`) ❓ |
//! | money, smallmoney | 4 decimals, same rule ❓ |
//! | float, real | shortest round-trip text ❓ |
//! | datetime | `YYYY-MM-DD hh:mm:ss.fff` |
//! | smalldatetime | `YYYY-MM-DD hh:mm:ss` |
//! | datetime2(n), time(n) | `n` fraction digits |
//! | datetimeoffset(n) | `… +hh:mm` |
//! | uniqueidentifier | upper-case `8-4-4-4-12` |
//! | binary types | upper-case hex |
//! | char types, xml | the text (UTF-8) |

use tiberius::{ColumnData, ColumnType};

fn civil(days_since_0001: i64) -> (i64, u32, u32) {
    // Days since 0001-01-01 → (year, month, day), proleptic Gregorian.
    let z = days_since_0001 - 719_162 + 719_468; // to days since 1970-03-01 shift
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + (m <= 2) as i64;
    (y, m, d)
}

/// Days from 0001-01-01 to 1900-01-01.
const DAYS_1900: i64 = 693_595;

fn date(days_since_0001: i64) -> String {
    let (y, m, d) = civil(days_since_0001);
    format!("{y:04}-{m:02}-{d:02}")
}

fn time(increments: u64, scale: u8) -> String {
    let unit = 10u64.pow(scale as u32);
    let secs = increments / unit;
    let frac = increments % unit;
    let mut s = format!("{:02}:{:02}:{:02}", secs / 3600, secs / 60 % 60, secs % 60);
    if scale > 0 {
        s.push_str(&format!(".{frac:0width$}", width = scale as usize));
    }
    s
}

/// Decimal text with `scale` digits and no leading `0` before the point.
fn decimal(value: i128, scale: u8) -> String {
    let neg = value < 0;
    let digits = value.unsigned_abs().to_string();
    let scale = scale as usize;
    let (int, frac) = if digits.len() > scale {
        digits.split_at(digits.len() - scale)
    } else {
        ("", digits.as_str())
    };
    let frac = format!("{frac:0>scale$}");
    let int = if int == "0" { "" } else { int };
    let mut s = String::new();
    if neg {
        s.push('-');
    }
    s.push_str(int);
    if scale > 0 {
        s.push('.');
        s.push_str(&frac);
    }
    if s.is_empty() || s == "-" {
        s = "0".into();
    }
    s
}

/// Text of one cell; `None` = NULL (the column element is left out).
pub fn cell(ty: ColumnType, data: &ColumnData<'_>) -> Option<String> {
    Some(match data {
        ColumnData::U8(v) => (*v)?.to_string(),
        ColumnData::I16(v) => (*v)?.to_string(),
        ColumnData::I32(v) => (*v)?.to_string(),
        ColumnData::I64(v) => (*v)?.to_string(),
        ColumnData::F32(v) => {
            let v = (*v)?;
            match ty {
                ColumnType::Money4 => decimal((v as f64 * 10_000.0).round() as i128, 4),
                _ => v.to_string(),
            }
        }
        ColumnData::F64(v) => {
            let v = (*v)?;
            match ty {
                ColumnType::Money | ColumnType::Money4 => decimal((v * 10_000.0).round() as i128, 4),
                _ => v.to_string(),
            }
        }
        ColumnData::Bit(v) => (if (*v)? { "1" } else { "0" }).to_owned(),
        ColumnData::String(v) => v.as_ref()?.to_string(),
        ColumnData::Guid(v) => v.as_ref()?.to_string().to_uppercase(),
        ColumnData::Binary(v) => v.as_ref()?.iter().map(|b| format!("{b:02X}")).collect(),
        ColumnData::Numeric(v) => {
            let n = (*v)?;
            decimal(n.value(), n.scale())
        }
        ColumnData::Xml(v) => v.as_ref()?.to_string(),
        ColumnData::DateTime(v) => {
            let v = (*v)?;
            // 1/300 s ticks → milliseconds, rounded like SQL Server.
            let ms = (v.seconds_fragments() as u64 * 10 + 1) / 3;
            format!("{} {}", date(DAYS_1900 + v.days() as i64), time(ms, 3))
        }
        ColumnData::SmallDateTime(v) => {
            let v = (*v)?;
            format!("{} {}", date(DAYS_1900 + v.days() as i64), time(v.seconds_fragments() as u64 * 60, 0))
        }
        ColumnData::Time(v) => {
            let v = (*v)?;
            time(v.increments(), v.scale())
        }
        ColumnData::Date(v) => date((*v)?.days() as i64),
        ColumnData::DateTime2(v) => {
            let v = (*v)?;
            format!("{} {}", date(v.date().days() as i64), time(v.time().increments(), v.time().scale()))
        }
        ColumnData::DateTimeOffset(v) => {
            let v = (*v)?;
            let dt = v.datetime2();
            // The stored value is UTC; the text shows local time and the offset.
            let scale = dt.time().scale();
            let unit = 10i128.pow(scale as u32);
            let total = (dt.date().days() as i128 * 86_400 * unit) + dt.time().increments() as i128 + v.offset() as i128 * 60 * unit;
            let days = total.div_euclid(86_400 * unit);
            let rest = total.rem_euclid(86_400 * unit) as u64;
            let off = v.offset();
            let sign = if off < 0 { '-' } else { '+' };
            format!("{} {} {sign}{:02}:{:02}", date(days as i64), time(rest, scale), off.unsigned_abs() / 60, off.unsigned_abs() % 60)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiberius::numeric::Numeric;
    use tiberius::time::{Date, DateTime, DateTime2, Time};

    #[test]
    fn dates() {
        // 2016-10-20 = 42661 days after 1900-01-01; 12:34:56.789 ≈ 13577037 ticks.
        let ticks = ((12 * 3600 + 34 * 60 + 56) * 300) as u32 + 237; // .790
        assert_eq!(cell(ColumnType::Datetime, &ColumnData::DateTime(Some(DateTime::new(42661, ticks)))).unwrap(), "2016-10-20 12:34:56.790");
        assert_eq!(date(0), "0001-01-01");
        let d2 = DateTime2::new(Date::new(DAYS_1900 as u32), Time::new(1234567, 7));
        assert_eq!(cell(ColumnType::Datetime2, &ColumnData::DateTime2(Some(d2))).unwrap(), "1900-01-01 00:00:00.1234567");
    }

    #[test]
    fn numbers() {
        assert_eq!(decimal(50, 2), ".50");
        assert_eq!(decimal(-50, 2), "-.50");
        assert_eq!(decimal(12345, 2), "123.45");
        assert_eq!(decimal(7, 0), "7");
        assert_eq!(decimal(0, 0), "0");
        let n = Numeric::new_with_scale(-1205, 1);
        assert_eq!(cell(ColumnType::Decimaln, &ColumnData::Numeric(Some(n))).unwrap(), "-120.5");
        assert_eq!(cell(ColumnType::Money, &ColumnData::F64(Some(1234.56))).unwrap(), "1234.5600");
        assert_eq!(cell(ColumnType::Int4, &ColumnData::I32(None)), None);
        assert_eq!(cell(ColumnType::BigVarBin, &ColumnData::Binary(Some(vec![0xAB, 1].into()))).unwrap(), "AB01");
    }
}
