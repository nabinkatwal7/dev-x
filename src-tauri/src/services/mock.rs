use crate::error::AppError;
use crate::models::{CommandExecutionResult, CommandExecutionStatus};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};
use std::process::Command;

#[derive(Clone)]
struct RestRequest {
    name: String,
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: String,
}

#[derive(Clone)]
struct HttpMockRoute {
    method: String,
    path: String,
    status: u16,
    content_type: String,
    body: String,
    headers: Vec<(String, String)>,
}

#[derive(Clone)]
struct MockRequestLog {
    method: String,
    path: String,
    status: u16,
    body: String,
}

struct HttpMockState {
    running: bool,
    port: u16,
    stop: Option<Arc<AtomicBool>>,
    routes: Vec<HttpMockRoute>,
    logs: VecDeque<MockRequestLog>,
}

struct WebhookState {
    running: bool,
    port: u16,
    stop: Option<Arc<AtomicBool>>,
    logs: Arc<Mutex<VecDeque<String>>>,
    response_status: u16,
    response_body: String,
    response_content_type: String,
}

lazy_static::lazy_static! {
    static ref REST_COLLECTION: Mutex<Vec<RestRequest>> = Mutex::new(Vec::new());
    static ref HTTP_MOCK: Mutex<HttpMockState> = Mutex::new(HttpMockState {
        running: false,
        port: 0,
        stop: None,
        routes: vec![HttpMockRoute {
            method: "GET".into(),
            path: "/".into(),
            status: 200,
            content_type: "application/json".into(),
            body: json!({"ok": true, "source": "devforge-mock"}).to_string(),
            headers: Vec::new(),
        }],
        logs: VecDeque::new(),
    });
    static ref WEBHOOK: Mutex<WebhookState> = Mutex::new(WebhookState {
        running: false,
        port: 0,
        stop: None,
        logs: Arc::new(Mutex::new(VecDeque::new())),
        response_status: 202,
        response_body: String::new(),
        response_content_type: "application/json".into(),
    });
}

