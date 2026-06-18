use std::collections::BTreeMap;

use crate::error::AppError;
use crate::models::{CommandExecutionResult, CommandExecutionStatus};

fn parse_json_input(input: &str) -> Result<serde_json::Value, AppError> {
    serde_json::from_str(input.trim())
        .map_err(|e| AppError::Internal(format!("invalid JSON: {}", e)))
}

// ============================================================
// 022 - Schema Inter-Converter Engine
// ============================================================
pub fn schema_convert(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "data.schema-convert".into(),
            title: "Schema Convert".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste data with a target format prefix, e.g.:\nto: yaml\n{\"key\": \"value\"}".into(),
        });
    }

    let (directive, body) = if let Some(pos) = trimmed.find('\n') {
        let first = trimmed[..pos].trim().to_lowercase();
        let rest = trimmed[pos..].trim();
        (first, rest)
    } else {
        (String::new(), trimmed)
    };

    let target = if let Some(t) = directive.strip_prefix("to:") {
        t.trim()
    } else if let Some(t) = directive.strip_prefix("as:") {
        t.trim()
    } else {
        "yaml"
    };

    // Detect source format and parse
    let value = if body.trim().starts_with('{') || body.trim().starts_with('[') {
        parse_json_input(body)?
    } else if body.contains(": ") && !body.contains("<?xml") {
        // Try YAML or TOML parsing
        serde_yaml::from_str(body)
            .or_else(|_| {
                let toml_val: toml::Value = toml::from_str(body)
                    .map_err(|e| AppError::Internal(format!("cannot parse source: {}", e)))?;
                toml_value_to_json(&toml_val)
            })?
    } else if body.trim().starts_with("<?xml") || body.trim().starts_with('<') {
        // Parse simple XML
        parse_simple_xml(body)?
    } else {
        // Try JSON first, then YAML
        parse_json_input(body).or_else(|_| {
            serde_yaml::from_str(body)
                .map_err(|e| AppError::Internal(format!("cannot parse input: {}", e)))
        })?
    };

    let output = match target {
        "yaml" | "yml" => serde_yaml::to_string(&value)
            .map_err(|e| AppError::Internal(format!("YAML conversion failed: {}", e)))?,
        "toml" => {
            let table = value_to_toml(&value);
            toml::to_string(&table)
                .map_err(|e| AppError::Internal(format!("TOML conversion failed: {}", e)))?
        }
        "xml" => value_to_xml(&value),
        "csv" => value_to_csv(&value),
        _ => serde_json::to_string_pretty(&value)
            .map_err(|e| AppError::Internal(format!("JSON conversion failed: {}", e)))?,
    };

    Ok(CommandExecutionResult {
        command_id: "data.schema-convert".into(),
        title: format!("Converted to {}", target.to_uppercase()),
        output,
        status: CommandExecutionStatus::Success,
        summary: format!("Input converted to {} format", target.to_uppercase()),
    })
}

fn parse_simple_xml(input: &str) -> Result<serde_json::Value, AppError> {
    let mut map = serde_json::Map::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '<' {
            let mut tag = String::new();
            for c in chars.by_ref() {
                if c == '>' || c == ' ' || c == '/' { break; }
                tag.push(c);
            }
            if tag.is_empty() || tag.starts_with('?') { continue; }
            let mut content = String::new();
            let end_tag = format!("</{}>", tag);
            for c in chars.by_ref() {
                content.push(c);
                if content.ends_with(&end_tag) {
                    content = content[..content.len() - end_tag.len()].to_string();
                    break;
                }
            }
            let value = if content.trim().is_empty() {
                serde_json::Value::Null
            } else if content.contains('<') {
                parse_simple_xml(&content)?
            } else if let Ok(n) = content.trim().parse::<f64>() {
                serde_json::Value::Number(serde_json::Number::from_f64(n).unwrap_or(0.into()))
            } else {
                serde_json::Value::String(content.trim().to_string())
            };
            if map.contains_key(&tag) {
                let existing = map.remove(&tag).unwrap_or(serde_json::Value::Null);
                let arr = match existing {
                    serde_json::Value::Array(mut a) => { a.push(value); serde_json::Value::Array(a) }
                    other => serde_json::Value::Array(vec![other, value]),
                };
                map.insert(tag, arr);
            } else {
                map.insert(tag, value);
            }
        }
    }
    Ok(serde_json::Value::Object(map))
}

fn toml_value_to_json(v: &toml::Value) -> Result<serde_json::Value, AppError> {
    match v {
        toml::Value::String(s) => Ok(serde_json::Value::String(s.clone())),
        toml::Value::Integer(i) => Ok(serde_json::Value::Number((*i).into())),
        toml::Value::Float(f) => {
            serde_json::Number::from_f64(*f).map(serde_json::Value::Number)
                .ok_or_else(|| AppError::Internal("invalid float value".into()))
        }
        toml::Value::Boolean(b) => Ok(serde_json::Value::Bool(*b)),
        toml::Value::Datetime(dt) => Ok(serde_json::Value::String(dt.to_string())),
        toml::Value::Array(arr) => {
            let items: Result<Vec<_>, _> = arr.iter().map(toml_value_to_json).collect();
            items.map(serde_json::Value::Array)
        }
        toml::Value::Table(table) => {
            let mut map = serde_json::Map::new();
            for (k, val) in table {
                map.insert(k.clone(), toml_value_to_json(val)?);
            }
            Ok(serde_json::Value::Object(map))
        }
    }
}

