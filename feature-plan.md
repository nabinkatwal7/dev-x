# Product Requirement Document & Feature Roadmap

## Project: DevForge — The Local Desktop Developer Supertool

**Document Version:** 1.0.0
**Target Architecture:** Desktop Native (Tauri + Rust Backend + Svelte/TypeScript Frontend)
**Execution Environment:** 100% Local, Offline-First, Zero-Cloud Reliance

---

## Executive Summary & Vision

Developers lose hours every week to **micro-frictions**: opening browser tabs to format JSON, pasting sensitive tokens into untrusted web decoders, running terminal loops to kill locked ports, and losing track of multi-step clipboard histories.

**DevForge** is an open-core, hyper-lightweight local desktop application designed to eliminate these context switches. By combining native system integration (Rust) with a keyboard-driven interface, DevForge acts as a unified command palette and utility suite. It operates with a **zero-latency, zero-cloud, privacy-first** paradigm.

---

## Core System Architecture & Guardrails

1. **Memory & Performance Footprint:** Idle RAM must remain under **45MB**. Maximum binary size must stay under **30MB** across platforms (macOS, Windows, Linux) via Tauri's native webview bindings.
2. **Security & Privacy Isolation:** Zero external analytics, zero tracking, zero background telemetry. All operations (parsing, conversion, extraction) happen strictly in local memory.
3. **Keyboard-First Interface:** Every single feature must be fully accessible via custom global hotkeys and an integrated **Command Palette (`Option + Space` / `Alt + Space`)** without requiring mouse interaction.
4. **Plugin Extensibility:** A robust local API allowing users to drop custom JavaScript/TypeScript scripts or WASM binaries into a specific local folder to expand the command library automatically.

---

## Global Framework Matrix

The following matrix maps out the 100 mandatory features across 10 functional pillars required for a complete minimum viable product and immediate iteration roadmap.

### 1. Unified Command Launcher & System Core [done]

- **001. Instant Overlay Engine:** Global hotkey toggle (`Alt+Space` or `Cmd+Space`) showing an un-focused, instantly reactive overlay mimicking Spotlight/Raycast.
- **002. Fuzzy-Match Omnibar:** Custom fuzzy sorting algorithm prioritizing exact matches, historical execution frequencies, and structural tags.
- **003. Dynamic Result Previews:** Split-pane interface showcasing immediate output layout transformations dynamically as the user types commands.
- **004. Zero-Lag System Tray Agent:** Background daemon utilizing low-overhead Rust threads to monitor hotkeys while the primary window is unmapped.
- **005. Configurable Workspace Profiles:** Context-aware layouts switching tools, hotkeys, and defaults depending on active environment tags (e.g., Frontend, DevOps, Security).
- **006. Standalone Modular Window Pinning:** Ability to tear away any specific module (like a regex tester) into an independent, always-on-top micro-window.
- **007. Interactive Local Action History:** A structured, searchable ledger of commands invoked within DevForge with one-click re-execution capabilities.
- **008. Custom Script Extension Loader:** A dedicated hot-reloading directory watcher executing third-party UI-less tools mapped directly into the omnibar.
- **009. Layout/Theme Engine Matrix:** Full Tailwind-driven theme selector with native support for matching system light/dark/contrast modes instantly.
- **010. Advanced Export/Import Configurations:** Single-file encrypted JSON synchronization mechanism for migrating custom aliases, settings, and snippets between systems.

### 2. Network, Port & Process Wrangling [done]

- **011. Zero-Latency Process Port Mapper:** Real-time visual list displaying all active local TCP/UDP listeners paired with process IDs (PID) and binary names.
- **012. Atomic Process Termination (Force Kill):** Integrated administrative mechanism to trigger instant graceful (`SIGTERM`) or immediate hard (`SIGKILL`) process terminations.
- **013. Multiplexed Port Monitor:** Live active notification triggers alerting the developer immediately when background processes hijack designated debugging ports (e.g., 3000, 8080).
- **014. Local DNS Host File Editor:** Secure interface to quickly read, parse, comment, and insert custom loopbacks into system `/etc/hosts` or Windows equivalent.
- **015. Reverse Proxy Tunnel Configuration Manager:** Local wrapper to manage, initialize, and visualize status pipelines for tools like ngrok, localtunnel, or custom SSH reverse forwards.
- **016. Network Interception Curl Builder:** Raw raw socket log viewer capable of converting intercepted outbound packets into reproducible `curl` commands.
- **017. ICMP Ping / Traceroute Analyzer:** High-precision parallelized diagnostic tools visualizing internal and external hops via clear text charts.
- **018. Public/Private IP Discovery & Geolocation Check:** Offline lookup mechanisms detailing subnet masks, router gateways, and external routing paths.
- **019. Domain Name & SSL Expiry Validator:** Quick checker analyzing target domain certificates, showing trust chains, encryption standards, and days-remaining counters.
- **020. Local Subnet Sweep Engine:** Fast asynchronous IP scanning utility locating network devices, internal developer sandboxes, and active headless servers.

