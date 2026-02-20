#!/usr/bin/env node
/**
 * DLT TCP Client - Node.js
 *
 * Connects to dlt_tcp_server on localhost:3490 and parses incoming DLT messages
 * using the WASM module built from wasm_demo.rs.
 *
 * Usage:
 *   node examples/dlt_tcp_client.mjs [host] [port]
 *
 * Example:
 *   node examples/dlt_tcp_client.mjs
 *   node examples/dlt_tcp_client.mjs localhost 3490
 */

import { readFileSync } from 'fs';
import { createConnection } from 'net';
import { fileURLToPath } from 'url';
import { dirname, resolve } from 'path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const WASM_PATH = resolve(__dirname, '../target/wasm32-unknown-unknown/release/examples/wasm_demo.wasm');

const HOST = process.argv[2] ?? 'localhost';
const PORT = parseInt(process.argv[3] ?? '3490', 10);

// ─── WASM helpers ─────────────────────────────────────────────────────────────

const LOG_LEVEL_NAMES = ['UNKNOWN', 'FATAL', 'ERROR', 'WARN', 'INFO', 'DEBUG', 'VERBOSE'];

function id4(mem, offset) {
    const bytes = new Uint8Array(mem.buffer, offset, 4);
    return String.fromCharCode(...bytes).replace(/\0/g, '').trim() || '----';
}

/**
 * Parse a single DLT message from `data` at `offset` using the WASM API.
 * Returns { parsed, consumed } or null if the message is incomplete.
 */
function parseDltMessage(wasm, data, offset) {
    const mem = wasm.exports.memory;
    const remaining = data.length - offset;
    if (remaining < 4) return null;

    // ── Detect optional headers to find standard-header start ─────────────────
    let stdOffset = offset;
    let hasFileHeader = false;
    let hasSerialHeader = false;

    // File header: "DLT\x01"
    if (data[offset] === 0x44 && data[offset + 1] === 0x4C &&
        data[offset + 2] === 0x54 && data[offset + 3] === 0x01) {
        hasFileHeader = true;
        stdOffset += 4;
    }
    // Serial header: "DLS\x01"
    if (remaining >= (stdOffset - offset) + 4 &&
        data[stdOffset] === 0x44 && data[stdOffset + 1] === 0x4C &&
        data[stdOffset + 2] === 0x53 && data[stdOffset + 3] === 0x01) {
        hasSerialHeader = true;
        stdOffset += 4;
    }

    if (data.length - stdOffset < 4) return null;

    // Length is at bytes 2-3 of standard header (big-endian)
    const msgLen = (data[stdOffset + 2] << 8) | data[stdOffset + 3];
    const totalLen = (stdOffset - offset) + msgLen; // prefixes + standard header payload

    if (remaining < totalLen) return null; // wait for more data

    // Copy the message bytes into WASM memory
    const ptr = wasm.exports.allocate(totalLen);
    if (!ptr) return null;
    const slice = new Uint8Array(mem.buffer, ptr, totalLen);
    slice.set(data.subarray(offset, offset + totalLen));

    // Call analyze_dlt_message
    const resultPtr = wasm.exports.analyze_dlt_message(ptr, totalLen);

    let parsed = null;
    if (resultPtr) {
        const r = new DataView(mem.buffer, resultPtr, 32);
        const logLevel = r.getUint8(9);
        const ecuId = id4(mem, resultPtr + 12);
        const appId = id4(mem, resultPtr + 16);
        const ctxId = id4(mem, resultPtr + 20);
        const isVerbose = r.getUint8(25);

        // Try to decode the payload as a UTF-8 string for display
        const payloadOffset = r.getUint16(6, true); // little-endian
        const payloadLen = r.getUint16(4, true);
        let payloadText = '';
        if (payloadLen > 0 && payloadOffset + payloadLen <= totalLen) {
            const payloadBytes = data.subarray(offset + payloadOffset, offset + payloadOffset + payloadLen);
            // Non-verbose: raw bytes as UTF-8; verbose: skip 4-byte type-info per argument
            if (isVerbose && payloadLen > 4) {
                // Skip the 4-byte type-info of the first argument
                payloadText = Buffer.from(payloadBytes.subarray(4)).toString('utf8').replace(/\0/g, '');
            } else {
                payloadText = Buffer.from(payloadBytes).toString('utf8').replace(/\0/g, '');
            }
        }

        parsed = {
            ecuId,
            appId,
            ctxId,
            logLevel,
            logLevelName: LOG_LEVEL_NAMES[logLevel] ?? 'UNKNOWN',
            isVerbose: !!isVerbose,
            hasSerialHeader,
            hasFileHeader,
            payloadText,
            totalLen,
        };

        wasm.exports.deallocate(resultPtr);
    }

    wasm.exports.deallocate(ptr);
    return { parsed, consumed: totalLen };
}

// ─── Main ────────────────────────────────────────────────────────────────────

async function main() {
    console.log(`Loading WASM from:\n  ${WASM_PATH}\n`);
    const wasmBytes = readFileSync(WASM_PATH);
    const { instance } = await WebAssembly.instantiate(wasmBytes, {});
    const wasm = instance;
    console.log('WASM loaded ✅\n');

    let rxBuf = Buffer.alloc(0); // reassembly buffer for partial TCP reads
    let msgCount = 0;

    console.log(`Connecting to DLT TCP server at ${HOST}:${PORT} …\n`);

    const socket = createConnection({ host: HOST, port: PORT }, () => {
        console.log(`Connected to ${HOST}:${PORT} ✅\n`);
    });

    socket.on('data', (chunk) => {
        // Append to reassembly buffer
        rxBuf = Buffer.concat([rxBuf, chunk]);

        let offset = 0;
        while (offset < rxBuf.length) {
            const result = parseDltMessage(wasm, rxBuf, offset);
            if (!result) break; // not enough data yet

            if (result.parsed) {
                msgCount++;
                const { ecuId, appId, ctxId, logLevel, logLevelName, isVerbose, payloadText } = result.parsed;
                console.log(
                    `[${String(msgCount).padStart(4, '0')}] ` +
                    `ECU:${ecuId} APP:${appId} CTX:${ctxId} ` +
                    `[${logLevelName.padEnd(7)}] ` +
                    `${isVerbose ? '(verbose) ' : ''}` +
                    `"${payloadText}"`
                );
            }

            offset += result.consumed;
        }

        // Keep only unprocessed bytes
        rxBuf = rxBuf.subarray(offset);
    });

    socket.on('end', () => {
        console.log(`\nServer closed the connection. Total messages received: ${msgCount}`);
    });

    socket.on('error', (err) => {
        console.error(`\nSocket error: ${err.message}`);
        process.exit(1);
    });

    process.on('SIGINT', () => {
        console.log(`\nInterrupted. Total messages received: ${msgCount}`);
        socket.destroy();
        process.exit(0);
    });
}

main().catch((err) => {
    console.error(err);
    process.exit(1);
});
