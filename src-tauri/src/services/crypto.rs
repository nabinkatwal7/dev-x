use crate::error::AppError;
use crate::models::{CommandExecutionResult, CommandExecutionStatus};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

// ============================================================
// 031 - Native Multi-Hash Compute Sandbox
// ============================================================
pub fn hash_compute(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "crypto.hash".into(),
            title: "Hash Compute".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste text to compute MD5, SHA-1, SHA-256, SHA-512, and CRC32 hashes.".into(),
        });
    }

    use md5::Md5;
    use sha1::Sha1;
    use sha2::{Sha256, Sha512, Digest};

    let md5 = format!("{:x}", Md5::digest(trimmed.as_bytes()));
    let sha1 = format!("{:x}", Sha1::digest(trimmed.as_bytes()));
    let sha256 = format!("{:x}", Sha256::digest(trimmed.as_bytes()));
    let sha512 = format!("{:x}", Sha512::digest(trimmed.as_bytes()));
    let crc32 = format!("{:08x}", crc32fast::hash(trimmed.as_bytes()));

    let output = format!(
        "MD5:      {}\nSHA-1:    {}\nSHA-256:  {}\nSHA-512:  {}\nCRC32:    {}",
        md5, sha1, sha256, sha512, crc32
    );

    Ok(CommandExecutionResult {
        command_id: "crypto.hash".into(),
        title: "Hash Results".into(),
        output,
        status: CommandExecutionStatus::Success,
        summary: "5 hashes computed".into(),
    })
}

// ============================================================
// 032 - Comprehensive JWT Inspection Deck
// ============================================================
pub fn jwt_inspect(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "crypto.jwt".into(),
            title: "JWT Inspect".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste a JWT token to decode its header, payload, and signature info.".into(),
        });
    }

    let parts: Vec<&str> = trimmed.split('.').collect();
    if parts.len() < 3 {
        return Ok(CommandExecutionResult {
            command_id: "crypto.jwt".into(),
            title: "Invalid JWT".into(),
            output: "The input does not appear to be a valid JWT (expected 3 dot-separated segments).".into(),
            status: CommandExecutionStatus::Error,
            summary: "Invalid token format".into(),
        });
    }

    let decode_b64 = |s: &str| -> String {
        let padded = match s.len() % 4 {
            0 => s.to_string(),
            r => format!("{}{}", s, "=".repeat(4 - r)),
        };
        use base64::Engine as _;
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        match engine.decode(&padded) {
            Ok(bytes) => String::from_utf8(bytes).unwrap_or_else(|_| s.to_string()),
            Err(_) => s.to_string(),
        }
    };

    let header = decode_b64(parts[0]);
    let payload = decode_b64(parts[1]);
    let signature = parts[2];

    let header_pretty = serde_json::from_str::<serde_json::Value>(&header)
        .map(|v| serde_json::to_string_pretty(&v).unwrap_or(header.clone()))
        .unwrap_or(header);
    let payload_pretty = serde_json::from_str::<serde_json::Value>(&payload)
        .map(|v| serde_json::to_string_pretty(&v).unwrap_or(payload.clone()))
        .unwrap_or(payload);

    let output = format!(
        "─── HEADER ───\n{}\n\n─── PAYLOAD ───\n{}\n\n─── SIGNATURE ───\n{} ({} chars)",
        header_pretty, payload_pretty, signature, signature.len()
    );

    Ok(CommandExecutionResult {
        command_id: "crypto.jwt".into(),
        title: "JWT Decoded".into(),
        output,
        status: CommandExecutionStatus::Success,
        summary: "Header + payload decoded".into(),
    })
}