### 3. Structural Data Transpilation & Formatting [done]

- **021. Streaming JSON Formatter & Minifier:** Ultra-high-speed parser capable of structural layout formatting and flattening of multiline JSON blocks.
- **022. Schema Inter-Converter Engine:** Bi-directional real-time schema converter translating between JSON, YAML, XML, TOML, and CSV structures instantly.
- **023. Smart JSON Flatten & Unflatten Utility:** Normalizer breaking complex nested array-objects into flat key-value paths or rebuilding path strings back into nested trees.
- **024. Dynamic SQL Statement Beautifier:** SQL query parsing engine formatting dense, unreadable raw query logs into cleanly structured, indented syntax.
- **025. Code to Clean String Escaper:** Utility to escape/unescape structural codeblocks for safe insertion into JSON strings, HTML attributes, or terminal blocks.
- **026. Interactive CSV/TSV Visual Matrix:** Tabular grid generator that processes raw comma/tab data into filterable, sortable interactive data tables.
- **027. Type-Definition Automated Generator:** Code generator producing accurate TypeScript interfaces, Go structs, or Rust types directly from pasted JSON payloads.
- **028. Structural Schema Differ:** Highly precise text diffing matrix optimized for showing additions, removals, and balance mutations between structural payloads.
- **029. URL Query Parameter Deconstructor:** Analyzer breaking complex URLs with multi-layered tracking parameters into editable, structured query keys.
- **030. XML Path (XPath) & JSONPath Evaluator:** Live query evaluator extracting nodes or properties from massive document structures via custom path expressions.

### 4. Advanced Cryptography & Key Management [done]

- **031. Native Multi-Hash Compute Sandbox:** Parallel calculator computing MD5, SHA-1, SHA-256, SHA-512, and CRC32 signatures over text strings or system binaries.
- **032. Comprehensive JWT Inspection Deck:** Cryptographic decoder parsing JSON Web Tokens to expose payloads, header algos, issuance times, and signature states.
- **033. Symmetric Cipher Laboratory:** Sandbox facilitating standard text transformations utilizing custom AES-128/256 or ChaCha20 algorithms with manual keys.
- **034. Asymmetric RSA Keypair Engine:** Generator producing highly secure RSA/Ed25519 public-private keypairs with simple copy-to-clipboard formatting.
- **035. Base64 Binary/Text Converter Tool:** Fast translation engine encoding text to base64 variants or converting base64 strings back to binaries/images.
- **036. Variable Hashing Benchmark Lab:** Sandbox validating, hashing, and comparing text strings against Bcrypt, Scrypt, and Argon2 ID frameworks.
- **037. HTML Entity & Hex Decoder:** Encoder resolving hexadecimal strings, escape chars, and character entities back into native readable UTF-8 strings.
- **038. Cryptographically Secure Token Generator:** High-entropy string builder compiling random API keys, UUID v4/v7 strings, passwords, or salts.
- **039. HMAC Signature Computational Matrix:** Validation tool checking webhook signatures across various provider payloads using manual tracking keys.
- **040. Local Cryptographic Vault Sandbox:** Secure, memory-encrypted notepad designed for staging transient production API keys during intensive debug sessions.

### 5. Developer-Centric Clipboard & History Engine [done]

- **041. Multi-Format Structural Clipboard Stack:** Continuous internal clipboard tracker capturing text, images, files, and rich structures into a history pane.
- **042. Semantic Contextual Data Classifier:** Background scanner categorizing copied content automatically (e.g., IP addresses, Colors, SQL queries, Dates).
- **043. Sticky Snippet Injection Board:** Area allowing users to pin frequently repeated template items to make them persistently available via quick hotkeys.
- **044. Bulk Content Merger & Splitter Suite:** Transformer enabling users to combine multiple copied elements into single lines or split single lines into array blocks.
- **045. RegEx Strip & Regex Find-and-Replace:** Global transform engine running advanced regex modifications over entire historical clipboard text blocks.
- **046. Text Case Normalizer Engine:** One-click utility shifting text arrays across `camelCase`, `snake_case`, `kebab-case`, `PASCAL_CASE`, and `UPPERCASE`.
- **047. Clipboard Diff Comparison Bridge:** Utility comparing the text structure of the currently copied item directly against the immediately preceding clip.
- **048. Whitespace Sanitizer & Minifier:** Utility analyzing strings to strip double spaces, remove leading/trailing carriage returns, or format single lines.
- **049. Automated Clipboard Sensitive Data Redactor:** Scanner matching and masking credit cards, passwords, and API secret strings within the history interface.
- **050. Multi-Item Queue Stack Sequence:** Pipeline allowing sequential popping of items from history, perfect for multi-field form-filling exercises.

