use crate::{
    error::AppError,
    models::{CommandExecutionResult, CommandExecutionStatus, ExecuteCommandPayload},
    services::{crypto, data, design, filesystem, local_ai, mock, network, shell, text},
};

pub struct CommandExecutor;

impl CommandExecutor {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        payload: ExecuteCommandPayload,
    ) -> Result<CommandExecutionResult, AppError> {
        match payload.command_id.as_str() {
            // === Data Features (021-030) ===
            "data.format-json" => format_json(&payload.input, false),
            "data.minify-json" => format_json(&payload.input, true),
            "data.schema-convert" => data::schema_convert(&payload.input),
            "data.json-flatten" => data::json_flatten(&payload.input),
            "data.json-unflatten" => data::json_unflatten(&payload.input),
            "data.sql-beautify" => data::sql_beautify(&payload.input),
            "data.escape" => data::escape_text(&payload.input),
            "data.unescape" => data::unescape_text(&payload.input),
            "data.csv-table" => data::csv_table(&payload.input),
            "data.gen-types" => data::gen_types(&payload.input),
            "data.struct-diff" => data::struct_diff(&payload.input),
            "data.url-parse" => data::url_parse(&payload.input),
            "data.path-eval" => data::path_eval(&payload.input),
            // === Network Features (011-020) ===
            "net.port-mapper" => network::list_listeners(&payload.input),
            "net.kill-process" => network::kill_process(&payload.input),
            "net.port-monitor" => network::check_port(&payload.input),
            "net.hosts-edit" => network::edit_hosts(&payload.input),
            "net.tunnel-mgr" => network::tunnel_manager(&payload.input),
            "net.curl-builder" => network::curl_builder(&payload.input),
            "net.ping" => network::run_ping(&payload.input),
            "net.trace" => network::run_traceroute(&payload.input),
            "net.ip-discover" => network::discover_ip(&payload.input),
            "net.domain-check" => network::check_domain(&payload.input),
            "net.subnet-sweep" => network::sweep_subnet(&payload.input),
            // === Crypto Features (031-040) ===
            "crypto.hash" => crypto::hash_compute(&payload.input),
            "crypto.jwt" => crypto::jwt_inspect(&payload.input),
            "crypto.cipher" => crypto::symmetric_cipher(&payload.input),
            "crypto.rsa-keygen" => crypto::rsa_keygen(&payload.input),
            "crypto.base64" => crypto::base64_convert(&payload.input),
            "crypto.hash-bench" => crypto::hash_benchmark(&payload.input),
            "crypto.html-decode" => crypto::html_decode(&payload.input),
            "crypto.gen-token" => crypto::token_generate(&payload.input),
            "crypto.hmac" => crypto::hmac_compute(&payload.input),
            "crypto.vault" => crypto::crypto_vault(&payload.input),
            // === Clipboard & Text Features (041-050) ===
            "clip.stack" => text::clipboard_stack(&payload.input),
            "clip.classify" => text::classify_text(&payload.input),
            "clip.snippets" => text::snippet_board(&payload.input),
            "clip.merge-split" => text::merge_split(&payload.input),
            "clip.regex" => {
                if payload.input.trim().starts_with("strip:") {
                    text::regex_strip(&payload.input)
                } else {
                    text::regex_transform(&payload.input)
                }
            }
            "clip.case" => text::case_normalize(&payload.input),
            "clip.diff" => text::clipboard_diff(&payload.input),
            "clip.whitespace" => text::whitespace_sanitize(&payload.input),
            "clip.redact" => text::redact_data(&payload.input),
            "clip.queue" => text::clip_queue(&payload.input),
            // === Shell & Snippet Features (051-060) ===
            "shell.cheatsheet" => shell::shell_cheat_sheet(&payload.input),
            "shell.git-wizard" => shell::git_reconstruct(&payload.input),
            "shell.code-vault" => shell::code_vault(&payload.input),
            "shell.history" => shell::shell_history_search(&payload.input),
            "shell.alias" => shell::alias_blueprint(&payload.input),
            "shell.path-translate" => shell::path_translate(&payload.input),
            "shell.cron" => shell::cron_explain(&payload.input),
            "shell.compose" => shell::docker_compose_generate(&payload.input),
            "shell.ansi-strip" => shell::strip_ansi(&payload.input),
            "shell.exit-code" => shell::exit_code_reference(&payload.input),
            // === Filesystem Features (061-070) ===
            "fs.env-audit" => filesystem::env_audit(&payload.input),
            "fs.duplicate-scan" => filesystem::duplicate_scan(&payload.input),
            "fs.symlink-matrix" => filesystem::symlink_matrix(&payload.input),
            "fs.log-tail" => filesystem::log_tail(&payload.input),
            "fs.file-sentinel" => filesystem::file_sentinel(&payload.input),
            "fs.hex-inspect" => filesystem::hex_inspector(&payload.input),
            "fs.batch-rename" => filesystem::batch_rename(&payload.input),
            "fs.disk-explorer" => filesystem::disk_explorer(&payload.input),
            "fs.encoding-convert" => filesystem::encoding_convert(&payload.input),
            "fs.archive-check" => filesystem::archive_integrity(&payload.input),
            // === Design Features (071-080) ===
            "design.eyedropper" => design::eyedropper(&payload.input),
            "design.color-swap" => design::color_swapper(&payload.input),
            "design.svg-optimize" => design::svg_optimize(&payload.input),
            "design.type-scale" => design::typography_scale(&payload.input),
            "design.aspect-ratio" => design::aspect_ratio(&payload.input),
            "design.layout-css" => design::layout_constructor(&payload.input),
            "design.mock-data" => design::mock_data(&payload.input),
            "design.contrast" => design::contrast_check(&payload.input),
            "design.shadow-gradient" => design::shadow_gradient(&payload.input),
            "design.font-inventory" => design::font_inventory(&payload.input),
            // === Local AI Features (081-090) ===
            "ai.ollama-bridge" => local_ai::ollama_bridge(&payload.input),
            "ai.error-explain" => local_ai::explain_error_log(&payload.input),
            "ai.code-optimize" => local_ai::optimize_code(&payload.input),
            "ai.snippet-search" => local_ai::semantic_snippet_search(&payload.input),
            "ai.sql-translator" => local_ai::nl_to_sql(&payload.input),
            "ai.markdown-docs" => local_ai::markdown_docs(&payload.input),
            "ai.test-scaffold" => local_ai::test_scaffold(&payload.input),
            "ai.vuln-check" => local_ai::vuln_check(&payload.input),
            "ai.rename-vars" => local_ai::rename_variables(&payload.input),
            "ai.offline-dict" => local_ai::offline_dictionary(&payload.input),
            // === Mock & Testing Features (091-100) ===
            "mock.http-server" => mock::http_mock_server(&payload.input),
            "mock.load-test" => mock::http_load_test(&payload.input),
            "mock.websocket" => mock::websocket_lab(&payload.input),
            "mock.graphql" => mock::graphql_tools(&payload.input),
            "mock.grpc" => mock::grpc_tools(&payload.input),
            "mock.env-matrix" => mock::env_mock_matrix(&payload.input),
            "mock.rest-collection" => mock::rest_collection(&payload.input),
            "mock.webhook" => mock::webhook_receiver(&payload.input),
            "mock.status-codes" => mock::status_code_reference(&payload.input),
            "mock.cookie-parser" => mock::cookie_parser(&payload.input),
            _ => Err(AppError::Internal(format!(
                "unknown command id: {}",
                payload.command_id
            ))),
        }
    }
}

fn format_json(input: &str, minify: bool) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: if minify {
                "data.minify-json".into()
            } else {
                "data.format-json".into()
            },
            title: if minify {
                "Minify JSON".into()
            } else {
                "Format JSON".into()
            },
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste a JSON document to transform it.".into(),
        });
    }

    let value: serde_json::Value = serde_json::from_str(trimmed)
        .map_err(|error| AppError::Internal(format!("invalid JSON: {error}")))?;

    let output = if minify {
        serde_json::to_string(&value)?
    } else {
        serde_json::to_string_pretty(&value)?
    };

    Ok(CommandExecutionResult {
        command_id: if minify {
            "data.minify-json".into()
        } else {
            "data.format-json".into()
        },
        title: if minify {
            "Minified JSON".into()
        } else {
            "Formatted JSON".into()
        },
        output,
        status: CommandExecutionStatus::Success,
        summary: if minify {
            "JSON payload minified successfully.".into()
        } else {
            "JSON payload formatted successfully.".into()
        },
    })
}