// ============================================================
// 033 - Symmetric Cipher Laboratory
// ============================================================
pub fn symmetric_cipher(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "crypto.cipher".into(),
            title: "Symmetric Cipher".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Usage:\n  encrypt:aes256:<keyhex32>:<plaintext>\n  decrypt:aes256:<keyhex32>:<ct_hex>\n  encrypt:chacha:<keyhex32>:<plaintext>\n  decrypt:chacha:<keyhex32>:<ct_hex>\nKey is 32 bytes (64 hex chars). AES uses key[:16] as IV. ChaCha uses key[:12] as nonce.".into(),
        });
    }

    let parts: Vec<&str> = trimmed.splitn(4, ':').collect();
    if parts.len() < 4 {
        return Err(AppError::Internal("Usage: <encrypt/decrypt>:<aes256/chacha>:<keyhex>:<data>".into()));
    }

    let mode = parts[0].trim().to_lowercase();
    let algo = parts[1].trim().to_lowercase();
    let key_hex = parts[2].trim();
    let data = parts[3].trim();

    let key = hex::decode(key_hex)
        .map_err(|_| AppError::Internal("Invalid hex key".into()))?;
    if key.len() != 32 {
        return Err(AppError::Internal("Key must be 32 bytes (64 hex chars)".into()));
    }

    let output = match (mode.as_str(), algo.as_str()) {
        ("encrypt", "aes256") | ("encrypt", "aes") => {
            let iv = &key[..16];
            use aes::cipher::{KeyIvInit, BlockEncryptMut, block_padding::Pkcs7};
            type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
            let mut buf = data.as_bytes().to_vec();
            buf.resize(buf.len() + 64, 0);
            let ct = Aes256CbcEnc::new_from_slices(&key, iv)
                .map_err(|e| AppError::Internal(format!("Cipher init: {}", e)))?
                .encrypt_padded_mut::<Pkcs7>(&mut buf, data.len())
                .map_err(|e| AppError::Internal(format!("Encrypt failed: {}", e)))?;
            hex::encode(ct)
        }
        ("decrypt", "aes256") | ("decrypt", "aes") => {
            let iv = &key[..16];
            let ct = hex::decode(data)
                .map_err(|_| AppError::Internal("Ciphertext must be hex".into()))?;
            use aes::cipher::{KeyIvInit, BlockDecryptMut, block_padding::Pkcs7};
            type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
            let mut buf = ct;
            let pt = Aes256CbcDec::new_from_slices(&key, iv)
                .map_err(|e| AppError::Internal(format!("Cipher init: {}", e)))?
                .decrypt_padded_mut::<Pkcs7>(&mut buf)
                .map_err(|e| AppError::Internal(format!("Decrypt failed: {}", e)))?;
            String::from_utf8(pt.to_vec())
                .map_err(|_| AppError::Internal("Decrypted output is not valid UTF-8".into()))?
        }
        ("encrypt", "chacha") | ("encrypt", "chacha20") => {
            let nonce = &key[..12];
            use chacha20::cipher::{KeyIvInit, StreamCipher};
            use chacha20::ChaCha20;
            let mut buf = data.as_bytes().to_vec();
            let mut cipher = ChaCha20::new_from_slices(&key, nonce)
                .map_err(|e| AppError::Internal(format!("Cipher init: {}", e)))?;
            cipher.apply_keystream(&mut buf);
            hex::encode(&buf)
        }
        ("decrypt", "chacha") | ("decrypt", "chacha20") => {
            let nonce = &key[..12];
            let ct = hex::decode(data)
                .map_err(|_| AppError::Internal("Ciphertext must be hex".into()))?;
            use chacha20::cipher::{KeyIvInit, StreamCipher};
            use chacha20::ChaCha20;
            let mut buf = ct;
            let mut cipher = ChaCha20::new_from_slices(&key, nonce)
                .map_err(|e| AppError::Internal(format!("Cipher init: {}", e)))?;
            cipher.apply_keystream(&mut buf);
            String::from_utf8(buf)
                .map_err(|_| AppError::Internal("Decrypted output is not valid UTF-8".into()))?
        }
        _ => return Err(AppError::Internal("Mode must be encrypt/decrypt, algo must be aes256 or chacha".into())),
    };

    Ok(CommandExecutionResult {
        command_id: "crypto.cipher".into(),
        title: format!("{} ({})", mode, algo),
        output,
        status: CommandExecutionStatus::Success,
        summary: format!("{} with {} completed", mode, algo),
    })
}