### 6. Code Snippet Vault & Shell Assistant

- **051. Shell Script Cheat Sheet Bank:** Fast search engine compiling common command arguments for utilities like `tar`, `find`, `sed`, `awk`, and `chmod`.
- **052. Automated Git Reconstruction Wizard:** Blueprint engine outputting command structures for complex operations like interactive rebases and hard commits.
- **053. Syntactically Highlighted Code Vault:** Isolated snippet saver with syntax engines supporting over 40 popular programming languages.
- **054. Local Shell History Parser Engine:** Interface searching local system logs (`.zsh_history` / `.bash_history`) via fuzzy matching for immediate reuse.
- **055. Native CLI Alias Blueprint Generator:** Interactive configurator translating long multi-parameter commands into shell aliases for easy system exporting.
- **056. Cross-Platform Path Translator Tool:** Converter altering system file paths seamlessly between POSIX standards and Windows-style formatting.
- **057. Cron Pattern Deconstructor Workbench:** Real-time evaluator translating obscure crontab asterisk schedules into human-readable timetables.
- **058. Docker Compose Micro-Architect Canvas:** Text wizard producing valid multi-service configurations through an interactive form interface.
- **059. Terminal ANSI Escape Code Stripper:** Tool stripping terminal formatting variables, syntax flags, and color tags out of logs to yield clean text.
- **060. Standard Exit Code Diagnostic Reference:** Directory explaining numeric unix/windows exit codes to clarify cryptic script termination states.

### 7. File System & Environment Organizer

- **061. Multi-Environment `.env` Audit Matrix:** Viewer tracking multiple environment files side-by-side to detect missing keys or value discrepancies.
- **062. Duplicate File Discovery Scanner:** Local storage indexing mechanism locating identical files by cross-referencing byte size and hash checksum signatures.
- **063. Dynamic Symlink Matrix Constructor:** Visual manager creating, assessing, tracking, and breaking structural absolute/relative system symlinks.
- **064. Massive Text Log Tail Streaming Tool:** Ultra-fast IO reader streaming massive multi-gigabyte server logs without hitting memory allocation walls.
- **065. Real-Time Directory File Change Sentinel:** Background file monitor identifying creations, updates, or deletions within selected repositories.
- **066. Binary / Hexadecimal Deep Inspector:** Low-level viewer loading application compilation assets to expose data structures and internal markers.
- **067. Multi-Target Batch File Renaming Deck:** Tool applying structured naming systems, dates, counters, or regex updates across massive sets of assets.
- **068. Absolute Disk Space Allocation Explorer:** Storage mapping module highlighting large build directories (`node_modules`, `target`, `vendor`) for immediate cleanup.
- **069. File Encoding Conversion Bridge:** Translation layer resolving file encoding mismatches by shifting files cleanly between UTF-8, UTF-16, and ASCII.
- **070. Secure Archive Integrity Sandbox:** Extraction engine evaluating and validating zip, tar, gzip, and 7z architectures before writing files locally.

### 8. Frontend/UI Design Mechanics

- **071. Precision System Eyedropper Tool:** Native magnifier zooming into pixels to extract clean hexadecimal, RGB, HSL, or Swift color signatures.
- **072. Comprehensive Color Format Swapper:** Live palette space utility converting colors between Hex, RGB, HSL, CMYK, and CSS string syntaxes.
- **073. SVG Path Sanitizer & Optimization Laboratory:** Scalable vector graphics optimizer stripping bloat, structural coordinates, and metadata tags out of code blocks.
- **074. Relative Typography Scale Evaluator:** CSS pixel translator converting static font weights into proportional REM, EM, and viewport units.
- **075. Dynamic Display Aspect Ratio Sandbox:** Dimensional calculator verifying proportional width/height targets for responsive layouts.
- **076. CSS Flexbox & Grid Direct Code Constructor:** Visual styling canvas producing clean, modern structural layouts through slider inputs.
- **077. Target Asset Mock Data Engine:** Fast payload generator producing realistic mockup data arrays (names, emails, avatars) for layout prototyping.
- **078. Contrast Ratio Verification Console:** Color verification system grading foreground and background pairs against strict WCAG AA/AAA readability compliance scales.
- **079. CSS Shadow & Gradient Code Generator:** Layer designer creating standard multi-stop gradient steps and dropping copy-paste code arrays.
- **080. Local Font Family Inventory Indexer:** Font reference engine rendering typography specimens for all fonts currently installed on the host operating system.