pub fn http_mock_server(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        let state = HTTP_MOCK.lock().unwrap();
        return Ok(info(
            "mock.http-server",
            "HTTP Mock Server",
            "Commands:\nstart:7878\nstatus\nstop\nroute:METHOD /path status=200 type=application/json\nbody:{\"ok\":true}\nlist-routes\nlogs\nclear-logs\nclear-routes".into(),
            if state.running {
                format!("Running on port {}.", state.port)
            } else {
                "Not running.".into()
            },
        ));
    }

    if trimmed.eq_ignore_ascii_case("status") {
        let state = HTTP_MOCK.lock().unwrap();
        let routes = state
            .routes
            .iter()
            .map(|route| format!("{} {} -> {}", route.method, route.path, route.status))
            .collect::<Vec<_>>()
            .join("\n");
        return Ok(success(
            "mock.http-server",
            "HTTP Mock Server Status",
            format!(
                "Status: {}\nPort: {}\nRoutes: {}\nCaptured requests: {}\n\n{}",
                if state.running { "running" } else { "stopped" },
                state.port,
                state.routes.len(),
                state.logs.len(),
                routes
            ),
            "Reported HTTP mock server status.",
        ));
    }

    if trimmed.eq_ignore_ascii_case("logs") {
        let state = HTTP_MOCK.lock().unwrap();
        let output = if state.logs.is_empty() {
            "No requests captured.".into()
        } else {
            state
                .logs
                .iter()
                .enumerate()
                .map(|(index, log)| {
                    format!(
                        "{}. {} {} -> {}\n{}",
                        index + 1,
                        log.method,
                        log.path,
                        log.status,
                        log.body
                    )
                })
                .collect::<Vec<_>>()
                .join("\n\n")
        };
        return Ok(success(
            "mock.http-server",
            "HTTP Mock Logs",
            output,
            "Returned captured mock requests.",
        ));
    }

    if trimmed.eq_ignore_ascii_case("list-routes") {
        let state = HTTP_MOCK.lock().unwrap();
        let output = state
            .routes
            .iter()
            .enumerate()
            .map(|(index, route)| {
                format!(
                    "{}. {} {} -> {} [{}]\n{}",
                    index + 1,
                    route.method,
                    route.path,
                    route.status,
                    route.content_type,
                    route.body
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        return Ok(success(
            "mock.http-server",
            "HTTP Mock Routes",
            output,
            format!("{} route(s) configured.", state.routes.len()),
        ));
    }

    if trimmed.eq_ignore_ascii_case("clear-logs") {
        HTTP_MOCK.lock().unwrap().logs.clear();
        return Ok(success(
            "mock.http-server",
            "HTTP Mock Server",
            "Cleared request logs.".into(),
            "Request logs cleared.",
        ));
    }

    if trimmed.eq_ignore_ascii_case("clear-routes") {
        let mut state = HTTP_MOCK.lock().unwrap();
        state.routes.retain(|route| route.path == "/" && route.method == "GET");
        return Ok(success(
            "mock.http-server",
            "HTTP Mock Server",
            "Cleared custom routes and kept the default root route.".into(),
            "Custom routes cleared.",
        ));
    }

    if trimmed.eq_ignore_ascii_case("stop") {
        let mut state = HTTP_MOCK.lock().unwrap();
        if let Some(stop) = &state.stop {
            stop.store(true, Ordering::SeqCst);
        }
        state.running = false;
        state.stop = None;
        return Ok(success(
            "mock.http-server",
            "HTTP Mock Server",
            "Stop signal sent.".into(),
            "Mock server stopped.",
        ));
    }

    if let Some(port_text) = trimmed.strip_prefix("start:") {
        let port = port_text.trim().parse::<u16>().unwrap_or(7878);
        start_http_mock(port)?;
        return Ok(success(
            "mock.http-server",
            "HTTP Mock Server",
            format!("Started mock server on http://127.0.0.1:{}/", port),
            "Mock server started.",
        ));
    }

    if let Some(route) = trimmed.strip_prefix("route:") {
        let route = parse_http_mock_route(route)?;
        let mut state = HTTP_MOCK.lock().unwrap();
        state
            .routes
            .retain(|existing| !(existing.method == route.method && existing.path == route.path));
        state.routes.push(route.clone());
        return Ok(success(
            "mock.http-server",
            "HTTP Mock Route Saved",
            format!("{} {} -> {} [{}]\n{}", route.method, route.path, route.status, route.content_type, route.body),
            "Route configuration saved.",
        ));
    }

    Ok(info(
        "mock.http-server",
        "HTTP Mock Server",
        "Commands: start, status, stop, route, list-routes, logs, clear-logs, clear-routes".into(),
        "Unsupported mock server command.",
    ))
}

pub fn http_load_test(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "mock.load-test",
            "HTTP Load Tester",
            "Format:\nurl:http://127.0.0.1:7878/\nmethod:GET\nrequests:50\nconcurrency:5\nheader:Authorization=Bearer token\nbody:{\"ok\":true}".into(),
            "Provide an HTTP endpoint and load profile.",
        ));
    }

    let profile = parse_http_profile(trimmed);
    let total_requests = profile.requests;
    let concurrency = profile.concurrency.max(1).min(total_requests.max(1));
    let start = Instant::now();
    let success_count = Arc::new(Mutex::new(0usize));
    let failure_count = Arc::new(Mutex::new(0usize));
    let latencies = Arc::new(Mutex::new(Vec::<u64>::new()));
    let statuses = Arc::new(Mutex::new(BTreeMap::<u16, usize>::new()));

    let requests_per_worker = total_requests / concurrency;
    let remainder = total_requests % concurrency;
    let mut handles = Vec::new();

    for worker in 0..concurrency {
        let worker_requests = requests_per_worker + usize::from(worker < remainder);
        let url = profile.url.clone();
        let method = profile.method.clone();
        let headers = profile.headers.clone();
        let body = profile.body.clone();
        let success_count = success_count.clone();
        let failure_count = failure_count.clone();
        let latencies = latencies.clone();
        let statuses = statuses.clone();

        handles.push(thread::spawn(move || {
            for _ in 0..worker_requests {
                let before = Instant::now();
                match simple_http_request(&method, &url, &headers, &body) {
                    Ok((status, _)) => {
                        *success_count.lock().unwrap() += 1;
                        latencies.lock().unwrap().push(before.elapsed().as_millis() as u64);
                        *statuses.lock().unwrap().entry(status).or_insert(0) += 1;
                    }
                    Err(_) => {
                        *failure_count.lock().unwrap() += 1;
                    }
                }
            }
        }));
    }

    for handle in handles {
        let _ = handle.join();
    }

    let elapsed = start.elapsed().as_millis();
    let mut samples = latencies.lock().unwrap().clone();
    samples.sort_unstable();
    let min = samples.first().copied().unwrap_or(0);
    let max = samples.last().copied().unwrap_or(0);
    let avg = if samples.is_empty() {
        0.0
    } else {
        samples.iter().sum::<u64>() as f64 / samples.len() as f64
    };
    let p50 = percentile(&samples, 0.50);
    let p95 = percentile(&samples, 0.95);
    let statuses_text = statuses
        .lock()
        .unwrap()
        .iter()
        .map(|(status, count)| format!("{} -> {}", status, count))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(success(
        "mock.load-test",
        "HTTP Load Tester",
        format!(
            "URL: {}\nMethod: {}\nRequests: {}\nConcurrency: {}\nSuccess: {}\nFailed: {}\nElapsed: {} ms\nMin latency: {} ms\nAvg latency: {:.2} ms\nP50 latency: {} ms\nP95 latency: {} ms\nMax latency: {} ms\n\nStatus codes:\n{}",
            profile.url,
            profile.method,
            total_requests,
            concurrency,
            *success_count.lock().unwrap(),
            *failure_count.lock().unwrap(),
            elapsed,
            min,
            avg,
            p50,
            p95,
            max,
            if statuses_text.is_empty() { "(none)".into() } else { statuses_text }
        ),
        "Completed local load test run.",
    ))
}

pub fn websocket_lab(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "mock.websocket",
            "WebSocket Lab",
            "Commands:\nprobe:ws://127.0.0.1:3001/socket\nhandshake:ws://127.0.0.1:3001/socket\nsend:ws://127.0.0.1:3001/socket|hello".into(),
            "Provide a WebSocket action.",
        ));
    }

    let action = if let Some(value) = trimmed.strip_prefix("probe:") {
        ("probe", value.trim())
    } else if let Some(value) = trimmed.strip_prefix("handshake:") {
        ("handshake", value.trim())
    } else if let Some(value) = trimmed.strip_prefix("send:") {
        ("send", value.trim())
    } else {
        ("probe", trimmed)
    };

    if action.0 == "send" {
        let (url, payload) = action
            .1
            .split_once('|')
            .ok_or_else(|| AppError::Internal("send format: send:ws://host:port/path|message".into()))?;
        let endpoint = parse_ws_endpoint(url.trim())?;
        let mut stream = websocket_upgrade(&endpoint)?;
        write_ws_text_frame(&mut stream, payload.trim())?;
        let reply = read_ws_text_frame(&mut stream).unwrap_or_else(|_| "(no text frame returned before timeout)".into());
        return Ok(success(
            "mock.websocket",
            "WebSocket Send",
            format!(
                "URL: {}\nSent: {}\nReceived: {}",
                url.trim(),
                payload.trim(),
                reply
            ),
            "Executed one-shot WebSocket send/receive.",
        ));
    }

    let endpoint = parse_ws_endpoint(action.1)?;
    let reachable = TcpStream::connect((&endpoint.host[..], endpoint.port)).is_ok();
    if action.0 == "probe" {
        return Ok(success(
            "mock.websocket",
            "WebSocket Probe",
            format!(
                "URL: {}\nHost: {}\nPort: {}\nPath: {}\nTCP reachable: {}",
                action.1,
                endpoint.host,
                endpoint.port,
                endpoint.path,
                if reachable { "yes" } else { "no" }
            ),
            "Probed WebSocket TCP reachability.",
        ));
    }

    let (request, response) = websocket_handshake_text(&endpoint)?;

    Ok(success(
        "mock.websocket",
        "WebSocket Handshake",
        format!("Request:\n{}\nResponse:\n{}", request.trim_end(), response.trim()),
        "Executed raw WebSocket upgrade handshake.",
    ))
}