fn value_to_toml(v: &serde_json::Value) -> toml::Value {
    match v {
        serde_json::Value::Null => toml::Value::String("".into()),
        serde_json::Value::Bool(b) => toml::Value::Boolean(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() { toml::Value::Integer(i) }
            else if let Some(f) = n.as_f64() { toml::Value::Float(f) }
            else { toml::Value::String(n.to_string()) }
        }
        serde_json::Value::String(s) => toml::Value::String(s.clone()),
        serde_json::Value::Array(arr) => {
            let vec: Vec<toml::Value> = arr.iter().map(value_to_toml).collect();
            toml::Value::Array(vec)
        }
        serde_json::Value::Object(obj) => {
            let mut table = toml::value::Table::new();
            for (k, v) in obj {
                table.insert(k.clone(), value_to_toml(v));
            }
            toml::Value::Table(table)
        }
    }
}

fn value_to_xml(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Object(obj) => {
            let mut xml = String::new();
            for (key, val) in obj {
                xml.push_str(&format!("<{}>", key));
                xml.push_str(&value_to_xml(val));
                xml.push_str(&format!("</{}>", key));
            }
            xml
        }
        serde_json::Value::Array(arr) => {
            arr.iter().map(value_to_xml).collect::<Vec<_>>().join("\n")
        }
        serde_json::Value::String(s) => {
            s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
        }
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
    }
}

fn value_to_csv(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Array(arr) => {
            let mut wtr = csv::Writer::from_writer(Vec::new());
            let mut headers: Vec<String> = Vec::new();
            for item in arr {
                if let serde_json::Value::Object(obj) = item {
                    if headers.is_empty() {
                        headers = obj.keys().cloned().collect();
                        if wtr.write_record(&headers).is_err() { break; }
                    }
                    let row: Vec<String> = headers.iter()
                        .map(|h| obj.get(h).map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        }).unwrap_or_default())
                        .collect();
                    if wtr.write_record(&row).is_err() { break; }
                }
            }
            String::from_utf8(wtr.into_inner().unwrap_or_default()).unwrap_or_default()
        }
        serde_json::Value::Object(obj) => {
            let mut wtr = csv::Writer::from_writer(Vec::new());
            let headers: Vec<String> = obj.keys().cloned().collect();
            if wtr.write_record(&headers).is_ok() {
                let row: Vec<String> = headers.iter()
                    .map(|h| obj.get(h).map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    }).unwrap_or_default())
                    .collect();
                let _ = wtr.write_record(&row);
            }
            String::from_utf8(wtr.into_inner().unwrap_or_default()).unwrap_or_default()
        }
        _ => v.to_string(),
    }
}

// ============================================================
// 023 - Smart JSON Flatten & Unflatten
// ============================================================
pub fn json_flatten(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "data.json-flatten".into(),
            title: "JSON Flatten".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste a nested JSON object to flatten into dot-notation paths.".into(),
        });
    }

    let value = parse_json_input(trimmed)?;
    let mut flat = BTreeMap::new();
    flatten_value(&value, String::new(), &mut flat);

    let mut output = String::new();
    for (path, val) in &flat {
        output.push_str(&format!("{} = {}\n", path, val));
    }

    Ok(CommandExecutionResult {
        command_id: "data.json-flatten".into(),
        title: "JSON Flattened".into(),
        output,
        status: CommandExecutionStatus::Success,
        summary: format!("{} path(s) flattened", flat.len()),
    })
}

fn flatten_value(v: &serde_json::Value, prefix: String, result: &mut BTreeMap<String, String>) {
    match v {
        serde_json::Value::Object(obj) => {
            for (k, val) in obj {
                let path = if prefix.is_empty() { k.clone() } else { format!("{}.{}", prefix, k) };
                flatten_value(val, path, result);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let path = format!("{}[{}]", prefix, i);
                flatten_value(val, path, result);
            }
        }
        other => {
            result.insert(prefix, match other {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => "null".into(),
                v => v.to_string(),
            });
        }
    }
}

pub fn json_unflatten(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "data.json-unflatten".into(),
            title: "JSON Unflatten".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste flattened key=value or key: value pairs to rebuild nested JSON.".into(),
        });
    }

    let mut root = serde_json::Value::Object(serde_json::Map::new());

    for line in trimmed.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }

        let (path, raw_val) = if let Some(pos) = line.find(" = ") {
            (line[..pos].trim(), line[pos + 3..].trim())
        } else if let Some(pos) = line.find(": ") {
            (line[..pos].trim(), line[pos + 2..].trim())
        } else {
            continue;
        };

        let value: serde_json::Value = if raw_val == "null" {
            serde_json::Value::Null
        } else if raw_val == "true" {
            serde_json::Value::Bool(true)
        } else if raw_val == "false" {
            serde_json::Value::Bool(false)
        } else if let Ok(n) = raw_val.parse::<i64>() {
            serde_json::Value::Number(n.into())
        } else if let Ok(n) = raw_val.parse::<f64>() {
            serde_json::Value::Number(serde_json::Number::from_f64(n).unwrap_or(0.into()))
        } else {
            serde_json::Value::String(raw_val.to_string())
        };

        set_nested_value(&mut root, &parse_path_segments(path), value);
    }

    let output = serde_json::to_string_pretty(&root)
        .map_err(|e| AppError::Internal(format!("JSON serialization failed: {}", e)))?;

    Ok(CommandExecutionResult {
        command_id: "data.json-unflatten".into(),
        title: "JSON Unflattened".into(),
        output,
        status: CommandExecutionStatus::Success,
        summary: "Flat paths rebuilt into nested JSON".into(),
    })
}

