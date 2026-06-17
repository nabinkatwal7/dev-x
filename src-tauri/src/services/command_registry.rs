use crate::models::{CommandAction, CommandCategory};

pub struct CommandRegistry {
    commands: Vec<CommandAction>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: core_commands(),
        }
    }

    pub fn commands(&self) -> &[CommandAction] {
        &self.commands
    }
}

fn core_commands() -> Vec<CommandAction> {
    vec![
        // === Data ===
        CommandAction {
            id: "data.format-json".into(),
            title: "Format JSON".into(),
            subtitle: "Pretty-print or minify JSON payloads locally.".into(),
            category: CommandCategory::Data,
            tags: vec!["json".into(), "formatter".into(), "payload".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "data.minify-json".into(),
            title: "Minify JSON".into(),
            subtitle: "Compress JSON payloads into a single line locally.".into(),
            category: CommandCategory::Data,
            tags: vec!["json".into(), "minify".into(), "payload".into()],
            shortcut: None,
            accepts_input: true,
        },
        // === 022: Schema Inter-Converter ===
        CommandAction {
            id: "data.schema-convert".into(),
            title: "Schema Convert".into(),
            subtitle: "Convert between JSON, YAML, TOML, XML, and CSV. Prefix with 'to: yaml'.".into(),
            category: CommandCategory::Data,
            tags: vec!["convert".into(), "yaml".into(), "toml".into(), "xml".into(), "csv".into()],
            shortcut: None,
            accepts_input: true,
        },
        // === 023: JSON Flatten / Unflatten ===
        CommandAction {
            id: "data.json-flatten".into(),
            title: "JSON Flatten".into(),
            subtitle: "Flatten nested JSON objects into dot-notation key=value paths.".into(),
            category: CommandCategory::Data,
            tags: vec!["flatten".into(), "normalize".into(), "paths".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "data.json-unflatten".into(),
            title: "JSON Unflatten".into(),
            subtitle: "Rebuild nested JSON from flat dot-notation key=value pairs.".into(),
            category: CommandCategory::Data,
            tags: vec!["unflatten".into(), "rebuild".into(), "paths".into()],
            shortcut: None,
            accepts_input: true,
        },
        // === 024: SQL Statement Beautifier ===
        CommandAction {
            id: "data.sql-beautify".into(),
            title: "SQL Beautify".into(),
            subtitle: "Format raw SQL queries with proper indentation and line breaks.".into(),
            category: CommandCategory::Data,
            tags: vec!["sql".into(), "format".into(), "beautify".into(), "query".into()],
            shortcut: None,
            accepts_input: true,
        },
        // === 025: String Escaper / Unescaper ===
        CommandAction {
            id: "data.escape".into(),
            title: "String Escape".into(),
            subtitle: "Escape text for JSON strings, HTML attributes, or terminal/shell.".into(),
            category: CommandCategory::Data,
            tags: vec!["escape".into(), "json".into(), "html".into(), "terminal".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "data.unescape".into(),
            title: "String Unescape".into(),
            subtitle: "Unescape JSON, HTML, or terminal-escaped text.".into(),
            category: CommandCategory::Data,
            tags: vec!["unescape".into(), "json".into(), "html".into(), "terminal".into()],
            shortcut: None,
            accepts_input: true,
        },
        // === 026: CSV/TSV Visual Matrix ===
        CommandAction {
            id: "data.csv-table".into(),
            title: "CSV Table".into(),
            subtitle: "Render CSV or TSV data as a formatted text table.".into(),
            category: CommandCategory::Data,
            tags: vec!["csv".into(), "tsv".into(), "table".into(), "grid".into()],
            shortcut: None,
            accepts_input: true,
        },
        // === 027: Type Definition Generator ===
        CommandAction {
            id: "data.gen-types".into(),
            title: "Generate Types".into(),
            subtitle: "Generate TypeScript, Go, or Rust types from JSON. Prefix with 'ts:', 'go:', or 'rust:'.".into(),
            category: CommandCategory::Data,
            tags: vec!["types".into(), "generate".into(), "typescript".into(), "go".into(), "rust".into()],
            shortcut: None,
            accepts_input: true,
        },
        // === 028: Structural Schema Differ ===
        CommandAction {
            id: "data.struct-diff".into(),
            title: "Struct Diff".into(),
            subtitle: "Compare two JSON payloads separated by '---' on its own line.".into(),
            category: CommandCategory::Data,
            tags: vec!["diff".into(), "compare".into(), "schema".into(), "changes".into()],
            shortcut: None,
            accepts_input: true,
        },
        // === 029: URL Query Parameter Deconstructor ===
        CommandAction {
            id: "data.url-parse".into(),
            title: "URL Parse".into(),
            subtitle: "Deconstruct URLs into scheme, host, path, and query parameters.".into(),
            category: CommandCategory::Data,
            tags: vec!["url".into(), "parse".into(), "query".into(), "parameters".into()],
            shortcut: None,
            accepts_input: true,
        },
        // === 030: JSONPath Evaluator ===
        CommandAction {
            id: "data.path-eval".into(),
            title: "Path Eval".into(),
            subtitle: "Evaluate JSONPath expressions against JSON documents.".into(),
            category: CommandCategory::Data,
            tags: vec!["jsonpath".into(), "path".into(), "query".into(), "evaluate".into()],
            shortcut: None,
            accepts_input: true,
        },
        // === 011: Zero-Latency Process Port Mapper ===
        CommandAction {
            id: "net.port-mapper".into(),
            title: "Port Mapper".into(),
            subtitle: "Real-time list of all active TCP/UDP listeners with PID and binary name.".into(),
            category: CommandCategory::Network,
            tags: vec!["ports".into(), "listeners".into(), "process".into(), "tcp".into(), "udp".into()],
            shortcut: None,
            accepts_input: true,
        },
        // === 012: Atomic Process Termination ===
        CommandAction {
            id: "net.kill-process".into(),
            title: "Kill Process".into(),
            subtitle: "Terminate a process by PID with graceful or forced kill.".into(),
            category: CommandCategory::Network,
            tags: vec!["kill".into(), "process".into(), "pid".into(), "terminate".into()],
            shortcut: None,
            accepts_input: true,
        },
        // === 013: Multiplexed Port Monitor ===
        CommandAction {
            id: "net.port-monitor".into(),
            title: "Port Monitor".into(),
            subtitle: "Check if a port is in use and identify the occupying process.".into(),
            category: CommandCategory::Network,
            tags: vec!["port".into(), "monitor".into(), "check".into(), "listen".into()],
            shortcut: None,
            accepts_input: true,
        },
        // === 014: Local DNS Host File Editor ===
        CommandAction {
            id: "net.hosts-edit".into(),
            title: "Hosts Editor".into(),
            subtitle: "View, add, remove, or toggle entries in the system hosts file.".into(),
            category: CommandCategory::Network,
            tags: vec!["hosts".into(), "dns".into(), "etc".into(), "loopback".into()],
            shortcut: None,
            accepts_input: true,
        },
        // === 015: Reverse Proxy Tunnel Configuration Manager ===
        CommandAction {
            id: "net.tunnel-mgr".into(),
            title: "Tunnel Manager".into(),
            subtitle: "Manage ngrok, localtunnel, and cloudflared tunnels.".into(),
            category: CommandCategory::Network,
            tags: vec!["tunnel".into(), "ngrok".into(), "proxy".into(), "reverse".into()],
            shortcut: None,
            accepts_input: true,
        },
        // === 016: Network Interception Curl Builder ===
        CommandAction {
            id: "net.curl-builder".into(),
            title: "Curl Builder".into(),
            subtitle: "Convert raw HTTP request logs into reproducible curl commands.".into(),
            category: CommandCategory::Network,
            tags: vec!["curl".into(), "http".into(), "request".into(), "convert".into()],
            shortcut: None,
            accepts_input: true,
        },
        // === 017a: ICMP Ping Analyzer ===
        CommandAction {
            id: "net.ping".into(),
            title: "Ping".into(),
            subtitle: "Send ICMP echo requests and analyze latency, loss, and statistics.".into(),
            category: CommandCategory::Network,
            tags: vec!["ping".into(), "icmp".into(), "latency".into(), "diagnostic".into()],
            shortcut: None,
            accepts_input: true,
        },
        // === 017b: Traceroute Analyzer ===
        CommandAction {
            id: "net.trace".into(),
            title: "Traceroute".into(),
            subtitle: "Visualize network hops to a target host via ICMP or UDP probes.".into(),
            category: CommandCategory::Network,
            tags: vec!["traceroute".into(), "trace".into(), "hops".into(), "route".into()],
            shortcut: None,
            accepts_input: true,
        },
        // === 018: Public/Private IP Discovery ===
        CommandAction {
            id: "net.ip-discover".into(),
            title: "IP Discovery".into(),
            subtitle: "Show local network config, public IP, subnet mask, gateway, and MAC addresses.".into(),
            category: CommandCategory::Network,
            tags: vec!["ip".into(), "network".into(), "config".into(), "gateway".into(), "subnet".into()],
            shortcut: None,
            accepts_input: false,
        },
        // === 019: Domain & SSL Expiry Validator ===
        CommandAction {
            id: "net.domain-check".into(),
            title: "Domain Check".into(),
            subtitle: "Resolve DNS records and validate SSL certificate trust chain and expiry.".into(),
            category: CommandCategory::Network,
            tags: vec!["domain".into(), "ssl".into(), "dns".into(), "certificate".into(), "expiry".into()],
            shortcut: None,
            accepts_input: true,
        },
        // === 020: Local Subnet Sweep Engine ===
        CommandAction {
            id: "net.subnet-sweep".into(),
            title: "Subnet Sweep".into(),
            subtitle: "Fast async IP scan to discover active devices on the local subnet.".into(),
            category: CommandCategory::Network,
            tags: vec!["subnet".into(), "scan".into(), "discovery".into(), "network".into()],
            shortcut: None,
            accepts_input: true,
        },
        // ============================================================
        // Features 031–040: Advanced Cryptography & Key Management
        // ============================================================
        CommandAction {
            id: "crypto.hash".into(),
            title: "Hash Compute".into(),
            subtitle: "Compute MD5, SHA-1, SHA-256, SHA-512, and CRC32 hashes.".into(),
            category: CommandCategory::Crypto,
            tags: vec!["hash".into(), "md5".into(), "sha".into(), "crc32".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "crypto.jwt".into(),
            title: "JWT Inspect".into(),
            subtitle: "Decode JWT tokens to inspect header, payload, and signature.".into(),
            category: CommandCategory::Crypto,
            tags: vec!["jwt".into(), "token".into(), "decode".into(), "inspect".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "crypto.cipher".into(),
            title: "Symmetric Cipher".into(),
            subtitle: "Encrypt/decrypt with AES-256-CBC or ChaCha20. Usage: encrypt:aes256:<keyhex>:<text>".into(),
            category: CommandCategory::Crypto,
            tags: vec!["aes".into(), "chacha".into(), "encrypt".into(), "decrypt".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "crypto.rsa-keygen".into(),
            title: "RSA Keygen".into(),
            subtitle: "Generate RSA public/private keypair (default 2048 bits).".into(),
            category: CommandCategory::Crypto,
            tags: vec!["rsa".into(), "keygen".into(), "public".into(), "private".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "crypto.base64".into(),
            title: "Base64 Convert".into(),
            subtitle: "Encode text to base64 or decode base64 with 'decode:' prefix.".into(),
            category: CommandCategory::Crypto,
            tags: vec!["base64".into(), "encode".into(), "decode".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "crypto.hash-bench".into(),
            title: "Hash Benchmark".into(),
            subtitle: "Hash passwords with bcrypt, argon2, or scrypt.".into(),
            category: CommandCategory::Crypto,
            tags: vec!["bcrypt".into(), "argon2".into(), "scrypt".into(), "password".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "crypto.html-decode".into(),
            title: "HTML Decode".into(),
            subtitle: "Decode HTML entities, hex escapes, and URL-encoded sequences.".into(),
            category: CommandCategory::Crypto,
            tags: vec!["html".into(), "decode".into(), "entities".into(), "hex".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "crypto.gen-token".into(),
            title: "Generate Token".into(),
            subtitle: "Generate UUIDs, hex tokens, passwords, or PINs.".into(),
            category: CommandCategory::Crypto,
            tags: vec!["token".into(), "uuid".into(), "password".into(), "generate".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "crypto.hmac".into(),
            title: "HMAC Compute".into(),
            subtitle: "Compute HMAC-SHA256/512 signatures. Key on first line, message on second.".into(),
            category: CommandCategory::Crypto,
            tags: vec!["hmac".into(), "sha256".into(), "sha512".into(), "signature".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "crypto.vault".into(),
            title: "Crypto Vault".into(),
            subtitle: "In-memory encrypted vault for staging secrets. Commands: set, get, delete, list, clear".into(),
            category: CommandCategory::Crypto,
            tags: vec!["vault".into(), "secret".into(), "key".into(), "storage".into()],
            shortcut: None,
            accepts_input: true,
        },
        // ============================================================
        // Features 041–050: Clipboard & Text Processing
        // ============================================================
        CommandAction {
            id: "clip.stack".into(),
            title: "Clipboard Stack".into(),
            subtitle: "Push, list, and retrieve clipboard history items.".into(),
            category: CommandCategory::Clipboard,
            tags: vec!["clipboard".into(), "history".into(), "stack".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "clip.classify".into(),
            title: "Classify Text".into(),
            subtitle: "Auto-detect semantic type of pasted content (JSON, SQL, URL, etc).".into(),
            category: CommandCategory::Clipboard,
            tags: vec!["classify".into(), "detect".into(), "semantic".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "clip.snippets".into(),
            title: "Snippet Board".into(),
            subtitle: "Store and retrieve sticky text snippets. Commands: add, get, del, list, clear".into(),
            category: CommandCategory::Clipboard,
            tags: vec!["snippet".into(), "sticky".into(), "board".into(), "template".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "clip.merge-split".into(),
            title: "Merge / Split".into(),
            subtitle: "Merge lines with separator or split text by separator.".into(),
            category: CommandCategory::Clipboard,
            tags: vec!["merge".into(), "split".into(), "join".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "clip.regex".into(),
            title: "Regex Transform".into(),
            subtitle: "Find-and-replace using regex patterns, or strip matches.".into(),
            category: CommandCategory::Clipboard,
            tags: vec!["regex".into(), "replace".into(), "strip".into(), "find".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "clip.case".into(),
            title: "Case Normalize".into(),
            subtitle: "Convert text between camelCase, snake_case, kebab-case, and more.".into(),
            category: CommandCategory::Clipboard,
            tags: vec!["case".into(), "transform".into(), "camel".into(), "snake".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "clip.diff".into(),
            title: "Clipboard Diff".into(),
            subtitle: "Compare two texts separated by '---' and show line differences.".into(),
            category: CommandCategory::Clipboard,
            tags: vec!["diff".into(), "compare".into(), "lines".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "clip.whitespace".into(),
            title: "Whitespace Sanitizer".into(),
            subtitle: "Minify, trim, or collapse whitespace in text blocks.".into(),
            category: CommandCategory::Clipboard,
            tags: vec!["whitespace".into(), "minify".into(), "trim".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "clip.redact".into(),
            title: "Redact Data".into(),
            subtitle: "Detect and redact API keys, passwords, emails, IPs, and credit cards.".into(),
            category: CommandCategory::Clipboard,
            tags: vec!["redact".into(), "security".into(), "privacy".into(), "filter".into()],
            shortcut: None,
            accepts_input: true,
        },
        CommandAction {
            id: "clip.queue".into(),
            title: "Clip Queue".into(),
            subtitle: "FIFO multi-item queue for form-filling. Commands: push, pop, list, clear".into(),
            category: CommandCategory::Clipboard,
            tags: vec!["queue".into(), "fifo".into(), "stack".into(), "form".into()],
            shortcut: None,
            accepts_input: true,
        },
    ]
}
