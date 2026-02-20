// Generate a sample.dlt file containing DLT log messages with 16-byte storage headers.
//
// Each message in a .dlt file is prefixed with:
//   [0..4]   magic "DLT\x01"
//   [4..8]   seconds (u32 LE) — Unix epoch
//   [8..12]  microseconds (u32 LE)
//   [12..16] ECU ID (4 ASCII bytes)
// followed by the standard DLT message (standard + extra + extended headers + payload).
//
// Usage: cargo run --example generate_sample_dlt
// Output: sample.dlt (in the current working directory)

use dlt_protocol::r19_11::{DltMessageBuilder, MtinTypeDltLog};
use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get the current time as (seconds, microseconds) since Unix epoch.
fn now_unix() -> (u32, u32) {
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    (d.as_secs() as u32, d.subsec_micros())
}

/// Patch the storage header timestamp at the start of a generated message buffer.
/// The message must have been generated with `.add_file_header()`.
fn patch_storage_timestamp(buf: &mut [u8], seconds: u32, microseconds: u32) {
    // bytes [4..8]  = seconds LE
    buf[4..8].copy_from_slice(&seconds.to_le_bytes());
    // bytes [8..12] = microseconds LE
    buf[8..12].copy_from_slice(&microseconds.to_le_bytes());
}

struct MessageSpec {
    app_id: &'static [u8; 4],
    ctx_id: &'static [u8; 4],
    level: MtinTypeDltLog,
    payload: &'static str,
    verbose: bool,
}

fn main() -> std::io::Result<()> {
    let messages: &[MessageSpec] = &[
        MessageSpec {
            app_id: b"INIT",
            ctx_id: b"BOOT",
            level: MtinTypeDltLog::DltLogInfo,
            payload: "System initializing",
            verbose: true,
        },
        MessageSpec {
            app_id: b"INIT",
            ctx_id: b"BOOT",
            level: MtinTypeDltLog::DltLogInfo,
            payload: "Loading configuration",
            verbose: true,
        },
        MessageSpec {
            app_id: b"NET\0",
            ctx_id: b"CONN",
            level: MtinTypeDltLog::DltLogDebug,
            payload: "Connecting to remote host 192.168.1.100",
            verbose: true,
        },
        MessageSpec {
            app_id: b"NET\0",
            ctx_id: b"CONN",
            level: MtinTypeDltLog::DltLogInfo,
            payload: "Connection established",
            verbose: true,
        },
        MessageSpec {
            app_id: b"APP\0",
            ctx_id: b"MAIN",
            level: MtinTypeDltLog::DltLogInfo,
            payload: "Application started successfully",
            verbose: true,
        },
        MessageSpec {
            app_id: b"APP\0",
            ctx_id: b"DATA",
            level: MtinTypeDltLog::DltLogDebug,
            payload: "Processing data batch: 1024 records",
            verbose: true,
        },
        MessageSpec {
            app_id: b"APP\0",
            ctx_id: b"DATA",
            level: MtinTypeDltLog::DltLogWarn,
            payload: "Memory usage above 80%",
            verbose: true,
        },
        MessageSpec {
            app_id: b"NET\0",
            ctx_id: b"CONN",
            level: MtinTypeDltLog::DltLogError,
            payload: "Packet loss detected: 3 retransmissions",
            verbose: true,
        },
        MessageSpec {
            app_id: b"APP\0",
            ctx_id: b"MAIN",
            level: MtinTypeDltLog::DltLogVerbose,
            payload: "Heartbeat tick #100",
            verbose: false, // non-verbose (raw payload)
        },
        MessageSpec {
            app_id: b"APP\0",
            ctx_id: b"MAIN",
            level: MtinTypeDltLog::DltLogFatal,
            payload: "Unexpected shutdown requested",
            verbose: true,
        },
    ];

    let mut file = File::create("sample.dlt")?;
    let mut total_bytes = 0usize;

    for (i, spec) in messages.iter().enumerate() {
        let mut builder = DltMessageBuilder::new()
            .with_ecu_id(b"ECU1")
            .with_app_id(spec.app_id)
            .with_context_id(spec.ctx_id)
            .with_timestamp((i as u32) * 100) // 0.1ms units, 10ms apart
            .add_file_header();

        let mut buf = [0u8; 512];
        let size = builder
            .generate_log_message_with_payload(
                &mut buf,
                spec.payload.as_bytes(),
                spec.level,
                1,
                spec.verbose,
            )
            .expect("generate DLT message");

        // Patch the storage header with the actual wall-clock time
        let (secs, usecs) = now_unix();
        // Offset each message by its index so timestamps increase
        patch_storage_timestamp(&mut buf, secs + i as u32, usecs);

        file.write_all(&buf[..size])?;
        total_bytes += size;

        println!(
            "  [{:02}] ECU1/{}/{} [{:?}] \"{}\" ({} bytes)",
            i + 1,
            core::str::from_utf8(spec.app_id)
                .unwrap_or("????")
                .trim_end_matches('\0'),
            core::str::from_utf8(spec.ctx_id)
                .unwrap_or("????")
                .trim_end_matches('\0'),
            spec.level,
            spec.payload,
            size,
        );
    }

    println!(
        "\nWrote {} messages ({} bytes total) to sample.dlt",
        messages.len(),
        total_bytes
    );
    Ok(())
}