fn set_nested_value(target: &mut serde_json::Value, segments: &[PathSegment], value: serde_json::Value) {
    if segments.is_empty() {
        *target = value;
        return;
    }

    match &segments[0] {
        PathSegment::Key(key) => {
            if !target.is_object() {
                *target = serde_json::Value::Object(serde_json::Map::new());
            }
            let object = target.as_object_mut().unwrap();
            if segments.len() == 1 {
                object.insert(key.clone(), value);
                return;
            }
            let next = object.entry(key.clone()).or_insert(serde_json::Value::Null);
            set_nested_value(next, &segments[1..], value);
        }
        PathSegment::Index(index) => {
            if !target.is_array() {
                *target = serde_json::Value::Array(Vec::new());
            }
            let array = target.as_array_mut().unwrap();
            while array.len() <= *index {
                array.push(serde_json::Value::Null);
            }
            if segments.len() == 1 {
                array[*index] = value;
                return;
            }
            set_nested_value(&mut array[*index], &segments[1..], value);
        }
    }
}

#[derive(Clone, Debug)]
enum PathSegment {
    Key(String),
    Index(usize),
}

fn parse_path_segments(path: &str) -> Vec<PathSegment> {
    let mut segs = Vec::new();
    let mut current = String::new();
    let mut chars = path.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '.' => {
                if !current.is_empty() {
                    segs.push(PathSegment::Key(std::mem::take(&mut current)));
                }
            }
            '[' => {
                if !current.is_empty() {
                    segs.push(PathSegment::Key(std::mem::take(&mut current)));
                }
                let mut idx = String::new();
                for c in chars.by_ref() {
                    if c == ']' { break; }
                    idx.push(c);
                }
                if let Ok(parsed) = idx.parse::<usize>() {
                    segs.push(PathSegment::Index(parsed));
                } else if !idx.is_empty() {
                    segs.push(PathSegment::Key(idx.trim_matches('\'').trim_matches('"').to_string()));
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        segs.push(PathSegment::Key(current));
    }
    segs
}

// ============================================================
// 024 - Dynamic SQL Statement Beautifier
// ============================================================
pub fn sql_beautify(input: &str) -> Result<CommandExecutionResult, AppError> {
    let sql = input.trim();
    if sql.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "data.sql-beautify".into(),
            title: "SQL Beautify".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste raw SQL to format with proper indentation.".into(),
        });
    }

    let keywords = [
        "SELECT", "FROM", "WHERE", "AND", "OR", "ORDER BY", "GROUP BY",
        "HAVING", "LIMIT", "OFFSET", "JOIN", "LEFT JOIN", "RIGHT JOIN",
        "INNER JOIN", "OUTER JOIN", "FULL JOIN", "CROSS JOIN",
        "ON", "INTO", "VALUES", "SET", "INSERT INTO", "UPDATE", "DELETE",
        "CREATE TABLE", "ALTER TABLE", "DROP TABLE", "CREATE INDEX",
        "UNION", "UNION ALL", "INTERSECT", "EXCEPT",
        "CASE", "WHEN", "THEN", "ELSE", "END",
        "AS", "DISTINCT", "EXISTS", "NOT", "IN", "BETWEEN", "LIKE",
        "IS NULL", "IS NOT NULL", "ASC", "DESC",
    ];

    let mut output = String::new();
    let mut indent: usize = 0;
    let mut chars = sql.chars().peekable();
    let mut word = String::new();
    let mut in_string = false;
    let mut string_char = ' ';

    while let Some(ch) = chars.next() {
        if in_string {
            word.push(ch);
            if ch == string_char {
                in_string = false;
            }
            continue;
        }

        if ch == '\'' || ch == '"' {
            if !word.is_empty() {
                flush_word(&word, &mut output, &mut indent, &keywords);
                word.clear();
            }
            in_string = true;
            string_char = ch;
            word.push(ch);
            continue;
        }

        if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
            if !word.is_empty() {
                flush_word(&word, &mut output, &mut indent, &keywords);
                word.clear();
            }
            continue;
        }

        if ch == '(' {
            if !word.is_empty() {
                flush_word(&word, &mut output, &mut indent, &keywords);
                word.clear();
            }
            if !output.ends_with('\n') && !output.is_empty() && !output.ends_with(' ') {
                output.push(' ');
            }
            output.push('(');
            indent += 1;
            output.push('\n');
            output.push_str(&"  ".repeat(indent));
            continue;
        }

        if ch == ')' {
            if !word.is_empty() {
                flush_word(&word, &mut output, &mut indent, &keywords);
                word.clear();
            }
            indent = indent.saturating_sub(1);
            output.push('\n');
            output.push_str(&"  ".repeat(indent));
            output.push(')');
            continue;
        }

        if ch == ',' {
            if !word.is_empty() {
                flush_word(&word, &mut output, &mut indent, &keywords);
                word.clear();
            }
            output.push(',');
            output.push('\n');
            output.push_str(&"  ".repeat(indent));
            continue;
        }

        if ch == ';' {
            if !word.is_empty() {
                flush_word(&word, &mut output, &mut indent, &keywords);
                word.clear();
            }
            output.push(';');
            output.push('\n');
            continue;
        }

        word.push(ch);
    }

    if !word.is_empty() {
        flush_word(&word, &mut output, &mut indent, &keywords);
    }

    let result = output.trim().to_string();
    Ok(CommandExecutionResult {
        command_id: "data.sql-beautify".into(),
        title: "SQL Formatted".into(),
        output: result,
        status: CommandExecutionStatus::Success,
        summary: "SQL query formatted with proper indentation.".into(),
    })
}