// ============================================================
// 034 - Asymmetric RSA Keypair Engine
// ============================================================
pub fn rsa_keygen(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    let bits: usize = if trimmed.is_empty() { 2048 } else { trimmed.parse().unwrap_or(2048) };

    use rsa::RsaPrivateKey;
    use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
    use rand::rngs::OsRng;

    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, bits)
        .map_err(|e| AppError::Internal(format!("RSA key generation failed: {}", e)))?;

    let private_pem = private_key.to_pkcs8_pem(LineEnding::LF)
        .map_err(|e| AppError::Internal(format!("PEM encoding failed: {}", e)))?;
    let public_pem = private_key.to_public_key()
        .to_public_key_pem(LineEnding::LF)
        .map_err(|e| AppError::Internal(format!("Public key PEM encoding failed: {}", e)))?;

    let output = format!(
        "─── PRIVATE KEY ({} bits) ───\n{}\n─── PUBLIC KEY ───\n{}",
        bits, private_pem.as_str(), public_pem
    );

    Ok(CommandExecutionResult {
        command_id: "crypto.rsa-keygen".into(),
        title: format!("RSA {} Keypair", bits),
        output,
        status: CommandExecutionStatus::Success,
        summary: format!("{}-bit RSA keypair generated", bits),
    })
}

// ============================================================
// 035 - Base64 Binary/Text Converter Tool
// ============================================================
pub fn base64_convert(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "crypto.base64".into(),
            title: "Base64 Convert".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste text to encode, or 'decode:<base64>' to decode.\nUse 'decode:raw:<base64>' for raw decode.".into(),
        });
    }

    use base64::Engine as _;
    let engine = base64::engine::general_purpose::STANDARD;
    let url_engine = base64::engine::general_purpose::URL_SAFE;

    let output = if let Some(rest) = trimmed.strip_prefix("decode:") {
        let (use_raw, b64) = if let Some(raw) = rest.strip_prefix("raw:") { (true, raw.trim()) } else { (false, rest.trim()) };
        let bytes = engine.decode(b64)
            .or_else(|_| url_engine.decode(b64))
            .map_err(|e| AppError::Internal(format!("Invalid base64: {}", e)))?;
        if use_raw {
            format!("Decoded (raw, {} bytes):\n{}", bytes.len(), hex::encode(&bytes))
        } else {
            String::from_utf8(bytes.clone())
                .map(|s| format!("Decoded ({} bytes):\n{}", bytes.len(), s))
                .unwrap_or_else(|_| format!("Decoded ({} bytes, not UTF-8):\nHex: {}", bytes.len(), hex::encode(&bytes)))
        }
    } else {
        let encoded = engine.encode(trimmed.as_bytes());
        let url_encoded = url_engine.encode(trimmed.as_bytes());
        format!("Standard: {}\nURL-Safe: {}", encoded, url_encoded)
    };

    Ok(CommandExecutionResult {
        command_id: "crypto.base64".into(),
        title: "Base64 Result".into(),
        output,
        status: CommandExecutionStatus::Success,
        summary: "Base64 conversion complete".into(),
    })
}

