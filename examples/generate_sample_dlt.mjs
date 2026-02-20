#!/usr/bin/env node
// Generate a sample.dlt file using the WASM module.
// Mirrors examples/generate_sample_dlt.rs behaviour.
//
// Usage:
//   node examples/generate_sample_dlt.mjs
//   node examples/generate_sample_dlt.mjs my_output.dlt
//
// Output: sample.dlt (or the path given as first argument)
//
// Build WASM first: ./build-wasm.sh

import { readFileSync, writeFileSync } from 'fs';

// ─── Load WASM ───────────────────────────────────────────────────────────────

const wasmPath = 'target/wasm32-unknown-unknown/release/examples/wasm_demo.wasm';
const { instance } = await WebAssembly.instantiate(readFileSync(wasmPath), {});
const wasm = instance.exports;

// ─── Helper utilities ────────────────────────────────────────────────────────

/** Encode a 4-char ASCII ID into a Uint8Array(4), null-padded. */
function encodeId(str) {
    const out = new Uint8Array(4);
    for (let i = 0; i < 4 && i < str.length; i++) out[i] = str.charCodeAt(i);
    return out;
}

/** Current Unix time as { seconds, microseconds }. */
function nowUnix() {
    const ms = Date.now();
    return { seconds: Math.floor(ms / 1000), microseconds: (ms % 1000) * 1000 };
}

/**
 * Build the 24-byte config block for generate_log_message.
 *
 * Config layout:
 *   [0..4]   ecu_id
 *   [4..8]   app_id
 *   [8..12]  ctx_id
 *   [12]     log_level  1=Fatal 2=Error 3=Warn 4=Info 5=Debug 6=Verbose
 *   [13]     verbose    0|1
 *   [14]     noar       number of arguments
 *   [15]     file_header 1 = prepend 16-byte storage header
 *   [16..20] timestamp  u32 LE (0.1 ms units)
 *   [20..24] reserved
 */
function buildConfig({ ecuId, appId, ctxId, logLevel, verbose, noar, timestamp }) {
    const cfg = new Uint8Array(24);
    cfg.set(encodeId(ecuId), 0);
    cfg.set(encodeId(appId), 4);
    cfg.set(encodeId(ctxId), 8);
    cfg[12] = logLevel;
    cfg[13] = verbose ? 1 : 0;
    cfg[14] = noar ?? 1;
    cfg[15] = 1; // always write storage header for .dlt files
    // timestamp as u32 LE
    const ts = timestamp ?? 0;
    cfg[16] = ts & 0xFF;
    cfg[17] = (ts >> 8) & 0xFF;
    cfg[18] = (ts >> 16) & 0xFF;
    cfg[19] = (ts >> 24) & 0xFF;
    return cfg;
}

/**
 * Call wasm.generate_log_message and return the resulting bytes.
 * The storage header seconds/microseconds are patched with the real wall-clock time.
 */
function generateMessage(spec, index) {
    const enc = new TextEncoder();
    const payBytes = enc.encode(spec.payload);

    const cfg = buildConfig({
        ecuId: 'ECU1',
        appId: spec.appId,
        ctxId: spec.ctxId,
        logLevel: spec.logLevel,
        verbose: spec.verbose ?? true,
        noar: 1,
        timestamp: index * 100, // 0.1 ms units → 10 ms apart
    });

    const cfgPtr = wasm.allocate(24);
    const payPtr = wasm.allocate(payBytes.length || 1);
    const outPtr = wasm.allocate(512);

    if (!cfgPtr || !payPtr || !outPtr) throw new Error('WASM heap exhausted');

    new Uint8Array(wasm.memory.buffer, cfgPtr, 24).set(cfg);
    if (payBytes.length) new Uint8Array(wasm.memory.buffer, payPtr, payBytes.length).set(payBytes);

    const size = wasm.generate_log_message(cfgPtr, payPtr, payBytes.length, outPtr, 512);

    wasm.deallocate(cfgPtr);
    wasm.deallocate(payPtr);

    if (size <= 0) {
        wasm.deallocate(outPtr);
        throw new Error(`generate_log_message failed: error code ${size}`);
    }

    // Copy out before deallocating
    const bytes = new Uint8Array(wasm.memory.buffer, outPtr, size).slice();
    wasm.deallocate(outPtr);

    // Patch storage header: [4..8] = seconds LE, [8..12] = microseconds LE
    // Each message is 1 second later so timestamps are ordered.
    const { seconds, microseconds } = nowUnix();
    const dv = new DataView(bytes.buffer);
    dv.setUint32(4, seconds + index, true);
    dv.setUint32(8, microseconds, true);

    return bytes;
}

// ─── Message definitions (mirrors generate_sample_dlt.rs) ────────────────────

const LOG_LEVEL_NAMES = ['?', 'FATAL', 'ERROR', 'WARN', 'INFO', 'DEBUG', 'VERBOSE'];

const messages = [
    { appId: 'INIT', ctxId: 'BOOT', logLevel: 4, payload: 'System initializing', verbose: true },
    { appId: 'INIT', ctxId: 'BOOT', logLevel: 4, payload: 'Loading configuration', verbose: true },
    { appId: 'NET', ctxId: 'CONN', logLevel: 5, payload: 'Connecting to remote host 192.168.1.100', verbose: true },
    { appId: 'NET', ctxId: 'CONN', logLevel: 4, payload: 'Connection established', verbose: true },
    { appId: 'APP', ctxId: 'MAIN', logLevel: 4, payload: 'Application started successfully', verbose: true },
    { appId: 'APP', ctxId: 'DATA', logLevel: 5, payload: 'Processing data batch: 1024 records', verbose: true },
    { appId: 'APP', ctxId: 'DATA', logLevel: 3, payload: 'Memory usage above 80%', verbose: true },
    { appId: 'NET', ctxId: 'CONN', logLevel: 2, payload: 'Packet loss detected: 3 retransmissions', verbose: true },
    { appId: 'APP', ctxId: 'MAIN', logLevel: 6, payload: 'Heartbeat tick #100', verbose: false },
    { appId: 'APP', ctxId: 'MAIN', logLevel: 1, payload: 'Unexpected shutdown requested', verbose: true },
];

// ─── Generate and write ───────────────────────────────────────────────────────

const outPath = process.argv[2] ?? 'sample.dlt';
const chunks = [];
let totalBytes = 0;

for (let i = 0; i < messages.length; i++) {
    const spec = messages[i];
    const bytes = generateMessage(spec, i);
    chunks.push(bytes);
    totalBytes += bytes.length;

    const lvlName = LOG_LEVEL_NAMES[spec.logLevel] ?? '?';
    const mode = spec.verbose ? 'verbose' : 'non-verbose';
    console.log(
        `  [${String(i + 1).padStart(2, '0')}] ECU1/${spec.appId}/${spec.ctxId}` +
        ` [${lvlName}] (${mode}) "${spec.payload}" (${bytes.length} bytes)`
    );
}

// Concatenate all message buffers into one file
const fileBuffer = Buffer.concat(chunks.map(u8 => Buffer.from(u8)));
writeFileSync(outPath, fileBuffer);

console.log(`\nWrote ${messages.length} messages (${totalBytes} bytes total) to ${outPath}`);
