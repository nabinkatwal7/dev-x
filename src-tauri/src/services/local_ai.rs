use crate::error::AppError;
use crate::models::{CommandExecutionResult, CommandExecutionStatus};
use regex::Regex;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

const OLLAMA_ADDRESS: &str = "127.0.0.1:11434";
const DEFAULT_MODEL: &str = "llama3.1:8b";

pub fn ollama_bridge(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    let body = http_request(
        "GET",
        OLLAMA_ADDRESS,
        "/api/tags",
        &[("Accept", "application/json")],
        "",
    )?;
    let parsed: Value = serde_json::from_str(&body)
        .map_err(|error| AppError::Internal(format!("invalid Ollama response: {}", error)))?;
    let models = parsed["models"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|model| model["name"].as_str().map(|name| name.to_string()))
        .collect::<Vec<_>>();

    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("models") || trimmed.eq_ignore_ascii_case("tags") {
        let mut output = format!(
            "Endpoint: http://{}\nStatus: reachable\nModels: {}\n",
            OLLAMA_ADDRESS,
            models.len()
        );
        if models.is_empty() {
            output.push_str("No local models reported.");
        } else {
            output.push_str(
                &models
                    .iter()
                    .enumerate()
                    .map(|(index, name)| format!("{}. {}", index + 1, name))
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
        }
        return Ok(success(
            "ai.ollama-bridge",
            "Ollama Bridge",
            output,
            "Connected to the local Ollama runtime.",
        ));
    }

    if let Some(model) = trimmed.strip_prefix("show:") {
        let model = model.trim();
        let details = http_request(
            "POST",
            OLLAMA_ADDRESS,
            "/api/show",
            &[("Content-Type", "application/json")],
            &json!({ "name": model }).to_string(),
        )?;
        let parsed: Value = serde_json::from_str(&details)
            .map_err(|error| AppError::Internal(format!("invalid Ollama model details: {}", error)))?;
        let output = format!(
            "Model: {}\nLicense: {}\nParameters: {}\nTemplate present: {}\nDetails:\n{}",
            model,
            parsed["license"].as_str().unwrap_or("(unknown)"),
            parsed["parameters"].as_str().unwrap_or("(none)"),
            parsed["template"].as_str().map(|value| !value.trim().is_empty()).unwrap_or(false),
            parsed["details"].to_string()
        );
        return Ok(success(
            "ai.ollama-bridge",
            "Ollama Model Details",
            output,
            format!("Loaded metadata for '{}'.", model),
        ));
    }

    let request = parse_model_request(trimmed);
    let prompt = request.prompt.unwrap_or_else(|| trimmed.to_string());
    let response = ollama_generate(&request.model, "You are a concise local developer assistant.", &prompt)?;
    Ok(success(
        "ai.ollama-bridge",
        "Ollama Prompt",
        response,
        format!("Generated a local response with '{}'.", request.model),
    ))
}

pub fn explain_error_log(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "ai.error-explain",
            "Error Explanation",
            "Paste a compiler error, runtime exception, stack trace, or failing command log.".into(),
            "Provide an error log to explain.",
        ));
    }

    let fallback = explain_error_log_fallback(trimmed);
    respond_with_model_or_fallback(
        "ai.error-explain",
        "Error Explanation",
        "Explain the developer error log. Identify the likely root cause, the evidence line, and the next debugging steps. Keep it concrete and technical.",
        trimmed,
        &fallback,
        "Generated a local diagnostic explanation.",
    )
}

pub fn optimize_code(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "ai.code-optimize",
            "Code Optimization Analyst",
            "Paste a function, module, or code block to review performance, maintainability, and failure risks.".into(),
            "Provide code to analyze.",
        ));
    }

    let fallback = optimize_code_fallback(trimmed);
    respond_with_model_or_fallback(
        "ai.code-optimize",
        "Code Optimization Analyst",
        "Review the code like a senior engineer. Prioritize concrete performance issues, reliability risks, and maintainability problems. Do not rewrite the whole snippet unless necessary.",
        trimmed,
        &fallback,
        "Generated local optimization guidance.",
    )
}