// ============================================================
// 036 - Variable Hashing Benchmark Lab
// ============================================================
pub fn hash_benchmark(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "crypto.hash-bench".into(),
            title: "Hash Benchmark".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Format:\nalgo:bcrypt|argon2|sha256\niterations:5\ncost:10\ntext:benchmark me".into(),
        });
    }

    let mut algo = String::from("bcrypt");
    let mut iterations = 3usize;
    let mut cost = 10u32;
    let mut text = String::new();

    if trimmed.contains('\n') {
        for line in trimmed.lines() {
            if let Some(value) = line.strip_prefix("algo:") {
                algo = value.trim().to_lowercase();
            } else if let Some(value) = line.strip_prefix("iterations:") {
                iterations = value.trim().parse::<usize>().unwrap_or(3).clamp(1, 50);
            } else if let Some(value) = line.strip_prefix("cost:") {
                cost = value.trim().parse::<u32>().unwrap_or(10).clamp(4, 15);
            } else if let Some(value) = line.strip_prefix("text:") {
                text = value.to_string();
            }
        }
    } else if let Some(pos) = trimmed.find(':') {
        algo = trimmed[..pos].trim().to_lowercase();
        text = trimmed[pos + 1..].trim().to_string();
    } else {
        text = trimmed.to_string();
    }

    if text.is_empty() {
        return Err(AppError::Internal("benchmark text cannot be empty".into()));
    }

    let output = match algo.as_str() {
        "bcrypt" => {
            let mut timings = Vec::new();
            let mut hash = String::new();
            for _ in 0..iterations {
                let start = Instant::now();
                hash = bcrypt::hash(&text, cost)
                    .map_err(|e| AppError::Internal(format!("bcrypt failed: {}", e)))?;
                timings.push(start.elapsed().as_millis() as u128);
            }
            let valid = bcrypt::verify(&text, &hash).unwrap_or(false);
            format_benchmark_output("bcrypt", iterations, &timings, &hash, Some(format!("verify={}", valid)))
        }
        "argon2" | "argon2id" => {
            use argon2::Argon2;
            use argon2::password_hash::{SaltString, PasswordHasher};
            let mut timings = Vec::new();
            let mut hash = String::new();
            for _ in 0..iterations {
                let salt = SaltString::generate(&mut rand::thread_rng());
                let start = Instant::now();
                hash = Argon2::default().hash_password(text.as_bytes(), &salt)
                    .map_err(|e| AppError::Internal(format!("Argon2 failed: {}", e)))?
                    .to_string();
                timings.push(start.elapsed().as_millis() as u128);
            }
            format_benchmark_output("argon2id", iterations, &timings, &hash, None)
        }
        "sha256" => {
            use sha2::{Digest, Sha256};
            let mut timings = Vec::new();
            let mut hash = String::new();
            for _ in 0..iterations {
                let start = Instant::now();
                hash = format!("{:x}", Sha256::digest(text.as_bytes()));
                timings.push(start.elapsed().as_micros() as u128);
            }
            format_benchmark_output("sha256", iterations, &timings, &hash, Some("unit=us".into()))
        }
        _ => return Err(AppError::Internal(format!("unknown algorithm '{}'. Supported: bcrypt, argon2, sha256", algo))),
    };

    Ok(CommandExecutionResult {
        command_id: "crypto.hash-bench".into(),
        title: format!("{} Benchmark", algo.to_uppercase()),
        output,
        status: CommandExecutionStatus::Success,
        summary: format!("{} benchmark completed over {} iteration(s)", algo, iterations),
    })
}

fn format_benchmark_output(
    algo: &str,
    iterations: usize,
    timings: &[u128],
    sample_hash: &str,
    extra: Option<String>,
) -> String {
    let min = timings.iter().min().copied().unwrap_or(0);
    let max = timings.iter().max().copied().unwrap_or(0);
    let avg = if timings.is_empty() {
        0.0
    } else {
        timings.iter().sum::<u128>() as f64 / timings.len() as f64
    };
    let unit = if matches!(algo, "sha256") { "us" } else { "ms" };
    let mut output = format!(
        "Algorithm: {}\nIterations: {}\nMin: {} {}\nAvg: {:.2} {}\nMax: {} {}\nSample hash:\n{}",
        algo, iterations, min, unit, avg, unit, max, unit, sample_hash
    );
    if let Some(extra) = extra {
        output.push_str(&format!("\n{}", extra));
    }
    output
}