pub fn graphql_tools(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "mock.graphql",
            "GraphQL Builder",
            "Formats:\nendpoint:http://localhost:4000/graphql\nquery:{ users { id name } }\nvariables:{\"limit\":10}\n\nOr generate from:\nentity:user\nfields:id,name,email".into(),
            "Provide GraphQL context to generate or execute queries.",
        ));
    }

    let fields = parse_prefixed_fields(trimmed);
    if let Some(endpoint) = fields.get("endpoint") {
        let query = fields.get("query").cloned().unwrap_or_else(introspection_query);
        let variables = fields.get("variables").cloned().unwrap_or_else(|| "{}".into());
        let payload = json!({
            "query": query,
            "variables": serde_json::from_str::<Value>(&variables).unwrap_or_else(|_| json!({}))
        })
        .to_string();
        let body = execute_http_json("POST", endpoint, &[("Content-Type", "application/json")], &payload)?;
        return Ok(success(
            "mock.graphql",
            "GraphQL Response",
            body,
            "Executed GraphQL request against the local endpoint.",
        ));
    }

    let entity = fields.get("entity").cloned().unwrap_or_else(|| "users".into());
    let fields_text = fields
        .get("fields")
        .map(|value| value.split(',').map(|item| item.trim()).filter(|item| !item.is_empty()).collect::<Vec<_>>().join("\n    "))
        .unwrap_or_else(|| "id\n    name\n    email".into());
    let query = format!(
        "query {}List {{\n  {} {{\n    {}\n  }}\n}}",
        to_title_case(&entity),
        entity,
        fields_text
    );
    let mutation = format!(
        "mutation Create{} {{\n  create{}(input: {{ /* fields */ }}) {{\n    id\n  }}\n}}",
        to_title_case(&entity),
        to_title_case(&entity)
    );
    Ok(success(
        "mock.graphql",
        "GraphQL Builder",
        format!("{}\n\n{}", query, mutation),
        "Generated GraphQL query and mutation templates.",
    ))
}

pub fn grpc_tools(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "mock.grpc",
            "gRPC Verification Deck",
            "Paste a .proto definition. Optional:\nendpoint:127.0.0.1:50051\nThen include the proto below it.".into(),
            "Provide proto text to inspect.",
        ));
    }

    let fields = parse_prefixed_fields(trimmed);
    let endpoint = fields.get("endpoint").cloned();
    let rpc = fields.get("rpc").cloned();
    let proto_path = fields.get("proto").cloned();
    let data = fields.get("data").cloned().unwrap_or_else(|| "{}".into());
    let proto = trimmed
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !(trimmed.starts_with("endpoint:")
                || trimmed.starts_with("rpc:")
                || trimmed.starts_with("proto:")
                || trimmed.starts_with("data:"))
        })
        .collect::<Vec<_>>()
        .join("\n");

    if let (Some(endpoint), Some(rpc)) = (endpoint.clone(), rpc.clone()) {
        match execute_grpcurl(&endpoint, proto_path.as_deref(), &rpc, &data) {
            Ok(output) => {
                return Ok(success(
                    "mock.grpc",
                    "gRPC Execution",
                    output,
                    "Executed gRPC request using grpcurl.",
                ));
            }
            Err(error) => {
                if proto.trim().is_empty() {
                    return Ok(info(
                        "mock.grpc",
                        "gRPC Verification Deck",
                        format!(
                            "Real invocation failed.\n{}\n\nTo execute directly, provide grpcurl on PATH and use:\nendpoint:{}\nrpc:{}\nproto:path/to/service.proto\ndata:{{}}",
                            error, endpoint, rpc
                        ),
                        "gRPC execution unavailable; returning diagnostic guidance.",
                    ));
                }
            }
        }
    }

    let services = proto
        .lines()
        .filter_map(|line| line.trim().strip_prefix("service ").map(|rest| rest.trim_end_matches('{').trim().to_string()))
        .collect::<Vec<_>>();
    let rpcs = proto
        .lines()
        .filter_map(|line| line.trim().strip_prefix("rpc ").map(|rest| rest.trim_end_matches(';').trim().to_string()))
        .collect::<Vec<_>>();
    let messages = proto
        .lines()
        .filter_map(|line| line.trim().strip_prefix("message ").map(|rest| rest.trim_end_matches('{').trim().to_string()))
        .collect::<Vec<_>>();

    let connectivity = endpoint
        .as_ref()
        .map(|value| {
            let reachable = TcpStream::connect(value.as_str()).is_ok();
            format!("Endpoint: {}\nTCP reachable: {}", value, if reachable { "yes" } else { "no" })
        })
        .unwrap_or_default();

    let method_templates = rpcs
        .iter()
        .map(|rpc| format!("- grpcurl -plaintext <host:port> <package.Service/{}>", rpc.split('(').next().unwrap_or(rpc)))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(success(
        "mock.grpc",
        "gRPC Verification Deck",
        format!(
            "{}\n\nServices:\n{}\n\nRPCs:\n{}\n\nMessages:\n{}\n\nCall templates:\n{}",
            connectivity,
            format_lines(&services),
            format_lines(&rpcs),
            format_lines(&messages),
            if method_templates.is_empty() { "(none)".into() } else { method_templates }
        ),
        "Parsed proto service structure.",
    ))
}