fn flush_word(word: &str, output: &mut String, indent: &mut usize, keywords: &[&str]) {
    let upper = word.to_uppercase();
    let is_keyword = keywords.iter().any(|k| {
        let kw_upper = k.to_uppercase();
        upper == kw_upper || upper.starts_with(&format!("{} ", kw_upper))
    });

    if is_keyword && !output.is_empty() {
        output.push('\n');
        output.push_str(&"  ".repeat(*indent));
    } else if !output.is_empty() && !output.ends_with('\n') && !output.ends_with(' ') {
        output.push(' ');
    }

    output.push_str(word);
}

// ============================================================
// 025 - Code to Clean String Escaper
// ============================================================
pub fn escape_text(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "data.escape".into(),
            title: "String Escape".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste text to escape. Prefix with 'html:' or 'json:' (default).".into(),
        });
    }

    let (mode, body) = parse_mode(trimmed);

    let output = match mode.as_str() {
        "html" | "xml" => {
            body.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
                .replace('\'', "&#39;")
        }
        "json" => {
            serde_json::to_string(&body)
                .map(|s| s[1..s.len()-1].to_string())
                .unwrap_or_default()
        }
        "terminal" | "shell" => {
            body.replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('`', "\\`")
                .replace('$', "\\$")
        }
        _ => body.to_string(),
    };

    Ok(CommandExecutionResult {
        command_id: "data.escape".into(),
        title: format!("Escaped ({})", mode.to_uppercase()),
        output,
        status: CommandExecutionStatus::Success,
        summary: format!("Text escaped for {} context", mode),
    })
}

pub fn unescape_text(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "data.unescape".into(),
            title: "String Unescape".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste escaped text to unescape. Prefix with 'html:' or 'json:' (default).".into(),
        });
    }

    let (mode, body) = parse_mode(trimmed);

    let output = match mode.as_str() {
        "html" | "xml" => {
            body.replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&quot;", "\"")
                .replace("&#39;", "'")
                .replace("&amp;", "&")
        }
        "json" => {
            let wrapped = format!("\"{}\"", body);
            serde_json::from_str::<String>(&wrapped)
                .unwrap_or_else(|_| body.to_string())
        }
        "terminal" | "shell" => {
            body.replace("\\\\", "\\")
                .replace("\\\"", "\"")
                .replace("\\`", "`")
                .replace("\\$", "$")
        }
        _ => body.to_string(),
    };

    Ok(CommandExecutionResult {
        command_id: "data.unescape".into(),
        title: format!("Unescaped ({})", mode.to_uppercase()),
        output,
        status: CommandExecutionStatus::Success,
        summary: format!("Text unescaped from {} context", mode),
    })
}

fn parse_mode(input: &str) -> (String, String) {
    let trimmed = input.trim();
    if let Some(pos) = trimmed.find('\n') {
        let first_line = trimmed[..pos].trim().to_lowercase();
        let known = ["html", "xml", "json", "terminal", "shell"];
        if known.contains(&first_line.as_str()) {
            return (first_line, trimmed[pos..].trim().to_string());
        }
        if let Some(mode) = first_line.strip_suffix(':') {
            let m = mode.trim().to_string();
            if known.contains(&m.as_str()) {
                return (m, trimmed[pos..].trim().to_string());
            }
        }
    }
    ("json".into(), trimmed.to_string())
}

// ============================================================
// 026 - Interactive CSV/TSV Visual Matrix
// ============================================================
pub fn csv_table(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "data.csv-table".into(),
            title: "CSV Table".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste CSV or TSV data to render as a formatted table.".into(),
        });
    }

    let delimiter = if trimmed.contains('\t') && !trimmed.contains(',') {
        b'\t'
    } else {
        b','
    };

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(true)
        .from_reader(trimmed.as_bytes());

    let headers = match reader.headers() {
        Ok(h) => h.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        Err(e) => return Ok(CommandExecutionResult {
            command_id: "data.csv-table".into(),
            title: "CSV Error".into(),
            output: format!("Failed to parse CSV: {}", e),
            status: CommandExecutionStatus::Error,
            summary: "Parse error".into(),
        }),
    };

    let mut rows: Vec<Vec<String>> = Vec::new();
    for result in reader.records() {
        match result {
            Ok(rec) => {
                let row: Vec<String> = rec.iter().map(|s| s.to_string()).collect();
                rows.push(row);
            }
            Err(e) => {
                return Ok(CommandExecutionResult {
                    command_id: "data.csv-table".into(),
                    title: "CSV Error".into(),
                    output: format!("Row parse error: {}", e),
                    status: CommandExecutionStatus::Error,
                    summary: "Parse error".into(),
                });
            }
        }
    }

    let mut col_widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in &rows {
        for (i, val) in row.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(val.len());
            }
        }
    }

    let mut output = String::new();
    // Header
    output.push('|');
    for (i, h) in headers.iter().enumerate() {
        output.push(' ');
        output.push_str(&pad_or_truncate(h, col_widths[i]));
        output.push_str(" |");
    }
    output.push('\n');
    // Separator
    output.push('|');
    for w in &col_widths {
        output.push_str(&":".repeat(w + 2));
        output.push('|');
    }
    output.push('\n');
    // Rows
    for row in &rows {
        output.push('|');
        for (i, val) in row.iter().enumerate() {
            output.push(' ');
            if i < col_widths.len() {
                output.push_str(&pad_or_truncate(val, col_widths[i]));
            } else {
                output.push_str(val);
            }
            output.push_str(" |");
        }
        output.push('\n');
    }

    Ok(CommandExecutionResult {
        command_id: "data.csv-table".into(),
        title: format!("{} Table ({} cols x {} rows)", if delimiter == b'\t' { "TSV" } else { "CSV" }, col_widths.len(), rows.len()),
        output,
        status: CommandExecutionStatus::Success,
        summary: format!("{} columns, {} rows", col_widths.len(), rows.len()),
    })
}