// ============================================================
// 037 - HTML Entity & Hex Decoder
// ============================================================
pub fn html_decode(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "crypto.html-decode".into(),
            title: "HTML Decode".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Paste HTML-encoded text or hex escape sequences to decode.".into(),
        });
    }

    let mut output = trimmed
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&#x60;", "`")
        .replace("&#x2F;", "/");

    let re = regex::Regex::new(r"&#(x?)([0-9a-fA-F]+);").unwrap();
    output = re.replace_all(&output, |caps: &regex::Captures| {
        let is_hex = !caps[1].is_empty();
        if let Ok(code) = if is_hex { u32::from_str_radix(&caps[2], 16) } else { caps[2].parse::<u32>() } {
            if let Some(ch) = char::from_u32(code) {
                return ch.to_string();
            }
        }
        caps[0].to_string()
    }).to_string();

    let re_hex = regex::Regex::new(r"\\x([0-9a-fA-F]{2})").unwrap();
    output = re_hex.replace_all(&output, |caps: &regex::Captures| {
        u8::from_str_radix(&caps[1], 16)
            .map(|b| (b as char).to_string())
            .unwrap_or_else(|_| caps[0].to_string())
    }).to_string();

    let re_pct = regex::Regex::new(r"%([0-9a-fA-F]{2})").unwrap();
    output = re_pct.replace_all(&output, |caps: &regex::Captures| {
        u8::from_str_radix(&caps[1], 16)
            .map(|b| (b as char).to_string())
            .unwrap_or_else(|_| caps[0].to_string())
    }).to_string();

    Ok(CommandExecutionResult {
        command_id: "crypto.html-decode".into(),
        title: "Decoded Output".into(),
        output,
        status: CommandExecutionStatus::Success,
        summary: "HTML entities and hex sequences decoded".into(),
    })
}

// ============================================================
// 038 - Cryptographically Secure Token Generator
// ============================================================
pub fn token_generate(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim().to_lowercase();

    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "crypto.gen-token".into(),
            title: "Token Generator".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Usage: uuid | uuid7 | hex <bytes> | password <len> | pin <len> [count]\nDefault: uuid".into(),
        });
    }

    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    let mode = tokens[0];
    let count = tokens.get(1).and_then(|s| s.parse().ok()).unwrap_or(1usize).min(20);

    use rand::Rng;

    let mut output = String::new();

    match mode {
        "uuid" | "uuid4" => {
            for _ in 0..count {
                output.push_str(&format!("{}\n", uuid::Uuid::new_v4()));
            }
        }
        "uuid7" => {
            for _ in 0..count {
                output.push_str(&format!("{}\n", uuid::Uuid::now_v7()));
            }
        }
        "hex" | "token" => {
            let len = tokens.get(1).and_then(|s| s.parse().ok()).unwrap_or(32usize).max(8).min(128);
            let count_actual = tokens.get(2).and_then(|s| s.parse().ok()).unwrap_or(1).min(20);
            for _ in 0..count_actual {
                let bytes: Vec<u8> = (0..len).map(|_| rand::thread_rng().gen()).collect();
                output.push_str(&format!("{}\n", hex::encode(&bytes)));
            }
        }
        "password" | "pass" => {
            let len = tokens.get(1).and_then(|s| s.parse().ok()).unwrap_or(24usize).max(8).min(128);
            let count_actual = tokens.get(2).and_then(|s| s.parse().ok()).unwrap_or(1).min(20);
            let charset: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789!@#$%^&*()_+-=".chars().collect();
            for _ in 0..count_actual {
                let pwd: String = (0..len).map(|_| charset[rand::thread_rng().gen_range(0..charset.len())]).collect();
                output.push_str(&format!("{}\n", pwd));
            }
        }
        "numeric" | "pin" => {
            let len = tokens.get(1).and_then(|s| s.parse().ok()).unwrap_or(6usize).max(4).min(32);
            let count_actual = tokens.get(2).and_then(|s| s.parse().ok()).unwrap_or(1).min(20);
            for _ in 0..count_actual {
                let pin: String = (0..len).map(|_| rand::thread_rng().gen_range(0..10).to_string()).collect();
                output.push_str(&format!("{}\n", pin));
            }
        }
        _ => {
            return Ok(CommandExecutionResult {
                command_id: "crypto.gen-token".into(),
                title: "Token Generator".into(),
                output: String::new(),
                status: CommandExecutionStatus::Info,
                summary: "Usage: uuid | uuid7 | hex <bytes> | password <len> | numeric <len> [count]".into(),
            });
        }
    }

    Ok(CommandExecutionResult {
        command_id: "crypto.gen-token".into(),
        title: format!("Generated Tokens ({})", mode),
        output: output.trim().to_string(),
        status: CommandExecutionStatus::Success,
        summary: format!("{} token(s) generated via {}", count, mode),
    })
}