fn execute_grpcurl(endpoint: &str, proto_path: Option<&str>, rpc: &str, data: &str) -> Result<String, AppError> {
    let mut command = Command::new("grpcurl");
    command.arg("-plaintext");
    if let Some(proto_path) = proto_path {
        command.arg("-proto").arg(proto_path);
    }
    command.arg("-d").arg(data).arg(endpoint).arg(rpc);
    let output = command
        .output()
        .map_err(|error| AppError::Internal(format!("failed to launch grpcurl: {}", error)))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::Internal(format!("grpcurl failed: {}", stderr.trim())));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn env_mock_matrix(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "mock.env-matrix",
            "Environment Mock Matrix",
            "Format:\nprofile:slow-3g\nlatency=250\njitter=40\ndrop=0.05\nstatus500=0.10".into(),
            "Provide simulated environment knobs.",
        ));
    }

    let mut config = HashMap::new();
    for line in trimmed.lines() {
        if let Some((key, value)) = line.split_once('=') {
            config.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    let profile = trimmed
        .lines()
        .find_map(|line| line.strip_prefix("profile:").map(|value| value.trim().to_string()))
        .unwrap_or_else(|| "custom".into());
    let latency = config.get("latency").cloned().unwrap_or_else(|| "0".into());
    let jitter = config.get("jitter").cloned().unwrap_or_else(|| "0".into());
    let drop = config.get("drop").cloned().unwrap_or_else(|| "0".into());
    let status500 = config.get("status500").cloned().unwrap_or_else(|| "0".into());
    let output = json!({
        "profile": profile,
        "networkLatencyMs": latency,
        "jitterMs": jitter,
        "packetDropRate": drop,
        "http500Rate": status500
    })
    .to_string();

    Ok(success(
        "mock.env-matrix",
        "Environment Mock Matrix",
        output,
        "Built environment simulation profile.",
    ))
}

pub fn rest_collection(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "mock.rest-collection",
            "REST Collection Bench",
            "Commands:\nadd:<name>\nmethod:GET\nurl:http://localhost:3000/health\nheader:Authorization=Bearer token\nbody:{...}\n\nlist\nget:<name>\nsend:<name>\ndel:<name>\nclear".into(),
            "Manage in-memory request definitions.",
        ));
    }

    if trimmed.eq_ignore_ascii_case("list") {
        let collection = REST_COLLECTION.lock().unwrap();
        let output = if collection.is_empty() {
            "No requests stored.".into()
        } else {
            collection
                .iter()
                .enumerate()
                .map(|(index, request)| format!("{}. {} [{} {}]", index + 1, request.name, request.method, request.url))
                .collect::<Vec<_>>()
                .join("\n")
        };
        return Ok(success(
            "mock.rest-collection",
            "REST Collection Bench",
            output,
            format!("{} request(s) stored.", collection.len()),
        ));
    }

    if trimmed.eq_ignore_ascii_case("clear") {
        REST_COLLECTION.lock().unwrap().clear();
        return Ok(success(
            "mock.rest-collection",
            "REST Collection Bench",
            "Cleared request collection.".into(),
            "Request collection cleared.",
        ));
    }

    if let Some(name) = trimmed.strip_prefix("get:") {
        let collection = REST_COLLECTION.lock().unwrap();
        if let Some(request) = collection.iter().find(|request| request.name.eq_ignore_ascii_case(name.trim())) {
            let headers = request
                .headers
                .iter()
                .map(|(key, value)| format!("{}: {}", key, value))
                .collect::<Vec<_>>()
                .join("\n");
            return Ok(success(
                "mock.rest-collection",
                "REST Request Definition",
                format!("{} {}\n{}\n\n{}", request.method, request.url, headers, request.body),
                "Returned request definition.",
            ));
        }
        return Ok(info(
            "mock.rest-collection",
            "REST Collection Bench",
            format!("No request named '{}'.", name.trim()),
            "Request not found.",
        ));
    }

    if let Some(name) = trimmed.strip_prefix("send:") {
        let request = {
            let collection = REST_COLLECTION.lock().unwrap();
            collection
                .iter()
                .find(|request| request.name.eq_ignore_ascii_case(name.trim()))
                .cloned()
        };
        if let Some(request) = request {
            let response = execute_http_text(&request.method, &request.url, &request.headers, &request.body)?;
            return Ok(success(
                "mock.rest-collection",
                "REST Request Execution",
                response,
                format!("Executed '{}'.", request.name),
            ));
        }
        return Ok(info(
            "mock.rest-collection",
            "REST Collection Bench",
            format!("No request named '{}'.", name.trim()),
            "Request not found.",
        ));
    }

    if let Some(name) = trimmed.strip_prefix("del:") {
        let name = name.trim();
        let mut collection = REST_COLLECTION.lock().unwrap();
        let before = collection.len();
        collection.retain(|request| !request.name.eq_ignore_ascii_case(name));
        return Ok(success(
            "mock.rest-collection",
            "REST Collection Bench",
            if collection.len() == before {
                format!("No request named '{}'.", name)
            } else {
                format!("Deleted '{}'.", name)
            },
            "Updated request collection.",
        ));
    }

    if let Some(name) = trimmed.strip_prefix("add:") {
        let request = parse_rest_request(name, trimmed)?;
        let mut collection = REST_COLLECTION.lock().unwrap();
        collection.retain(|item| !item.name.eq_ignore_ascii_case(&request.name));
        collection.push(request.clone());
        return Ok(success(
            "mock.rest-collection",
            "REST Request Saved",
            format!("{} {} [{}]", request.name, request.method, request.url),
            "Request definition saved.",
        ));
    }

    Ok(info(
        "mock.rest-collection",
        "REST Collection Bench",
        "Commands: add, list, get, send, del, clear".into(),
        "Unsupported request collection command.",
    ))
}