fn pad_or_truncate(s: &str, width: usize) -> String {
    if s.len() > width {
        format!("{}..", &s[..width.saturating_sub(2)])
    } else {
        format!("{:<width$}", s, width = width)
    }
}

// ============================================================
// 027 - Type-Definition Automated Generator
// ============================================================
pub fn gen_types(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "data.gen-types".into(),
            title: "Generate Types".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste JSON to generate type definitions. Prefix with:\nts:\n<json> for TypeScript\ngo:\n<json> for Go\nrust:\n<json> for Rust".into(),
        });
    }

    let (lang, body) = parse_gen_lang(trimmed);
    let value = parse_json_input(&body)?;

    let output = match lang.as_str() {
        "go" | "golang" => generate_go_types(&value, "Generated"),
        "rust" => generate_rust_types(&value, "Generated"),
        _ => generate_typescript_types(&value, "Generated"),
    };

    let label = match lang.as_str() {
        "go" | "golang" => "Go",
        "rust" => "Rust",
        _ => "TypeScript",
    };

    Ok(CommandExecutionResult {
        command_id: "data.gen-types".into(),
        title: format!("{} Types Generated", label),
        output,
        status: CommandExecutionStatus::Success,
        summary: format!("{} type definitions generated from JSON", label),
    })
}

fn parse_gen_lang(input: &str) -> (String, String) {
    let trimmed = input.trim();
    if let Some(pos) = trimmed.find('\n') {
        let first_line = trimmed[..pos].trim().to_lowercase();
        let langs = ["ts", "typescript", "go", "golang", "rust"];
        if langs.contains(&first_line.as_str()) {
            return (first_line, trimmed[pos..].trim().to_string());
        }
        if let Some(mode) = first_line.strip_suffix(':') {
            let m = mode.trim().to_string();
            if langs.contains(&m.as_str()) {
                return (m, trimmed[pos..].trim().to_string());
            }
        }
    }
    ("ts".into(), trimmed.to_string())
}

fn type_name_from_key(key: &str) -> String {
    let mut name = String::new();
    let mut cap_next = true;
    for ch in key.chars() {
        if ch == '_' || ch == '-' || ch == ' ' {
            cap_next = true;
        } else if cap_next {
            name.push(ch.to_ascii_uppercase());
            cap_next = false;
        } else {
            name.push(ch);
        }
    }
    if name.is_empty() { name = "Root".into(); }
    name
}

fn json_type_to_ts(v: &serde_json::Value, name: &str, seen: &mut Vec<String>, defs: &mut Vec<String>) -> String {
    match v {
        serde_json::Value::Null => "null".into(),
        serde_json::Value::Bool(_) => "boolean".into(),
        serde_json::Value::Number(_) => "number".into(),
        serde_json::Value::String(_) => "string".into(),
        serde_json::Value::Array(arr) => {
            if arr.is_empty() { return "any[]".into(); }
            let elem_name = format!("{}Item", name);
            let types: Vec<String> = arr.iter().map(|e| json_type_to_ts(e, &elem_name, seen, defs)).collect();
            let elem_type = if types.is_empty() || types.iter().all(|t| t == &types[0]) {
                types.first().cloned().unwrap_or_else(|| "any".into())
            } else {
                "any".into()
            };
            format!("{}[]", elem_type)
        }
        serde_json::Value::Object(obj) => {
            if seen.contains(&name.to_string()) { return name.to_string(); }
            seen.push(name.to_string());

            let mut fields = Vec::new();
            for (k, val) in obj {
                let field_type = json_type_to_ts(val, &type_name_from_key(k), seen, defs);
                let opt = if val.is_null() { "?" } else { "" };
                fields.push(format!("  {}{}: {};", k, opt, field_type));
            }
            defs.push(format!("export interface {} {{\n{}\n}}", name, fields.join("\n")));
            name.to_string()
        }
    }
}

fn generate_typescript_types(v: &serde_json::Value, name: &str) -> String {
    let mut seen = Vec::new();
    let mut defs = Vec::new();
    let root = json_type_to_ts(v, name, &mut seen, &mut defs);
    if defs.is_empty() { root }
    else { defs.reverse(); defs.join("\n\n") }
}