// ============================================================
// 039 - HMAC Signature Computational Matrix
// ============================================================
pub fn hmac_compute(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(CommandExecutionResult {
            command_id: "crypto.hmac".into(),
            title: "HMAC Compute".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Format: <key> | <message>\nFirst line is the secret key, rest is the message.".into(),
        });
    }

    let parts: Vec<&str> = trimmed.splitn(2, '\n').collect();
    if parts.len() < 2 {
        return Ok(CommandExecutionResult {
            command_id: "crypto.hmac".into(),
            title: "HMAC Compute".into(),
            output: String::new(),
            status: CommandExecutionStatus::Info,
            summary: "Provide key on first line, message on second line.".into(),
        });
    }

    let key_bytes = parts[0].trim().as_bytes();
    let message = parts[1].trim().as_bytes();

    use hmac::{Hmac, Mac, digest::FixedOutput};
    use sha2::{Sha256, Sha512};

    let mut mac256 = Hmac::<Sha256>::new_from_slice(key_bytes)
        .map_err(|e| AppError::Internal(format!("HMAC init: {}", e)))?;
    mac256.update(message);
    let result256 = hex::encode(mac256.finalize_fixed());

    let mut mac512 = Hmac::<Sha512>::new_from_slice(key_bytes)
        .map_err(|e| AppError::Internal(format!("HMAC init: {}", e)))?;
    mac512.update(message);
    let result512 = hex::encode(mac512.finalize_fixed());

    let output = format!(
        "HMAC-SHA256: {}\nHMAC-SHA512: {}",
        result256, result512
    );

    Ok(CommandExecutionResult {
        command_id: "crypto.hmac".into(),
        title: "HMAC Results".into(),
        output,
        status: CommandExecutionStatus::Success,
        summary: "HMAC-SHA256 and HMAC-SHA512 computed".into(),
    })
}

// ============================================================
// 040 - Local Cryptographic Vault Sandbox
// ============================================================