pub fn webhook_receiver(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        let state = WEBHOOK.lock().unwrap();
        return Ok(info(
            "mock.webhook",
            "Webhook Receiver",
            "Commands:\nstart:8787\nstatus\nlogs\nstop\nclear\nresponse:status=202 type=application/json\nbody:{\"received\":true}".into(),
            if state.running {
                format!("Running on port {}.", state.port)
            } else {
                "Not running.".into()
            },
        ));
    }

    if trimmed.eq_ignore_ascii_case("status") {
        let state = WEBHOOK.lock().unwrap();
        return Ok(success(
            "mock.webhook",
            "Webhook Receiver Status",
            format!(
                "Status: {}\nPort: {}\nCaptured payloads: {}\nResponse status: {}\nResponse content-type: {}",
                if state.running { "running" } else { "stopped" },
                state.port,
                state.logs.lock().unwrap().len(),
                state.response_status,
                state.response_content_type
            ),
            "Reported webhook receiver status.",
        ));
    }

    if trimmed.eq_ignore_ascii_case("logs") {
        let state = WEBHOOK.lock().unwrap();
        let logs = state.logs.lock().unwrap();
        let output = if logs.is_empty() {
            "No webhook payloads captured.".into()
        } else {
            logs.iter().cloned().collect::<Vec<_>>().join("\n\n---\n\n")
        };
        return Ok(success("mock.webhook", "Webhook Receiver Logs", output, "Returned captured webhook payloads."));
    }

    if trimmed.eq_ignore_ascii_case("clear") {
        WEBHOOK.lock().unwrap().logs.lock().unwrap().clear();
        return Ok(success(
            "mock.webhook",
            "Webhook Receiver",
            "Cleared captured webhook logs.".into(),
            "Webhook logs cleared.",
        ));
    }

    if trimmed.eq_ignore_ascii_case("stop") {
        let mut state = WEBHOOK.lock().unwrap();
        if let Some(stop) = &state.stop {
            stop.store(true, Ordering::SeqCst);
        }
        state.running = false;
        state.stop = None;
        return Ok(success(
            "mock.webhook",
            "Webhook Receiver",
            "Stop signal sent.".into(),
            "Webhook receiver stopped.",
        ));
    }

    if let Some(port_text) = trimmed.strip_prefix("start:") {
        let port = port_text.trim().parse::<u16>().unwrap_or(8787);
        start_webhook_receiver(port)?;
        return Ok(success(
            "mock.webhook",
            "Webhook Receiver",
            format!("Started webhook receiver on http://127.0.0.1:{}/", port),
            "Webhook receiver started.",
        ));
    }

    if let Some(config) = trimmed.strip_prefix("response:") {
        let response = parse_response_config(config, trimmed)?;
        let mut state = WEBHOOK.lock().unwrap();
        state.response_status = response.0;
        state.response_content_type = response.1;
        state.response_body = response.2;
        return Ok(success(
            "mock.webhook",
            "Webhook Receiver Response",
            format!(
                "Status: {}\nContent-Type: {}\nBody:\n{}",
                state.response_status, state.response_content_type, state.response_body
            ),
            "Updated webhook response settings.",
        ));
    }

    Ok(info(
        "mock.webhook",
        "Webhook Receiver",
        "Commands: start, status, logs, stop, clear, response".into(),
        "Unsupported webhook command.",
    ))
}

pub fn status_code_reference(input: &str) -> Result<CommandExecutionResult, AppError> {
    let query = input.trim().to_lowercase();
    let entries = vec![
        (200, "OK", "Request succeeded."),
        (201, "Created", "Resource created successfully."),
        (202, "Accepted", "Request accepted for asynchronous processing."),
        (204, "No Content", "Request succeeded with no response body."),
        (301, "Moved Permanently", "Resource location changed permanently."),
        (302, "Found", "Temporary redirect."),
        (304, "Not Modified", "Client can reuse the cached representation."),
        (400, "Bad Request", "Malformed request or validation failure."),
        (401, "Unauthorized", "Authentication required or invalid."),
        (403, "Forbidden", "Authenticated but not allowed."),
        (404, "Not Found", "Target resource does not exist."),
        (409, "Conflict", "State conflict or duplicate resource."),
        (422, "Unprocessable Entity", "Validation or semantic error."),
        (429, "Too Many Requests", "Rate limit exceeded."),
        (500, "Internal Server Error", "Unhandled server failure."),
        (502, "Bad Gateway", "Upstream service returned an invalid response."),
        (503, "Service Unavailable", "Server unavailable or overloaded."),
        (504, "Gateway Timeout", "Upstream service did not respond in time."),
    ];

    let filtered = if query.is_empty() {
        entries.clone()
    } else if let Ok(code) = query.parse::<u16>() {
        entries
            .into_iter()
            .filter(|(value, _, _)| *value == code)
            .collect::<Vec<_>>()
    } else {
        entries
            .into_iter()
            .filter(|(_, label, detail)| {
                label.to_lowercase().contains(&query) || detail.to_lowercase().contains(&query)
            })
            .collect::<Vec<_>>()
    };

    let output = if filtered.is_empty() {
        format!("No built-in reference matched '{}'.", query)
    } else {
        filtered
            .iter()
            .map(|(value, label, detail)| format!("{} {}\n{}", value, label, detail))
            .collect::<Vec<_>>()
            .join("\n\n")
    };

    Ok(success(
        "mock.status-codes",
        "Status Code Reference",
        output,
        "Returned HTTP status code reference.",
    ))
}