fn json_type_to_go(v: &serde_json::Value, name: &str, seen: &mut Vec<String>, defs: &mut Vec<String>) -> String {
    match v {
        serde_json::Value::Null => "interface{}".into(),
        serde_json::Value::Bool(_) => "bool".into(),
        serde_json::Value::Number(n) => {
            if n.is_f64() { "float64".into() }
            else if n.as_i64().map_or(false, |i| i >= 0) { "uint64".into() }
            else { "int64".into() }
        }
        serde_json::Value::String(_) => "string".into(),
        serde_json::Value::Array(arr) => {
            if arr.is_empty() { "[]interface{}".into() }
            else {
                let elem_name = format!("{}Item", name);
                let types: Vec<String> = arr.iter().map(|e| json_type_to_go(e, &elem_name, seen, defs)).collect();
                let elem_type = if types.is_empty() || types.iter().all(|t| t == &types[0]) {
                    types.first().cloned().unwrap_or_else(|| "interface{}".into())
                } else {
                    "interface{}".into()
                };
                format!("[]{}", elem_type)
            }
        }
        serde_json::Value::Object(obj) => {
            if seen.contains(&name.to_string()) { return name.to_string(); }
            seen.push(name.to_string());

            let mut fields = Vec::new();
            for (k, val) in obj {
                let go_name = type_name_from_key(k);
                let go_type = json_type_to_go(val, &go_name, seen, defs);
                fields.push(format!("    {} {} `json:\"{}\"`", go_name, go_type, k));
            }
            defs.push(format!("type {} struct {{\n{}\n}}", name, fields.join("\n")));
            name.to_string()
        }
    }
}

fn generate_go_types(v: &serde_json::Value, name: &str) -> String {
    let mut seen = Vec::new();
    let mut defs = Vec::new();
    let root = json_type_to_go(v, name, &mut seen, &mut defs);
    if defs.is_empty() { format!("package main\n\ntype {} {}", name, root) }
    else { defs.reverse(); format!("package main\n\n{}", defs.join("\n\n")) }
}

fn json_type_to_rust(v: &serde_json::Value, name: &str, seen: &mut Vec<String>, defs: &mut Vec<String>) -> String {
    match v {
        serde_json::Value::Null => "Option<serde_json::Value>".into(),
        serde_json::Value::Bool(_) => "bool".into(),
        serde_json::Value::Number(n) => {
            if n.is_f64() { "f64".into() }
            else if n.as_i64().map_or(false, |i| i >= 0 && i <= u64::MAX as i64) { "u64".into() }
            else { "i64".into() }
        }
        serde_json::Value::String(_) => "String".into(),
        serde_json::Value::Array(arr) => {
            if arr.is_empty() { "Vec<serde_json::Value>".into() }
            else {
                let elem_name = format!("{}Item", name);
                let types: Vec<String> = arr.iter().map(|e| json_type_to_rust(e, &elem_name, seen, defs)).collect();
                let elem_type = if types.is_empty() || types.iter().all(|t| t == &types[0]) {
                    types.first().cloned().unwrap_or_else(|| "serde_json::Value".into())
                } else {
                    "serde_json::Value".into()
                };
                format!("Vec<{}>", elem_type)
            }
        }
        serde_json::Value::Object(obj) => {
            if seen.contains(&name.to_string()) { return name.to_string(); }
            seen.push(name.to_string());

            let mut fields = Vec::new();
            for (k, val) in obj {
                let rs_type = json_type_to_rust(val, &type_name_from_key(k), seen, defs);
                fields.push(format!("    pub {}: {},", k, rs_type));
            }
            defs.push(format!("#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct {} {{\n{}\n}}", name, fields.join("\n")));
            name.to_string()
        }
    }
}

fn generate_rust_types(v: &serde_json::Value, name: &str) -> String {
    let mut seen = Vec::new();
    let mut defs = Vec::new();
    let root = json_type_to_rust(v, name, &mut seen, &mut defs);
    if defs.is_empty() { root }
    else { defs.reverse(); defs.join("\n\n") }
}

// ============================================================
// 028 - Structural Schema Differ
// ============================================================
pub fn struct_diff(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "data.struct-diff".into(),
            title: "Struct Diff".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste two JSON payloads separated by '---' on its own line.".into(),
        });
    }

    // Split on a line containing only "---"
    let mut parts: Vec<String> = Vec::new();
    let mut buf = String::new();
    for line in trimmed.lines() {
        if line.trim() == "---" {
            parts.push(buf.trim().to_string());
            buf = String::new();
        } else {
            buf.push_str(line);
            buf.push('\n');
        }
    }
    let last = buf.trim().to_string();
    if !last.is_empty() {
        parts.push(last);
    }

    if parts.len() < 2 {
        return Ok(CommandExecutionResult {
            command_id: "data.struct-diff".into(),
            title: "Struct Diff".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Separate two JSON payloads with '---' on its own line.".into(),
        });
    }

    let left = parse_json_input(&parts[0])?;
    let right = parse_json_input(&parts[1])?;

    let mut output = String::new();
    let mut additions = 0;
    let mut removals = 0;
    let mut changes = 0;

    diff_values(&left, &right, "", &mut output, &mut additions, &mut removals, &mut changes);

    if output.is_empty() {
        output = "No differences found — the two payloads are identical.".to_string();
    }

    Ok(CommandExecutionResult {
        command_id: "data.struct-diff".into(),
        title: "Diff Result".into(),
        output,
        status: CommandExecutionStatus::Success,
        summary: format!("{} addition(s), {} removal(s), {} change(s)", additions, removals, changes),
    })
}