pub fn semantic_snippet_search(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "ai.snippet-search",
            "Semantic Snippet Search",
            "Format:\nquery:<what you need>\n---\n<snippet one>\n---\n<snippet two>\n\nOptional per-snippet title header:\ntitle: auth middleware".into(),
            "Provide a query and candidate snippets.",
        ));
    }

    let parts = split_blocks(trimmed);
    if parts.len() < 2 {
        return Err(AppError::Internal(
            "provide query:<text> followed by one or more snippets separated by ---".into(),
        ));
    }

    let query = parts[0]
        .trim()
        .strip_prefix("query:")
        .unwrap_or(parts[0].trim())
        .trim()
        .to_lowercase();
    if query.is_empty() {
        return Err(AppError::Internal("query text cannot be empty".into()));
    }

    let query_terms = tokenize(&query);
    let mut scored = parts[1..]
        .iter()
        .enumerate()
        .map(|(index, snippet)| {
            let (title, body) = parse_titled_snippet(snippet, index + 1);
            let tokens = tokenize(&body);
            let unique_overlap = query_terms
                .iter()
                .filter(|term| tokens.contains(*term))
                .count();
            let title_boost = tokenize(&title)
                .iter()
                .filter(|term| query_terms.contains(term))
                .count();
            let line_hits = body
                .lines()
                .filter(|line| {
                    let lower = line.to_lowercase();
                    query_terms.iter().any(|term| lower.contains(term))
                })
                .take(4)
                .map(|line| line.trim().to_string())
                .collect::<Vec<_>>();
            (unique_overlap * 10 + title_boost * 8, title, body, line_hits)
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));

    let output = scored
        .iter()
        .take(5)
        .enumerate()
        .map(|(index, (score, title, body, line_hits))| {
            let preview = if line_hits.is_empty() {
                body.lines().take(5).collect::<Vec<_>>().join("\n")
            } else {
                line_hits.join("\n")
            };
            format!(
                "{}. {} | score {}\n{}",
                index + 1,
                title,
                score,
                preview
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    Ok(success(
        "ai.snippet-search",
        "Semantic Snippet Search",
        output,
        format!("Ranked {} snippet candidate(s).", scored.len().min(5)),
    ))
}

pub fn nl_to_sql(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "ai.sql-translator",
            "SQL Translator",
            "Describe a query in plain English.\nOptional schema block:\nschema:\nusers(id, name, email, created_at)\norders(id, user_id, total, status)\n---\nrequest:\ncount paid orders by status".into(),
            "Provide a natural language query request.",
        ));
    }

    let fallback = sql_fallback(trimmed);
    respond_with_model_or_fallback(
        "ai.sql-translator",
        "SQL Translator",
        "Convert the request into SQL. Prefer ANSI SQL, use explicit table names, and add brief notes only when assumptions are necessary.",
        trimmed,
        &fallback,
        "Generated a local SQL draft.",
    )
}

pub fn markdown_docs(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "ai.markdown-docs",
            "Markdown Docs Drafter",
            "Paste source material or use:\nname: package-or-function\nsummary: one sentence\nusage: command or API example\nnotes: optional detail".into(),
            "Provide source material to draft documentation.",
        ));
    }

    let fallback = markdown_docs_fallback(trimmed);
    respond_with_model_or_fallback(
        "ai.markdown-docs",
        "Markdown Docs Drafter",
        "Draft developer-facing Markdown documentation with clear sections, examples, and limitations. Keep it concise and practical.",
        trimmed,
        &fallback,
        "Generated documentation scaffold.",
    )
}

pub fn test_scaffold(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "ai.test-scaffold",
            "Test Scaffold",
            "Paste a function, class, or module snippet from Rust, TypeScript, or JavaScript to scaffold tests.".into(),
            "Provide code to scaffold tests for.",
        ));
    }

    let fallback = test_scaffold_fallback(trimmed);
    respond_with_model_or_fallback(
        "ai.test-scaffold",
        "Test Scaffold",
        "Generate a practical unit test scaffold for the pasted code. Include a happy path, an edge case, and one failure-mode test if relevant.",
        trimmed,
        &fallback,
        "Generated unit test boilerplate.",
    )
}