pub fn cookie_parser(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "mock.cookie-parser",
            "Cookie Parser",
            "Paste a Cookie header or Set-Cookie string.".into(),
            "Provide cookie data to parse.",
        ));
    }

    let parts = trimmed.split(';').map(|part| part.trim()).collect::<Vec<_>>();
    let mut output = Vec::new();
    let mut security_notes = Vec::new();

    for (index, part) in parts.iter().enumerate() {
        if let Some((key, value)) = part.split_once('=') {
            if index == 0 {
                output.push(format!("Name: {}", key.trim()));
                output.push(format!("Value: {}", value.trim()));
            } else {
                output.push(format!("Attribute: {} = {}", key.trim(), value.trim()));
                let lower = key.trim().to_lowercase();
                if lower == "samesite" && value.trim().eq_ignore_ascii_case("none") && !trimmed.to_lowercase().contains("secure") {
                    security_notes.push("SameSite=None should usually be paired with Secure.".into());
                }
                if lower == "max-age" {
                    security_notes.push(format!("Max-Age indicates {} second(s).", value.trim()));
                }
                if lower == "expires" {
                    security_notes.push(format!("Expires at {}.", value.trim()));
                }
            }
        } else {
            output.push(format!("Flag: {}", part));
            if part.eq_ignore_ascii_case("httponly") {
                security_notes.push("HttpOnly prevents JavaScript access to the cookie.".into());
            }
            if part.eq_ignore_ascii_case("secure") {
                security_notes.push("Secure restricts the cookie to HTTPS transport.".into());
            }
        }
    }

    if security_notes.is_empty() {
        security_notes.push("No additional security notes detected.".into());
    }

    output.push(String::new());
    output.push("Security notes:".into());
    output.extend(security_notes.into_iter().map(|note| format!("- {}", note)));

    Ok(success(
        "mock.cookie-parser",
        "Cookie Parser",
        output.join("\n"),
        "Parsed cookie and session attributes.",
    ))
}

fn start_http_mock(port: u16) -> Result<(), AppError> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|error| AppError::Internal(format!("failed to bind mock server: {}", error)))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| AppError::Internal(format!("failed to configure listener: {}", error)))?;

    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();
    thread::spawn(move || {
        let listener = listener;
        while !stop_clone.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let _ = handle_http_mock_client(&mut stream);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(_) => break,
            }
        }
    });

    let mut state = HTTP_MOCK.lock().unwrap();
    if let Some(existing) = &state.stop {
        existing.store(true, Ordering::SeqCst);
    }
    state.running = true;
    state.port = port;
    state.stop = Some(stop);
    Ok(())
}

fn handle_http_mock_client(stream: &mut TcpStream) -> Result<(), AppError> {
    let request = read_http_request(stream)?;
    let request_line = request.lines().next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("GET").to_string();
    let path = parts.next().unwrap_or("/").to_string();
    let body = request.split("\r\n\r\n").nth(1).unwrap_or("").trim().to_string();

    let route = {
        let state = HTTP_MOCK.lock().unwrap();
        state
            .routes
            .iter()
            .find(|route| route.method.eq_ignore_ascii_case(&method) && route.path == path)
            .cloned()
    };

    let (status, content_type, response_body, extra_headers) = if let Some(route) = route {
        (route.status, route.content_type, route.body, route.headers)
    } else {
        (
            404,
            "application/json".into(),
            json!({"ok": false, "error": "route not found", "method": method, "path": path}).to_string(),
            Vec::new(),
        )
    };

    {
        let mut state = HTTP_MOCK.lock().unwrap();
        if state.logs.len() >= 100 {
            state.logs.pop_front();
        }
        state.logs.push_back(MockRequestLog {
            method: method.clone(),
            path: path.clone(),
            status,
            body: body.clone(),
        });
    }

    let reason = http_reason(status);
    let mut response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        status,
        reason,
        content_type,
        response_body.as_bytes().len()
    );
    for (key, value) in extra_headers {
        response.push_str(&format!("{}: {}\r\n", key, value));
    }
    response.push_str("\r\n");
    response.push_str(&response_body);
    stream
        .write_all(response.as_bytes())
        .map_err(|error| AppError::Internal(format!("failed to write mock response: {}", error)))
}

fn start_webhook_receiver(port: u16) -> Result<(), AppError> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|error| AppError::Internal(format!("failed to bind webhook receiver: {}", error)))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| AppError::Internal(format!("failed to configure webhook listener: {}", error)))?;

    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();
    let logs = WEBHOOK.lock().unwrap().logs.clone();
    thread::spawn(move || {
        let listener = listener;
        while !stop_clone.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let _ = capture_webhook_payload(&mut stream, &logs);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(_) => break,
            }
        }
    });

    let mut state = WEBHOOK.lock().unwrap();
    if let Some(existing) = &state.stop {
        existing.store(true, Ordering::SeqCst);
    }
    state.running = true;
    state.port = port;
    state.stop = Some(stop);
    Ok(())
}

fn capture_webhook_payload(stream: &mut TcpStream, logs: &Arc<Mutex<VecDeque<String>>>) -> Result<(), AppError> {
    let request = read_http_request(stream)?;
    let body = request.split("\r\n\r\n").nth(1).unwrap_or("").trim().to_string();
    let headers = request
        .split("\r\n\r\n")
        .next()
        .unwrap_or("")
        .lines()
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n");
    let first_line = request.lines().next().unwrap_or("");

    {
        let mut logs = logs.lock().unwrap();
        if logs.len() >= 100 {
            logs.pop_front();
        }
        logs.push_back(format!("{}\n{}\n\n{}", first_line, headers, body));
    }

    let state = WEBHOOK.lock().unwrap();
    let reason = http_reason(state.response_status);
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        state.response_status,
        reason,
        state.response_content_type,
        state.response_body.as_bytes().len(),
        state.response_body
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|error| AppError::Internal(format!("failed to write webhook response: {}", error)))
}