### 9. Privacy-Preserving Local AI & Ollama Integration

- **081. Local LLM Runtime Gateway Bridge:** Direct socket pipeline connecting to local Ollama endpoints without routing data outside the machine.
- **082. Context-Aware Error Log Explanation Canvas:** Dedicated diagnostic input explaining terminal compilation stack traces using local compute models.
- **083. Local Code Optimization Analyst:** Inline review engine analyzing structure, runtime performance, and vulnerability points locally.
- **084. Semantic Local Snippet Retrieval Agent:** AI search engine analyzing saved snippets using local embedding spaces rather than simple text keywords.
- **085. SQL Query Natural Language Translator:** Assistant transforming basic English instructions into well-formed SQL query patterns via local models.
- **086. Markdown Documentation Drafting Wizard:** Framework generating basic function README structures and usage blocks via local LLM runtimes.
- **087. Structural Test Suite Automated Scaffold:** Code assistant outputting boilerplate unit tests based on raw programming logic templates pasted by users.
- **088. Local Regulatory Compliance & Vulnerability Checker:** Token scanner identifying exposed credentials, hardcoded keys, or risky code strings before code commits.
- **089. Automated Dynamic Variable Renaming Engine:** Refactoring engine reviewing confusing code and suggesting clear variable names based on context.
- **090. Local Offline Multilingual Dictionary:** Translation dictionary converting error comments across international languages entirely offline.

### 10. Mock APIs & Testing Sandboxes

- **091. Local HTTP Response Engine Mock Server:** API prototyping engine spawning custom local endpoints with explicit status codes and JSON payloads.
- **092. High-Performance Parallel HTTP Load Tester:** Stress tester executing high-concurrency request loops against local servers to measure performance.
- **093. Native WebSocket Connection Laboratory:** Socket console managing connections, monitoring incoming events, and emitting text payloads to active socket systems.
- **094. GraphQL Query Builder & Schema Inspector:** Query assistant pulling schema endpoints and facilitating introspection testing within a decoupled UI.
- **095. gRPC Protocol Buffer Verification Deck:** API tool connecting to local gRPC systems, parsing proto files, and executing RPC testing methods.
- **096. System Environment Target Mock Matrix:** Diagnostic board setting custom runtime latency, network packet drop rates, and error environments.
- **097. REST API Collection Storage Bench:** Request engine managing query paths, auth structures, and header configurations.
- **098. Incoming Webhook Receiver Log Deck:** Diagnostic server catching, recording, and breaking down multi-source incoming webhook structures.
- **099. Status Code Reference Dictionary:** Searchable index tracking standard HTTP error/success specifications to speed up debugging.
- **100. Cookie & Session Parsing Playground:** Analyzer breaking down raw string cookies, exposing validation rules, expiry metrics, and security configurations.

---

## Technical Specification & Implementation Framework

### Core Technology Stack Selection

```
┌────────────────────────────────────────────────────────┐
│ UI Layer: Svelte 5 + TypeScript + Tailwind CSS         │
├────────────────────────────────────────────────────────┤
│ Bridge Layer: Tauri Core IPC Channels                 │
├────────────────────────────────────────────────────────┤
│ System Backend Layer: Rust (Tokio Async Runtime)       │
└────────────────────────────────────────────────────────┘
```

- **Frontend Core:** Svelte 5 chosen for its near-zero framework overhead, surgical reactivity updates, and minimal compilation output size.
- **Backend Architecture:** Rust leverages the `tokio` multi-threaded async runtime to maintain high responsiveness across parallel process kills, file watches, and log streaming.
- **Inter-Process Communication (IPC):** Structured JSON-RPC commands traveling via Tauri's low-latency binary serializing channel.

### Local Data Architecture & Persistence

- **Storage Strategy:** SQLite embedded engine running entirely in local files, configured with Write-Ahead Logging (WAL) enabled to allow sub-millisecond data writes.
- **Folder Conventions:** Absolute compliance with platform-specific standard directories:
  - **macOS:** `~/Library/Application Support/devforge`
  - **Linux:** `~/.config/devforge`
  - **Windows:** `%APPDATA%/devforge`
- **Data Encryption:** High-security storage sectors utilize AES-256-GCM authenticated encryption, with keys generated safely via the system OS keychain service (`security` on macOS, `Credential Manager` on Windows, `Secret Service` on Linux).