pub fn vuln_check(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "ai.vuln-check",
            "Compliance & Vulnerability Checker",
            "Paste code, config, or logs to scan for secrets, risky transports, unsafe defaults, and suspicious shell patterns.".into(),
            "Provide content to scan.",
        ));
    }

    lazy_static::lazy_static! {
        static ref PATTERNS: Vec<(&'static str, &'static str, Regex)> = vec![
            ("critical", "Private key material", Regex::new(r"-----BEGIN [A-Z ]*PRIVATE KEY-----").unwrap()),
            ("critical", "AWS access key", Regex::new(r"AKIA[0-9A-Z]{16}").unwrap()),
            ("high", "Bearer token", Regex::new(r"(?i)bearer\\s+[A-Za-z0-9_\\-\\.=]+").unwrap()),
            ("high", "GitHub token", Regex::new(r"ghp_[A-Za-z0-9]{36,}").unwrap()),
            ("high", "Hardcoded password", Regex::new(r#"(?i)(password|passwd|pwd)\\s*[:=]\\s*["'][^"']+["']"#).unwrap()),
            ("medium", "Plain HTTP URL", Regex::new(r#"http://[^\\s"']+"#).unwrap()),
            ("medium", "Command injection risk", Regex::new(r#"(?i)(exec|spawn|system|popen)\\s*\\(.+\\+"#).unwrap()),
            ("medium", "Wildcard CORS", Regex::new(r#"(?i)access-control-allow-origin\\s*[:=]\\s*["']\\*["']"#).unwrap()),
            ("medium", "Debug bind all interfaces", Regex::new(r#"0\\.0\\.0\\.0:\\d+"#).unwrap()),
        ];
    }

    let mut findings: Vec<String> = Vec::new();
    for (severity, label, regex) in PATTERNS.iter() {
        let count = regex.find_iter(trimmed).count();
        if count > 0 {
            findings.push(format!("[{}] {} x{}", severity, label, count));
        }
    }

    if trimmed.contains("verify = false") || trimmed.contains("rejectUnauthorized: false") {
        findings.push("[high] TLS verification disabled".into());
    }
    if trimmed.contains("eval(") {
        findings.push("[high] Dynamic eval detected".into());
    }

    if findings.is_empty() {
        findings.push("No high-confidence security patterns detected in the supplied content.".into());
    }

    Ok(success(
        "ai.vuln-check",
        "Compliance & Vulnerability Checker",
        findings
            .iter()
            .enumerate()
            .map(|(index, finding)| format!("{}. {}", index + 1, finding))
            .collect::<Vec<_>>()
            .join("\n"),
        "Completed local risk scan.",
    ))
}

pub fn rename_variables(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "ai.rename-vars",
            "Variable Renaming Engine",
            "Paste code to get clearer, context-aware variable naming suggestions.".into(),
            "Provide code to review.",
        ));
    }

    let fallback = rename_variables_fallback(trimmed);
    respond_with_model_or_fallback(
        "ai.rename-vars",
        "Variable Renaming Engine",
        "Review the code and suggest better variable names. Keep each suggestion in the format old_name -> better_name: reason.",
        trimmed,
        &fallback,
        "Generated naming suggestions.",
    )
}

pub fn offline_dictionary(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "ai.offline-dict",
            "Offline Multilingual Dictionary",
            "Search a developer term or short phrase.\nOptional format:\ntranslate: permission denied\nto: es,fr,de,ne".into(),
            "Provide a word or phrase to translate.",
        ));
    }

    let fallback = dictionary_fallback(trimmed);
    respond_with_model_or_fallback(
        "ai.offline-dict",
        "Offline Multilingual Dictionary",
        "Translate the developer phrase into Spanish, French, German, and Nepali. Return one line per language with a short, practical translation.",
        trimmed,
        &fallback,
        "Returned offline translation entries.",
    )
}

fn respond_with_model_or_fallback(
    command_id: &str,
    title: &str,
    instruction: &str,
    input: &str,
    fallback: &str,
    summary: &str,
) -> Result<CommandExecutionResult, AppError> {
    let request = parse_model_request(input);
    let prompt = if let Some(prompt) = request.prompt {
        prompt
    } else {
        format!("{}\n\nInput:\n{}", instruction, input)
    };

    let output = match ollama_generate(&request.model, "You are a local offline developer copilot.", &prompt) {
        Ok(response) if !response.trim().is_empty() => response,
        _ => fallback.to_string(),
    };

    Ok(success(command_id, title, output, summary))
}

fn parse_model_request(input: &str) -> ModelRequest {
    let mut model = DEFAULT_MODEL.to_string();
    let mut prompt = None;
    let mut body_lines = Vec::new();

    for line in input.lines() {
        if let Some(value) = line.strip_prefix("model:") {
            model = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("prompt:") {
            prompt = Some(value.trim().to_string());
        } else {
            body_lines.push(line);
        }
    }

    if prompt.is_none() {
        let body = body_lines.join("\n").trim().to_string();
        if !body.is_empty() {
            prompt = Some(body);
        }
    }

    ModelRequest { model, prompt }
}

struct ModelRequest {
    model: String,
    prompt: Option<String>,
}

fn ollama_generate(model: &str, system: &str, prompt: &str) -> Result<String, AppError> {
    let body = json!({
        "model": model,
        "system": system,
        "prompt": prompt,
        "stream": false
    })
    .to_string();
    let response = http_request(
        "POST",
        OLLAMA_ADDRESS,
        "/api/generate",
        &[("Content-Type", "application/json"), ("Accept", "application/json")],
        &body,
    )?;
    let parsed: Value = serde_json::from_str(&response)
        .map_err(|error| AppError::Internal(format!("invalid Ollama generation response: {}", error)))?;
    parsed["response"]
        .as_str()
        .map(|value| value.trim().to_string())
        .ok_or_else(|| AppError::Internal("Ollama response missing generated text".into()))
}

fn http_request(
    method: &str,
    address: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &str,
) -> Result<String, AppError> {
    let mut stream = TcpStream::connect(address)
        .map_err(|error| AppError::Internal(format!("connect failed: {}", error)))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|error| AppError::Internal(format!("timeout setup failed: {}", error)))?;
    let mut request = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nContent-Length: {}\r\n",
        method,
        path,
        address,
        body.as_bytes().len()
    );
    for (key, value) in headers {
        request.push_str(&format!("{}: {}\r\n", key, value));
    }
    request.push_str("\r\n");
    request.push_str(body);

    stream
        .write_all(request.as_bytes())
        .map_err(|error| AppError::Internal(format!("request failed: {}", error)))?;
    let mut buffer = String::new();
    stream
        .read_to_string(&mut buffer)
        .map_err(|error| AppError::Internal(format!("response read failed: {}", error)))?;

    let mut parts = buffer.splitn(2, "\r\n\r\n");
    let head = parts.next().unwrap_or_default();
    let body = parts.next().unwrap_or_default();
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(0);
    if !(200..300).contains(&status) {
        return Err(AppError::Internal(format!(
            "HTTP {} returned from {} {}",
            status, method, path
        )));
    }
    Ok(body.to_string())
}

fn explain_error_log_fallback(input: &str) -> String {
    let lower = input.to_lowercase();
    let mut findings: Vec<String> = Vec::new();

    if let Some(line) = input.lines().find(|line| line.to_lowercase().contains("error")) {
        findings.push(format!("Primary error line: {}", line.trim()));
    }
    if lower.contains("cannot find module") || lower.contains("module not found") {
        findings.push("Likely root cause: missing dependency, wrong import path, or path casing mismatch.".into());
    }
    if lower.contains("permission denied") || lower.contains("eacces") {
        findings.push("Likely root cause: insufficient filesystem, process, or port permissions.".into());
    }
    if lower.contains("address already in use") || lower.contains("eaddrinuse") {
        findings.push("Likely root cause: target port already bound by another process.".into());
    }
    if lower.contains("unexpected token") || lower.contains("syntaxerror") {
        findings.push("Likely root cause: parse failure caused by malformed syntax near the reported line.".into());
    }
    if lower.contains("timeout") {
        findings.push("Likely root cause: network or blocking I/O exceeded the configured timeout.".into());
    }
    if lower.contains("undefined") || lower.contains("null") || lower.contains("nonetype") {
        findings.push("Likely root cause: missing value dereferenced before validation.".into());
    }
    if findings.is_empty() {
        findings.push("No strong signature matched. Start from the first failing line and trace the earliest referenced file and stack frame.".into());
    }

    findings
        .iter()
        .enumerate()
        .map(|(index, finding)| format!("{}. {}", index + 1, finding))
        .collect::<Vec<_>>()
        .join("\n")
}

fn optimize_code_fallback(input: &str) -> String {
    let mut findings: Vec<String> = Vec::new();
    let line_count = input.lines().count();

    if input.matches("for ").count() >= 2 || input.matches("while ").count() >= 2 {
        findings.push("Nested iteration detected. Check whether repeated scans can be replaced with indexing, caching, or precomputed maps.".into());
    }
    if input.matches(".clone()").count() > 2 {
        findings.push("Repeated cloning may create unnecessary allocations. Prefer borrowing or moving when ownership allows.".into());
    }
    if input.contains("unwrap()") {
        findings.push("Unwrap usage introduces crash paths on invalid input or I/O failure. Replace with explicit error handling on non-test paths.".into());
    }
    if input.contains("println!") || input.contains("console.log") {
        findings.push("Inline debug logging is present. Gate noisy logs behind log levels or remove them from hot code paths.".into());
    }
    if line_count > 80 {
        findings.push("The snippet is large enough to suggest splitting responsibilities into smaller functions or units.".into());
    }
    if findings.is_empty() {
        findings.push("No high-confidence structural issues detected. The snippet looks straightforward under local heuristic checks.".into());
    }

    findings
        .iter()
        .enumerate()
        .map(|(index, finding)| format!("{}. {}", index + 1, finding))
        .collect::<Vec<_>>()
        .join("\n")
}

fn sql_fallback(input: &str) -> String {
    let lower = input.to_lowercase();
    let request = if let Some(rest) = lower.split("request:").nth(1) {
        rest.trim().to_string()
    } else {
        lower
    };
    let table = infer_primary_table(&request);

    if request.contains("count") && request.contains(" by ") {
        let group = request.split(" by ").nth(1).unwrap_or("status").trim();
        return format!(
            "SELECT {group}, COUNT(*) AS total\nFROM {table}\nGROUP BY {group}\nORDER BY total DESC;"
        );
    }
    if request.contains("top") && request.contains(" by ") {
        let metric = request.split(" by ").nth(1).unwrap_or("metric").trim();
        return format!(
            "SELECT *\nFROM {table}\nORDER BY {metric} DESC\nLIMIT 10;"
        );
    }
    if request.contains("last 7 days") || request.contains("past 7 days") {
        return format!(
            "SELECT *\nFROM {table}\nWHERE created_at >= CURRENT_DATE - INTERVAL '7 days'\nORDER BY created_at DESC;"
        );
    }
    if request.contains("today") {
        return format!(
            "SELECT *\nFROM {table}\nWHERE DATE(created_at) = CURRENT_DATE\nORDER BY created_at DESC;"
        );
    }
    if request.contains("where") {
        return format!(
            "SELECT *\nFROM {table}\nWHERE /* translate filters from request */\nORDER BY created_at DESC;"
        );
    }

    format!("SELECT *\nFROM {table}\nORDER BY created_at DESC;")
}

fn markdown_docs_fallback(input: &str) -> String {
    let fields = parse_key_value_block(input);
    let name = fields.get("name").cloned().unwrap_or_else(|| "Project".into());
    let summary = fields
        .get("summary")
        .cloned()
        .unwrap_or_else(|| "Developer-facing overview.".into());
    let usage = fields
        .get("usage")
        .cloned()
        .unwrap_or_else(|| "command or API example".into());
    let notes = fields.get("notes").cloned().unwrap_or_else(|| "Add implementation constraints, edge cases, and troubleshooting notes.".into());

    format!(
        "# {name}\n\n{summary}\n\n## Usage\n\n```text\n{usage}\n```\n\n## Inputs\n\n- Document the required inputs.\n\n## Outputs\n\n- Document the returned values or side effects.\n\n## Notes\n\n- {notes}\n"
    )
}

fn test_scaffold_fallback(input: &str) -> String {
    if input.contains("fn ") {
        return "```rust\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn handles_happy_path() {\n        // arrange\n        // act\n        // assert\n    }\n\n    #[test]\n    fn handles_edge_case() {\n        // arrange\n        // act\n        // assert\n    }\n\n    #[test]\n    fn rejects_invalid_input() {\n        // arrange\n        // act\n        // assert\n    }\n}\n```".into();
    }

    "```ts\nimport { describe, it, expect } from \"vitest\";\n\ndescribe(\"subject\", () => {\n  it(\"handles the happy path\", () => {\n    // arrange\n    // act\n    // expect(...)\n  });\n\n  it(\"handles an edge case\", () => {\n    // arrange\n    // act\n    // expect(...)\n  });\n\n  it(\"rejects invalid input\", () => {\n    // arrange\n    // act\n    // expect(...)\n  });\n});\n```".into()
}

fn rename_variables_fallback(input: &str) -> String {
    let mut seen = BTreeSet::new();
    let generic = [
        ("tmp", "temporaryValue"),
        ("temp", "temporaryValue"),
        ("data", "payload"),
        ("res", "result"),
        ("obj", "record"),
        ("val", "value"),
        ("arr", "items"),
        ("fn", "handler"),
        ("ctx", "context"),
        ("cfg", "config"),
        ("x", "horizontalPosition"),
        ("y", "verticalPosition"),
    ];
    let mut suggestions = Vec::new();
    for (ident, replacement) in generic {
        if input.contains(ident) && seen.insert(ident) {
            suggestions.push(format!("{} -> {}: generic name hides intent", ident, replacement));
        }
    }
    if suggestions.is_empty() {
        suggestions.push("No obvious low-signal variable names were detected.".into());
    }
    suggestions.join("\n")
}

fn dictionary_fallback(input: &str) -> String {
    let mut phrase = input.trim().to_lowercase();
    let fields = parse_key_value_block(input);
    if let Some(value) = fields.get("translate") {
        phrase = value.to_lowercase();
    }

    let mut table = BTreeMap::new();
    table.insert(
        "error",
        vec![("es", "error"), ("fr", "erreur"), ("de", "Fehler"), ("ne", "त्रुटि")],
    );
    table.insert(
        "success",
        vec![("es", "éxito"), ("fr", "succès"), ("de", "Erfolg"), ("ne", "सफलता")],
    );
    table.insert(
        "timeout",
        vec![
            ("es", "tiempo de espera agotado"),
            ("fr", "délai dépassé"),
            ("de", "Zeitüberschreitung"),
            ("ne", "समय समाप्त"),
        ],
    );
    table.insert(
        "permission denied",
        vec![
            ("es", "permiso denegado"),
            ("fr", "permission refusée"),
            ("de", "Zugriff verweigert"),
            ("ne", "अनुमति अस्वीकृत"),
        ],
    );

    if let Some(entries) = table.get(phrase.as_str()) {
        return entries
            .iter()
            .map(|(lang, text)| format!("{}: {}", lang, text))
            .collect::<Vec<_>>()
            .join("\n");
    }

    format!(
        "en: {}\nes: translation unavailable locally\nfr: translation unavailable locally\nde: translation unavailable locally\nne: translation unavailable locally",
        phrase
    )
}

fn parse_key_value_block(input: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for line in input.lines() {
        if let Some((key, value)) = line.split_once(':') {
            map.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }
    map
}

fn infer_primary_table(request: &str) -> String {
    if request.contains("users") || request.contains("user") {
        "users".into()
    } else if request.contains("orders") || request.contains("order") {
        "orders".into()
    } else if request.contains("products") || request.contains("product") {
        "products".into()
    } else {
        "your_table".into()
    }
}

fn split_blocks(input: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = String::new();
    for line in input.lines() {
        if line.trim() == "---" {
            if !current.trim().is_empty() {
                blocks.push(current.trim().to_string());
            }
            current.clear();
        } else {
            current.push_str(line);
            current.push('\n');
        }
    }
    if !current.trim().is_empty() {
        blocks.push(current.trim().to_string());
    }
    blocks
}

fn parse_titled_snippet(snippet: &str, index: usize) -> (String, String) {
    let mut lines = snippet.lines();
    let first = lines.next().unwrap_or_default();
    if let Some(title) = first.strip_prefix("title:") {
        return (title.trim().to_string(), lines.collect::<Vec<_>>().join("\n"));
    }
    (format!("Snippet {}", index), snippet.trim().to_string())
}

fn tokenize(input: &str) -> Vec<String> {
    input
        .split(|char: char| !char.is_ascii_alphanumeric())
        .filter(|term| !term.is_empty())
        .map(|term| term.to_lowercase())
        .collect()
}

fn info(command_id: &str, title: &str, output: String, summary: impl Into<String>) -> CommandExecutionResult {
    CommandExecutionResult {
        command_id: command_id.into(),
        title: title.into(),
        output,
        status: CommandExecutionStatus::Info,
        summary: summary.into(),
    }
}

fn success(command_id: &str, title: &str, output: String, summary: impl Into<String>) -> CommandExecutionResult {
    CommandExecutionResult {
        command_id: command_id.into(),
        title: title.into(),
        output,
        status: CommandExecutionStatus::Success,
        summary: summary.into(),
    }
}