fn parse_http_mock_route(input: &str) -> Result<HttpMockRoute, AppError> {
    let mut lines = input.lines();
    let header = lines.next().unwrap_or_default().trim();
    let header_parts = header.split_whitespace().collect::<Vec<_>>();
    if header_parts.len() < 2 {
        return Err(AppError::Internal(
            "route format: route:METHOD /path status=200 type=application/json".into(),
        ));
    }

    let method = header_parts[0].to_uppercase();
    let path = header_parts[1].to_string();
    let mut status = 200u16;
    let mut content_type = "application/json".to_string();
    let mut headers = Vec::new();

    for token in header_parts.iter().skip(2) {
        if let Some(value) = token.strip_prefix("status=") {
            status = value.parse::<u16>().unwrap_or(200);
        } else if let Some(value) = token.strip_prefix("type=") {
            content_type = value.to_string();
        }
    }

    let mut body_lines = Vec::new();
    for line in lines {
        if let Some(value) = line.strip_prefix("header:") {
            if let Some((key, val)) = value.split_once('=') {
                headers.push((key.trim().into(), val.trim().into()));
            }
        } else if let Some(value) = line.strip_prefix("body:") {
            body_lines.push(value.to_string());
        } else {
            body_lines.push(line.to_string());
        }
    }

    let body = if body_lines.is_empty() {
        json!({"ok": true, "path": path, "method": method}).to_string()
    } else {
        body_lines.join("\n").trim().to_string()
    };

    Ok(HttpMockRoute {
        method,
        path,
        status,
        content_type,
        body,
        headers,
    })
}

#[derive(Clone)]
struct HttpProfile {
    url: String,
    method: String,
    requests: usize,
    concurrency: usize,
    headers: Vec<(String, String)>,
    body: String,
}

fn parse_http_profile(input: &str) -> HttpProfile {
    let mut profile = HttpProfile {
        url: "http://127.0.0.1:7878/".into(),
        method: "GET".into(),
        requests: 20,
        concurrency: 4,
        headers: Vec::new(),
        body: String::new(),
    };
    for line in input.lines() {
        if let Some(value) = line.strip_prefix("url:") {
            profile.url = value.trim().into();
        } else if let Some(value) = line.strip_prefix("method:") {
            profile.method = value.trim().to_uppercase();
        } else if let Some(value) = line.strip_prefix("requests:") {
            profile.requests = value.trim().parse::<usize>().unwrap_or(20).clamp(1, 5000);
        } else if let Some(value) = line.strip_prefix("concurrency:") {
            profile.concurrency = value.trim().parse::<usize>().unwrap_or(4).clamp(1, 256);
        } else if let Some(value) = line.strip_prefix("header:") {
            if let Some((key, val)) = value.split_once('=') {
                profile.headers.push((key.trim().into(), val.trim().into()));
            }
        } else if let Some(value) = line.strip_prefix("body:") {
            profile.body = value.to_string();
        }
    }
    profile
}

fn percentile(samples: &[u64], ratio: f64) -> u64 {
    if samples.is_empty() {
        return 0;
    }
    let index = ((samples.len() - 1) as f64 * ratio).round() as usize;
    samples[index.min(samples.len() - 1)]
}

struct WsEndpoint {
    host: String,
    port: u16,
    path: String,
}

fn parse_ws_endpoint(input: &str) -> Result<WsEndpoint, AppError> {
    let trimmed = input
        .strip_prefix("ws://")
        .or_else(|| input.strip_prefix("wss://"))
        .ok_or_else(|| AppError::Internal("WebSocket URL must start with ws:// or wss://".into()))?;
    let (host_port, path) = trimmed.split_once('/').unwrap_or((trimmed, ""));
    let (host, port) = if let Some((host, port)) = host_port.split_once(':') {
        (
            host.to_string(),
            port.parse::<u16>().map_err(|error| AppError::Internal(format!("invalid port: {}", error)))?,
        )
    } else {
        (host_port.to_string(), 80)
    };
    Ok(WsEndpoint {
        host,
        port,
        path: format!("/{}", path),
    })
}

fn websocket_handshake_text(endpoint: &WsEndpoint) -> Result<(String, String), AppError> {
    let mut stream = TcpStream::connect((&endpoint.host[..], endpoint.port))
        .map_err(|error| AppError::Internal(format!("websocket connect failed: {}", error)))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(|error| AppError::Internal(format!("failed to configure websocket timeout: {}", error)))?;
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}:{}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n",
        endpoint.path, endpoint.host, endpoint.port
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| AppError::Internal(format!("failed to send websocket handshake: {}", error)))?;
    let response = read_http_request(&mut stream)?;
    Ok((request, response))
}

fn websocket_upgrade(endpoint: &WsEndpoint) -> Result<TcpStream, AppError> {
    let mut stream = TcpStream::connect((&endpoint.host[..], endpoint.port))
        .map_err(|error| AppError::Internal(format!("websocket connect failed: {}", error)))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(|error| AppError::Internal(format!("failed to configure websocket timeout: {}", error)))?;
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}:{}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n",
        endpoint.path, endpoint.host, endpoint.port
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| AppError::Internal(format!("failed to send websocket handshake: {}", error)))?;
    let response = read_http_request(&mut stream)?;
    if !response.starts_with("HTTP/1.1 101") {
        return Err(AppError::Internal(format!(
            "websocket upgrade failed: {}",
            response.lines().next().unwrap_or("unknown response")
        )));
    }
    Ok(stream)
}

fn write_ws_text_frame(stream: &mut TcpStream, message: &str) -> Result<(), AppError> {
    let payload = message.as_bytes();
    let mut frame = Vec::new();
    frame.push(0x81);
    if payload.len() <= 125 {
        frame.push(0x80 | payload.len() as u8);
    } else {
        return Err(AppError::Internal("payload too large for simple websocket sender".into()));
    }
    let mask = [0x12, 0x34, 0x56, 0x78];
    frame.extend_from_slice(&mask);
    for (index, byte) in payload.iter().enumerate() {
        frame.push(byte ^ mask[index % 4]);
    }
    stream
        .write_all(&frame)
        .map_err(|error| AppError::Internal(format!("failed to write websocket frame: {}", error)))
}