fn diff_values(left: &serde_json::Value, right: &serde_json::Value, path: &str, output: &mut String, adds: &mut i32, rems: &mut i32, chgs: &mut i32) {
    use serde_json::Value;

    match (left, right) {
        (Value::Object(l), Value::Object(r)) => {
            let mut all_keys: Vec<&String> = l.keys().chain(r.keys()).collect();
            all_keys.sort();
            all_keys.dedup();
            for key in all_keys {
                let child_path = if path.is_empty() { key.clone() } else { format!("{}.{}", path, key) };
                match (l.get(key), r.get(key)) {
                    (Some(lv), Some(rv)) => diff_values(lv, rv, &child_path, output, adds, rems, chgs),
                    (Some(_), None) => {
                        *rems += 1;
                        output.push_str(&format!("  - {} (removed)\n", child_path));
                    }
                    (None, Some(_)) => {
                        *adds += 1;
                        output.push_str(&format!("  + {} (added)\n", child_path));
                    }
                    (None, None) => {}
                }
            }
        }
        (Value::Array(l), Value::Array(r)) => {
            let max = l.len().max(r.len());
            for i in 0..max {
                let child_path = format!("{}[{}]", path, i);
                match (l.get(i), r.get(i)) {
                    (Some(lv), Some(rv)) => diff_values(lv, rv, &child_path, output, adds, rems, chgs),
                    (Some(_), None) => {
                        *rems += 1;
                        output.push_str(&format!("  - {} (removed)\n", child_path));
                    }
                    (None, Some(_)) => {
                        *adds += 1;
                        output.push_str(&format!("  + {} (added)\n", child_path));
                    }
                    (None, None) => {}
                }
            }
        }
        (l, r) if l == r => {}
        _ => {
            *chgs += 1;
            output.push_str(&format!("  ~ {}: \"{}\" -> \"{}\"\n", path, left, right));
        }
    }
}

// ============================================================
// 029 - URL Query Parameter Deconstructor
// ============================================================
pub fn url_parse(input: &str) -> Result<CommandExecutionResult, AppError> {
    let url_str = input.trim();
    if url_str.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "data.url-parse".into(),
            title: "URL Parse".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste a URL to deconstruct its query parameters.".into(),
        });
    }

    let mut output = String::new();

    // Parse URL parts
    let url = url_str;

    // Extract scheme
    let (scheme, after_scheme) = if let Some(pos) = url.find("://") {
        let s = &url[..pos];
        (s.to_string(), &url[pos + 3..])
    } else {
        ("http".into(), url)
    };
    output.push_str(&format!("Scheme:  {}\n", scheme));

    // Extract host and path
    let (host_port, path_and_query) = if let Some(pos) = after_scheme.find('/') {
        (&after_scheme[..pos], &after_scheme[pos..])
    } else {
        (after_scheme, "")
    };

    // Extract host and port
    let (host, port_str) = if let Some(pos) = host_port.rfind(':') {
        // Check if it's an IPv6 address with port
        if host_port.starts_with('[') {
            if let Some(bracket_end) = host_port.find(']') {
                if let Some(port_pos) = host_port[bracket_end..].strip_prefix("]:") {
                    (&host_port[1..bracket_end], Some(port_pos.to_string()))
                } else {
                    (&host_port[1..bracket_end], None)
                }
            } else {
                (host_port, None)
            }
        } else {
            (&host_port[..pos], Some(host_port[pos + 1..].to_string()))
        }
    } else {
        (host_port, None)
    };

    output.push_str(&format!("Host:    {}\n", host));
    if let Some(port) = port_str {
        output.push_str(&format!("Port:    {}\n", port));
    }

    // Extract path and query
    output.push('\n');
    if let Some(qpos) = path_and_query.find('?') {
        let path = &path_and_query[..qpos];
        let query = &path_and_query[qpos + 1..];
        output.push_str(&format!("Path:    {}\n", if path.is_empty() { "/" } else { path }));

        if let Some(fpos) = query.find('#') {
            let fragment = &query[fpos + 1..];
            let qs = &query[..fpos];
            output.push_str(&format!("Fragment: {}\n", fragment));
            output.push('\n');
            output.push_str("Query Parameters:\n");
            parse_query_string(qs, &mut output);
        } else {
            output.push('\n');
            output.push_str("Query Parameters:\n");
            parse_query_string(query, &mut output);
        }
    } else if path_and_query.contains('#') {
        let fpos = path_and_query.find('#').unwrap();
        let path = &path_and_query[..fpos];
        let fragment = &path_and_query[fpos + 1..];
        output.push_str(&format!("Path:    {}\n", if path.is_empty() { "/" } else { path }));
        output.push_str(&format!("Fragment: {}\n", fragment));
    } else {
        output.push_str(&format!("Path:    {}\n", if path_and_query.is_empty() { "/" } else { path_and_query }));
    }

    Ok(CommandExecutionResult {
        command_id: "data.url-parse".into(),
        title: "URL Parsed".into(),
        output,
        status: CommandExecutionStatus::Success,
        summary: "URL query parameters deconstructed".into(),
    })
}