lazy_static::lazy_static! {
    static ref VAULT: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

pub fn crypto_vault(input: &str) -> Result<CommandExecutionResult, AppError> {
    load_vault_state()?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        let vault = VAULT.lock().unwrap();
        if vault.is_empty() {
            return Ok(CommandExecutionResult {
                command_id: "crypto.vault".into(),
                title: "Crypto Vault".into(),
                output: String::new(),
                status: CommandExecutionStatus::Info,
                summary: "Commands: set:<label>:<value> | get:<label> | delete:<label> | list | clear".into(),
            });
        }
        let mut output = String::from("Stored entries:\n");
        for (key, value) in vault.iter() {
            output.push_str(&format!("  {} => {} chars\n", key, value.len()));
        }
        return Ok(CommandExecutionResult {
            command_id: "crypto.vault".into(),
            title: "Vault Contents".into(),
            output,
            status: CommandExecutionStatus::Success,
            summary: format!("{} entries in vault", vault.len()),
        });
    }

    let (cmd, rest) = if let Some(pos) = trimmed.find(':') {
        (trimmed[..pos].trim().to_lowercase(), trimmed[pos + 1..].trim())
    } else {
        (trimmed.to_lowercase(), "")
    };

    match cmd.as_str() {
        "set" => {
            if let Some(pos) = rest.find(':') {
                let label = rest[..pos].trim();
                let value = rest[pos + 1..].trim();
                let mut vault = VAULT.lock().unwrap();
                vault.insert(label.to_string(), value.to_string());
                save_vault_state(&vault)?;
                Ok(CommandExecutionResult {
                    command_id: "crypto.vault".into(),
                    title: "Vault Set".into(),
                    output: format!("Stored '{}' ({} chars)", label, value.len()),
                    status: CommandExecutionStatus::Success,
                    summary: format!("'{}' saved to vault", label),
                })
            } else {
                Err(AppError::Internal("Format: set:<label>:<value>".into()))
            }
        }
        "get" => {
            let vault = VAULT.lock().unwrap();
            match vault.get(rest) {
                Some(value) => Ok(CommandExecutionResult {
                    command_id: "crypto.vault".into(),
                    title: format!("Vault: {}", rest),
                    output: value.clone(),
                    status: CommandExecutionStatus::Success,
                    summary: format!("Retrieved '{}'", rest),
                }),
                None => Ok(CommandExecutionResult {
                    command_id: "crypto.vault".into(),
                    title: "Not Found".into(),
                    output: format!("No entry '{}' in vault", rest),
                    status: CommandExecutionStatus::Error,
                    summary: "Entry not found".into(),
                }),
            }
        }
        "delete" | "del" | "remove" => {
            let mut vault = VAULT.lock().unwrap();
            if vault.remove(rest).is_some() {
                save_vault_state(&vault)?;
                Ok(CommandExecutionResult {
                    command_id: "crypto.vault".into(),
                    title: "Vault Delete".into(),
                    output: format!("Deleted '{}'", rest),
                    status: CommandExecutionStatus::Success,
                    summary: format!("'{}' removed from vault", rest),
                })
            } else {
                Ok(CommandExecutionResult {
                    command_id: "crypto.vault".into(),
                    title: "Not Found".into(),
                    output: format!("No entry '{}' in vault", rest),
                    status: CommandExecutionStatus::Error,
                    summary: "Entry not found".into(),
                })
            }
        }
        "list" | "ls" => {
            let vault = VAULT.lock().unwrap();
            if vault.is_empty() {
                Ok(CommandExecutionResult {
                    command_id: "crypto.vault".into(),
                    title: "Vault Empty".into(),
                    output: "No entries stored.".into(),
                    status: CommandExecutionStatus::Info,
                    summary: "Vault is empty".into(),
                })
            } else {
                let mut output = String::from("Vault entries:\n");
                for key in vault.keys() {
                    output.push_str(&format!("  - {}\n", key));
                }
                Ok(CommandExecutionResult {
                    command_id: "crypto.vault".into(),
                    title: "Vault List".into(),
                    output,
                    status: CommandExecutionStatus::Success,
                    summary: format!("{} entries", vault.len()),
                })
            }
        }
        "clear" => {
            let mut vault = VAULT.lock().unwrap();
            let count = vault.len();
            vault.clear();
            save_vault_state(&vault)?;
            Ok(CommandExecutionResult {
                command_id: "crypto.vault".into(),
                title: "Vault Cleared".into(),
                output: format!("Cleared {} entries.", count),
                status: CommandExecutionStatus::Success,
                summary: "Vault cleared".into(),
            })
        }
        _ => Ok(CommandExecutionResult {
            command_id: "crypto.vault".into(),
            title: "Unknown Command".into(),
            output: "Commands: set:<label>:<value> | get:<label> | delete:<label> | list | clear".into(),
            status: CommandExecutionStatus::Info,
            summary: "See output for usage".into(),
        }),
    }
}

fn vault_file_path() -> PathBuf {
    std::env::temp_dir().join("devforge-crypto-vault.json")
}

fn load_vault_state() -> Result<(), AppError> {
    let path = vault_file_path();
    if !path.exists() {
        return Ok(());
    }
    let contents = fs::read_to_string(&path)
        .map_err(|error| AppError::Internal(format!("failed to read vault file {}: {}", path.display(), error)))?;
    let parsed: HashMap<String, String> = serde_json::from_str(&contents)
        .map_err(|error| AppError::Internal(format!("failed to parse vault file {}: {}", path.display(), error)))?;
    let mut vault = VAULT.lock().unwrap();
    *vault = parsed;
    Ok(())
}

fn save_vault_state(vault: &HashMap<String, String>) -> Result<(), AppError> {
    let path = vault_file_path();
    let contents = serde_json::to_string_pretty(vault)
        .map_err(|error| AppError::Internal(format!("failed to serialize vault state: {}", error)))?;
    fs::write(&path, contents)
        .map_err(|error| AppError::Internal(format!("failed to write vault file {}: {}", path.display(), error)))
}
