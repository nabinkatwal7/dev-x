import type { ConfigurationSnapshot } from "../types";

const encoder = new TextEncoder();
const decoder = new TextDecoder();

export interface EncryptedConfigFile {
  version: 1;
  kdf: "PBKDF2";
  iterations: number;
  salt: string;
  iv: string;
  cipher: string;
}

export async function encryptConfiguration(
  snapshot: ConfigurationSnapshot,
  passphrase: string
): Promise<string> {
  const trimmed = passphrase.trim();
  if (!trimmed) {
    throw new Error("Passphrase is required.");
  }

  const salt = crypto.getRandomValues(new Uint8Array(16));
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const key = await deriveKey(trimmed, salt);
  const payload = encoder.encode(JSON.stringify(snapshot));
  const cipherBuffer = await crypto.subtle.encrypt({ name: "AES-GCM", iv }, key, payload);

  const file: EncryptedConfigFile = {
    version: 1,
    kdf: "PBKDF2",
    iterations: 250000,
    salt: toBase64(salt),
    iv: toBase64(iv),
    cipher: toBase64(new Uint8Array(cipherBuffer))
  };

  return JSON.stringify(file, null, 2);
}

export async function decryptConfiguration(
  encryptedText: string,
  passphrase: string
): Promise<ConfigurationSnapshot> {
  const trimmed = passphrase.trim();
  if (!trimmed) {
    throw new Error("Passphrase is required.");
  }

  const file = JSON.parse(encryptedText) as EncryptedConfigFile;
  if (file.version !== 1 || file.kdf !== "PBKDF2") {
    throw new Error("Unsupported configuration file format.");
  }

  const salt = fromBase64(file.salt);
  const iv = fromBase64(file.iv);
  const cipher = fromBase64(file.cipher);
  const key = await deriveKey(trimmed, salt, file.iterations);
  const plaintext = await crypto.subtle.decrypt({ name: "AES-GCM", iv }, key, cipher);
  return JSON.parse(decoder.decode(plaintext)) as ConfigurationSnapshot;
}

async function deriveKey(passphrase: string, salt: Uint8Array, iterations = 250000) {
  const material = await crypto.subtle.importKey("raw", encoder.encode(passphrase), "PBKDF2", false, [
    "deriveKey"
  ]);

  return crypto.subtle.deriveKey(
    {
      name: "PBKDF2",
      salt: toArrayBuffer(salt),
      iterations,
      hash: "SHA-256"
    },
    material,
    {
      name: "AES-GCM",
      length: 256
    },
    false,
    ["encrypt", "decrypt"]
  );
}

function toArrayBuffer(bytes: Uint8Array) {
  return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer;
}

function toBase64(bytes: Uint8Array) {
  let output = "";
  for (const value of bytes) {
    output += String.fromCharCode(value);
  }
  return btoa(output);
}

function fromBase64(value: string) {
  const decoded = atob(value);
  const bytes = new Uint8Array(decoded.length);
  for (let index = 0; index < decoded.length; index += 1) {
    bytes[index] = decoded.charCodeAt(index);
  }
  return bytes;
}