fn parse_query_string(qs: &str, output: &mut String) {
    for pair in qs.split('&') {
        if pair.is_empty() { continue; }
        if let Some(pos) = pair.find('=') {
            let key = url_decode(&pair[..pos]);
            let value = url_decode(&pair[pos + 1..]);
            output.push_str(&format!("  {:<30} = {}\n", key, value));
        } else {
            let key = url_decode(pair);
            output.push_str(&format!("  {:<30} (flag)\n", key));
        }
    }
}

fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else if ch == '+' {
            result.push(' ');
        } else {
            result.push(ch);
        }
    }
    result
}

// ============================================================
// 030 - XML Path (XPath) & JSONPath Evaluator
// ============================================================
pub fn path_eval(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "data.path-eval".into(),
            title: "Path Eval".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "First line is the path expression, rest is the JSON document.\nExamples:\n$.store.book[0].title\n$.store.book[*].author\n$..price".into(),
        });
    }

    let (expr, body) = if let Some(pos) = trimmed.find('\n') {
        (trimmed[..pos].trim(), trimmed[pos..].trim())
    } else {
        return Ok(CommandExecutionResult {
            command_id: "data.path-eval".into(),
            title: "Path Eval".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "First line should be the path expression, followed by the JSON document.".into(),
        });
    };

    if body.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "data.path-eval".into(),
            title: "Path Eval".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste a JSON document after the path expression.".into(),
        });
    }

    let value = parse_json_input(body)?;
    let results = evaluate_jsonpath(expr, &value);

    let output = if results.is_empty() {
        "No matches found.".to_string()
    } else {
        let mut out = String::new();
        for (i, result) in results.iter().enumerate() {
            out.push_str(&format!("[{}] {}\n", i, serde_json::to_string_pretty(result).unwrap_or_default()));
            if i < results.len() - 1 {
                out.push('\n');
            }
        }
        out
    };

    Ok(CommandExecutionResult {
        command_id: "data.path-eval".into(),
        title: format!("Path: {}", expr),
        output,
        status: CommandExecutionStatus::Success,
        summary: format!("{} match(es) found", results.len()),
    })
}

fn evaluate_jsonpath(expr: &str, root: &serde_json::Value) -> Vec<serde_json::Value> {
    let expr = expr.trim();
    let expr = expr.strip_prefix('$').unwrap_or(expr);
    let expr = expr.strip_prefix('.').unwrap_or(expr);

    if expr.is_empty() {
        return vec![root.clone()];
    }

    let mut results = Vec::new();
    evaluate_segment(expr, root, &mut results);
    results
}

fn evaluate_segment(expr: &str, current: &serde_json::Value, results: &mut Vec<serde_json::Value>) {
    if expr.is_empty() {
        results.push(current.clone());
        return;
    }

    // Handle recursive descent ..
    if expr.starts_with("..") {
        let rest = &expr[2..];
        // Add current if matches the rest
        recursive_search(rest, current, results);
        return;
    }

    // Handle wildcard .*
    if expr.starts_with(".*") || expr.starts_with("[*]") {
        let rest = if expr.starts_with(".*") { &expr[2..] } else { &expr[3..] };
        match current {
            serde_json::Value::Object(obj) => {
                for val in obj.values() {
                    evaluate_segment(rest, val, results);
                }
            }
            serde_json::Value::Array(arr) => {
                for val in arr {
                    evaluate_segment(rest, val, results);
                }
            }
            _ => {}
        }
        return;
    }

    // Handle bracket notation [0], ['key']
    if expr.starts_with('[') {
        if let Some(end) = find_matching_bracket(expr) {
            let bracket_content = &expr[1..end];
            let rest = expr[end + 1..].trim();
            let rest = rest.strip_prefix('.').unwrap_or(rest);

            // Numeric index
            if let Ok(idx) = bracket_content.parse::<usize>() {
                if let serde_json::Value::Array(arr) = current {
                    if idx < arr.len() {
                        evaluate_segment(rest, &arr[idx], results);
                    }
                }
                return;
            }

            // String key
            let key = bracket_content.trim_matches('\'').trim_matches('"');
            if let serde_json::Value::Object(obj) = current {
                if let Some(val) = obj.get(key) {
                    evaluate_segment(rest, val, results);
                }
            }
            return;
        }
    }

    // Handle dot notation .key or key
    let (key, rest) = if let Some(pos) = expr.find(|c: char| c == '.' || c == '[') {
        let k = expr[..pos].trim();
        let r = if expr.as_bytes().get(pos) == Some(&b'.') { &expr[pos + 1..] } else { &expr[pos..] };
        (k, r)
    } else {
        (expr.trim(), "")
    };

    if key.is_empty() {
        evaluate_segment(rest, current, results);
        return;
    }

    if let serde_json::Value::Object(obj) = current {
        if let Some(val) = obj.get(key) {
            evaluate_segment(rest, val, results);
        }
    }
}

fn recursive_search(expr: &str, current: &serde_json::Value, results: &mut Vec<serde_json::Value>) {
    // Check current against the rest expression
    if expr.is_empty() {
        results.push(current.clone());
    } else {
        evaluate_segment(expr, current, results);
    }

    // Recurse into children
    match current {
        serde_json::Value::Object(obj) => {
            for val in obj.values() {
                recursive_search(expr, val, results);
            }
        }
        serde_json::Value::Array(arr) => {
            for val in arr {
                recursive_search(expr, val, results);
            }
        }
        _ => {}
    }
}

fn find_matching_bracket(expr: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, ch) in expr.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}