### Phased Engineering Milestones

```
Phase 1: Foundation (Weeks 1-4)    ──► Phase 2: Utilities (Weeks 5-8)   ──► Phase 3: Systems (Weeks 9-12)
├─ Core IPC Engine & Tray Agent         ├─ Data Transpilation Suite          ├─ Process & Port Liberator
├─ Fuzzy-Match Command Omnibar         ├─ Clipboard Stack Engine            ├─ Environment & Storage Audit
└─ Encryption & Storage Vault           └─ Frontend Design Toolkit           └─ Local AI & Mock Sandboxes
```

#### Phase 1: Foundation Setup & Omnibar Integration (Weeks 1–4)

- Establish the baseline Rust-Tauri architecture, configure cross-compilation toolchains, and verify memory footprint baselines.
- Build the low-latency system tray agent and verify global shortcut hooks across all targeted operating systems.
- Complete the fuzzy matching core algorithm and build the user interface for the primary launcher command bar overlay.
- Secure the platform keychain configurations and implement the local database schema layer.

#### Phase 2: Core Data Utilities & Clipboard Stack (Weeks 5–8)

- Implement data formatting, structural validation, and conversion systems for JSON, YAML, and XML payloads.
- Launch the background clipboard monitor and integrate automated data type classification engines.
- Write the cryptographic toolkit, including token generation, hashing modules, and JWT decoders.
- Finalize the design toolkit, including color swappers, aspect ratio calculators, and coordinate sanitizers.

#### Phase 3: System Automation, Local AI, & Testing Sandboxes (Weeks 9–12)

- Build the system process analysis grid and integrate atomic process termination functionalities safely.
- Create the local log streaming reader along with folder change monitoring utilities.
- Wire up the local HTTP mock framework and build websocket and load testing engines.
- Integrate the asynchronous Ollama API channel to enable offline LLM features securely.

---

## Strategic Evaluation Matrix

| Pillar Framework         | Primary Engineering Complexity   | Core Dependency Check           | System Resource Weight                |
| :----------------------- | :------------------------------- | :------------------------------ | :------------------------------------ |
| **01. Command Launcher** | High IPC Communication Frequency | `tauri-plugin-global-shortcut`  | Minimal (< 5MB RAM)                   |
| **02. Network & Ports**  | Host Permission Configurations   | Native System Calls (`sysinfo`) | Low (Transient CPU usage)             |
| **03. Data Utilities**   | High String Mutation Edge-Cases  | `serde_json`, `serde_yaml`      | Medium (Temporary memory peaks)       |
| **04. Cryptography**     | String Formatting and Encodings  | `ring`, `jsonwebtoken`          | Minimal (< 2MB RAM)                   |
| **05. Clipboard Stack**  | Multi-Threaded State Tracking    | `arboard`                       | Low to Medium (Grows with stack size) |
| **06. Snippet Vault**    | Fast Text Fuzzy Search Matching  | `regex`, `nucleo`               | Minimal (Indexed structures)          |
| **07. File System**      | Large IO Asset Array Processing  | `notify`, `tokio::fs`           | High (Scales with log sizes)          |
| **08. Frontend Design**  | Native Screen Pixel Access       | Native OS Screengrab APIs       | Medium (Active UI execution only)     |
| **09. Local AI Bridge**  | Asynchronous JSON Streaming      | `reqwest` (Localhost pipeline)  | Minimal (Heavy lifting by Ollama)     |
| **10. Mock Sandboxes**   | Dynamic Local Port Allocation    | `axum`, `tokio::net`            | Medium (Active server runtime)        |

---

## Definitive Validation Metrics & Quality Baselines

To transition from developmental stages into release candidates, the application must hit explicit performance baselines under test configurations:

1. **Launcher Latency Guarantee:** The maximum time allowed from pressing the global shortcut hotkey to the primary omnibar interface rendering completely on-screen must not exceed **16 milliseconds** (60 frames per second threshold).
2. **Fuzzy Search Execution Ceiling:** Querying an indexed database of 10,000 distinct snippets or historical commands must deliver ordered matching arrays in under **5 milliseconds**.
3. **Data Parsing Safety Margins:** Any single clipboard transformation payload containing large text structures (up to 50MB) must process successfully without freezing the main application thread or causing memory leaks.
4. **Log Streaming Efficiency:** Monitoring a continuous 5GB flat file log stream must utilize less than **15% total CPU capacity** and maintain a flat memory footprint below 60MB.
5. **Port Liberation Reliability:** Process termination commands directed at standard testing port pipelines must execute safely and accurately, without leaving orphaned background tasks behind.