fn read_ws_text_frame(stream: &mut TcpStream) -> Result<String, AppError> {
    let mut header = [0u8; 2];
    stream
        .read_exact(&mut header)
        .map_err(|error| AppError::Internal(format!("failed to read websocket frame header: {}", error)))?;
    let masked = header[1] & 0x80 != 0;
    let length = (header[1] & 0x7F) as usize;
    if length >= 126 {
        return Err(AppError::Internal("extended websocket frames are not supported by the simple reader".into()));
    }
    let mut mask = [0u8; 4];
    if masked {
        stream
            .read_exact(&mut mask)
            .map_err(|error| AppError::Internal(format!("failed to read websocket mask: {}", error)))?;
    }
    let mut payload = vec![0u8; length];
    stream
        .read_exact(&mut payload)
        .map_err(|error| AppError::Internal(format!("failed to read websocket payload: {}", error)))?;
    if masked {
        for (index, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[index % 4];
        }
    }
    Ok(String::from_utf8_lossy(&payload).to_string())
}

fn parse_prefixed_fields(input: &str) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    let mut current_key = String::new();
    let mut current_value = String::new();

    for line in input.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let is_new_field = key.chars().all(|char| char.is_ascii_alphanumeric() || char == '-' || char == '_');
            if is_new_field {
                if !current_key.is_empty() {
                    fields.insert(current_key.clone(), current_value.trim().to_string());
                }
                current_key = key.trim().to_lowercase();
                current_value = value.trim().to_string();
                continue;
            }
        }
        if !current_value.is_empty() {
            current_value.push('\n');
        }
        current_value.push_str(line);
    }

    if !current_key.is_empty() {
        fields.insert(current_key, current_value.trim().to_string());
    }
    fields
}

fn introspection_query() -> String {
    "query IntrospectionQuery { __schema { queryType { name } mutationType { name } types { name kind fields { name } } } }".into()
}

fn to_title_case(input: &str) -> String {
    let mut chars = input.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.collect::<String>()),
        None => String::new(),
    }
}

fn parse_rest_request(name_line: &str, full_input: &str) -> Result<RestRequest, AppError> {
    let name = name_line.lines().next().unwrap_or_default().trim().to_string();
    let mut method = "GET".to_string();
    let mut url = String::new();
    let mut headers = Vec::new();
    let mut body_lines = Vec::new();
    let mut in_body = false;

    for line in full_input.lines().skip(1) {
        if let Some(value) = line.strip_prefix("method:") {
            method = value.trim().to_uppercase();
            in_body = false;
        } else if let Some(value) = line.strip_prefix("url:") {
            url = value.trim().into();
            in_body = false;
        } else if let Some(value) = line.strip_prefix("header:") {
            if let Some((key, val)) = value.split_once('=') {
                headers.push((key.trim().into(), val.trim().into()));
            }
            in_body = false;
        } else if let Some(value) = line.strip_prefix("body:") {
            body_lines.push(value.to_string());
            in_body = true;
        } else if in_body {
            body_lines.push(line.to_string());
        }
    }

    if name.is_empty() || url.is_empty() {
        return Err(AppError::Internal(
            "request name and url are required for add:<name> entries".into(),
        ));
    }

    Ok(RestRequest {
        name,
        method,
        url,
        headers,
        body: body_lines.join("\n").trim().to_string(),
    })
}

fn parse_response_config(first_line: &str, full_input: &str) -> Result<(u16, String, String), AppError> {
    let mut status = 202u16;
    let mut content_type = "application/json".to_string();
    for token in first_line.split_whitespace() {
        if let Some(value) = token.strip_prefix("status=") {
            status = value.parse::<u16>().unwrap_or(202);
        } else if let Some(value) = token.strip_prefix("type=") {
            content_type = value.to_string();
        }
    }

    let mut body_lines = Vec::new();
    for line in full_input.lines().skip(1) {
        if let Some(value) = line.strip_prefix("body:") {
            body_lines.push(value.to_string());
        } else {
            body_lines.push(line.to_string());
        }
    }
    let body = body_lines.join("\n").trim().to_string();
    Ok((status, content_type, body))
}

fn execute_http_json(
    method: &str,
    url: &str,
    headers: &[(&str, &str)],
    body: &str,
) -> Result<String, AppError> {
    let owned_headers = headers.iter().map(|(key, value)| (key.to_string(), value.to_string())).collect::<Vec<_>>();
    let response = simple_http_request(method, url, &owned_headers, body)?;
    Ok(response.1)
}

fn execute_http_text(
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: &str,
) -> Result<String, AppError> {
    let (status, response) = simple_http_request(method, url, headers, body)?;
    Ok(format!("Status: {}\n\n{}", status, response))
}

fn simple_http_request(
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: &str,
) -> Result<(u16, String), AppError> {
    let trimmed = url.trim().trim_start_matches("http://");
    let (host_port, path) = trimmed.split_once('/').unwrap_or((trimmed, ""));
    let mut stream = TcpStream::connect(host_port)
        .map_err(|error| AppError::Internal(format!("connect failed: {}", error)))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| AppError::Internal(format!("timeout setup failed: {}", error)))?;
    let mut request = format!(
        "{} /{} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nContent-Length: {}\r\n",
        method,
        path,
        host_port,
        body.as_bytes().len()
    );
    for (key, value) in headers {
        request.push_str(&format!("{}: {}\r\n", key, value));
    }
    request.push_str("\r\n");
    request.push_str(body);
    stream
        .write_all(request.as_bytes())
        .map_err(|error| AppError::Internal(format!("request write failed: {}", error)))?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|error| AppError::Internal(format!("response read failed: {}", error)))?;
    let status = response
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(0);
    let body = response.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    Ok((status, body))
}

fn read_http_request(stream: &mut TcpStream) -> Result<String, AppError> {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| AppError::Internal(format!("failed to set read timeout: {}", error)))?;
    let mut buffer = [0u8; 8192];
    let read = stream
        .read(&mut buffer)
        .map_err(|error| AppError::Internal(format!("failed to read request: {}", error)))?;
    Ok(String::from_utf8_lossy(&buffer[..read]).to_string())
}

fn http_reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        409 => "Conflict",
        422 => "Unprocessable Entity",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Response",
    }
}

fn format_lines(items: &[String]) -> String {
    if items.is_empty() {
        "(none)".into()
    } else {
        items.iter().map(|item| format!("- {}", item)).collect::<Vec<_>>().join("\n")
    }
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
