// DLT TCP server — cycles through all message-patterns.json view patterns once per second.
//
// View patterns covered (from message-patterns.json):
//   register  — REG:<ts>:<name>:<value>  |  REG:<name>:<value>  |  REG:<name>:0x<hex>
//   chart     — <name>:<ts>:<value>       |  <name>:<ts>:<v1>,<v2>,...
//   trace     — <id>:<ts>:start:<meta>    |  <id>:<ts>:end
//   callstack — <thread>:<fn>:<ts>:start  |  <fn>:<ts>:end
//   log       — ECU:APP:CTX:LEVEL:<msg>   |  LEVEL: <msg>
//   service   — [SVC_RESP] id=0x.. status=.. app=.. ctx=..
//   debug     — GDB console output  |  Cargo/compiler error line
//
// Usage: cargo run --example dlt_tcp_server

use dlt_protocol::r19_11::{DltMessageBuilder, MtinTypeDltLog};
use std::io::Write;
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Unix timestamp as floating-point seconds.
fn now_ts() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

/// One message to send: target APP/CTX IDs, DLT log level, payload string.
struct Msg {
    app_id: &'static [u8; 4],
    ctx_id: &'static [u8; 4],
    level: MtinTypeDltLog,
    payload: String,
}

/// Build the full list of pattern-example messages for the current cycle.
/// Each entry exercises exactly one pattern from message-patterns.json.
fn build_cycle(counter: u32, ts: f64) -> Vec<Msg> {
    // Vary some values so DLT viewers show changing data.
    let voltage = 3.3 + (ts * 0.05 % 0.5);
    let speed = 1500 + (counter % 50) * 10;
    let status_hex = counter & 0xFF;
    let temp = 70.0 + (ts * 0.2 % 5.0);
    let ax = (ts * 0.3).sin() * 0.5f64;
    let ay = (ts * 0.7).cos() * 0.3f64;

    vec![
        // ── register view ──────────────────────────────────────────────────
        Msg {
            app_id: b"APP1",
            ctx_id: b"REGI",
            level: MtinTypeDltLog::DltLogInfo,
            // reg_with_timestamp: REG:<ts>:<name>:<value>
            payload: format!("REG:{ts:.3}:voltage:{voltage:.2}"),
        },
        Msg {
            app_id: b"APP1",
            ctx_id: b"REGI",
            level: MtinTypeDltLog::DltLogDebug,
            // reg_simple: REG:<name>:<value>
            payload: format!("REG:motor_speed:{speed}"),
        },
        Msg {
            app_id: b"APP1",
            ctx_id: b"REGI",
            level: MtinTypeDltLog::DltLogDebug,
            // reg_hex: REG:<name>:0x<hex>
            payload: format!("REG:STATUS_REG:0x{status_hex:02X}"),
        },
        // ── chart view ─────────────────────────────────────────────────────
        Msg {
            app_id: b"APP1",
            ctx_id: b"CHRT",
            level: MtinTypeDltLog::DltLogInfo,
            // chart_named_ts_value: <name>:<ts>:<value>
            payload: format!("temperature:{ts:.3}:{temp:.1}"),
        },
        Msg {
            app_id: b"APP1",
            ctx_id: b"CHRT",
            level: MtinTypeDltLog::DltLogInfo,
            // chart_multi_value: <name>:<ts>:<v1>,<v2>,...
            payload: format!("accel:{ts:.3}:{ax:.2},{ay:.2},9.81"),
        },
        // ── trace view ─────────────────────────────────────────────────────
        Msg {
            app_id: b"APP1",
            ctx_id: b"TRCE",
            level: MtinTypeDltLog::DltLogDebug,
            // trace_span: <id>:<ts>:start:<metadata>
            payload: format!("task_{counter}:{ts:.3}:start:priority=5"),
        },
        Msg {
            app_id: b"APP1",
            ctx_id: b"TRCE",
            level: MtinTypeDltLog::DltLogDebug,
            // trace_span: <id>:<ts>:start:<metadata>
            payload: format!("task_{counter}:{ts:.3}:end:priority=5"),
        },
        Msg {
            app_id: b"APP1",
            ctx_id: b"TRCE",
            level: MtinTypeDltLog::DltLogDebug,
            // trace_span_no_meta: <id>:<ts>:end
            payload: format!("isr_handler:{ts:.3}:end"),
        },
        // ── callstack view ─────────────────────────────────────────────────
        Msg {
            app_id: b"APP1",
            ctx_id: b"CALL",
            level: MtinTypeDltLog::DltLogDebug,
            // call_thread_fn: <thread>:<fn>:<ts>:start
            payload: format!("main:hal_init:{ts:.3}:start"),
        },
        Msg {
            app_id: b"APP1",
            ctx_id: b"CALL",
            level: MtinTypeDltLog::DltLogDebug,
            // call_thread_fn: <thread>:<fn>:<ts>:start
            payload: format!("main:hal_init:{ts:.3}:end"),
        },
        Msg {
            app_id: b"APP1",
            ctx_id: b"CALL",
            level: MtinTypeDltLog::DltLogDebug,
            // call_fn_only: <fn>:<ts>:end
            payload: format!("CAN_Transmit:{ts:.3}:start"),
        },
        Msg {
            app_id: b"APP1",
            ctx_id: b"CALL",
            level: MtinTypeDltLog::DltLogDebug,
            // call_fn_only: <fn>:<ts>:end
            payload: format!("CAN_Transmit:{ts:.3}:end"),
        },
        // ── log view ───────────────────────────────────────────────────────
        Msg {
            app_id: b"APP1",
            ctx_id: b"LOGS",
            level: MtinTypeDltLog::DltLogInfo,
            // dlt_colon_full: ECU:APP:CTX:LEVEL:<payload>
            payload: format!("ECU1:APP1:LOGS:INFO:System event #{counter}"),
        },
        Msg {
            app_id: b"APP1",
            ctx_id: b"LOGS",
            level: MtinTypeDltLog::DltLogWarn,
            // log_level_prefix: LEVEL: <message>
            payload: format!("WARN: memory usage {:.0}%", 60.0 + (ts * 0.3 % 30.0)),
        },
        // ── service view ───────────────────────────────────────────────────
        Msg {
            app_id: b"APP1",
            ctx_id: b"SRVC",
            level: MtinTypeDltLog::DltLogInfo,
            // service_response: [SVC_RESP] id=0x.. status=.. ...
            payload: format!("[SVC_RESP] id=0x03 status=0 app=APP1 ctx=CTX1"),
        },
        // ── debug view ─────────────────────────────────────────────────────
        Msg {
            app_id: b"APP1",
            ctx_id: b"DEBG",
            level: MtinTypeDltLog::DltLogDebug,
            // gdb_response: ~"<text>"
            payload: format!(
                "~\"Breakpoint {} at 0x{:06X}\"",
                counter % 10 + 1,
                0x400000 + counter * 4
            ),
        },
    ]
}

fn send_msg(
    stream: &mut std::net::TcpStream,
    msg: &Msg,
    cycle: u32,
) -> Result<usize, std::io::Error> {
    let mut builder = DltMessageBuilder::new()
        .with_ecu_id(b"ECU1")
        .with_app_id(msg.app_id)
        .with_context_id(msg.ctx_id)
        .with_timestamp(cycle * 100)
        .add_serial_header();

    let mut buffer = [0u8; 512];
    let len = builder
        .generate_log_message_with_payload(&mut buffer, msg.payload.as_bytes(), msg.level, 1, true)
        .map_err(|e| std::io::Error::other(format!("{e:?}")))?;

    stream.write_all(&buffer[..len])?;
    Ok(len)
}

fn handle_client(mut stream: std::net::TcpStream) {
    let addr = stream.peer_addr().unwrap();
    println!("Client connected: {addr}");

    let mut cycle: u32 = 0;

    loop {
        let ts = now_ts();
        let messages = build_cycle(cycle, ts);
        let total = messages.len();

        for (i, msg) in messages.iter().enumerate() {
            match send_msg(&mut stream, msg, cycle * total as u32 + i as u32) {
                Ok(bytes) => println!(
                    "[cycle {cycle:04}:{i:02}] {}/{} {:?} \"{}\" ({bytes}B) → {addr}",
                    String::from_utf8_lossy(msg.app_id).trim_end_matches('\0'),
                    String::from_utf8_lossy(msg.ctx_id).trim_end_matches('\0'),
                    msg.level,
                    msg.payload,
                ),
                Err(e) => {
                    println!("Client {addr} disconnected: {e}");
                    return;
                }
            }
            thread::sleep(Duration::from_millis(200));
        }

        cycle += 1;
        // Pause between cycles so the viewer has time to display
        thread::sleep(Duration::from_secs(2));
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3490")?;
    println!("DLT TCP server listening on 127.0.0.1:3490");
    println!(
        "Sends {} pattern types per cycle (200ms apart), then waits 2s.",
        build_cycle(0, 0.0).len()
    );

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => eprintln!("Accept error: {e}"),
        }
    }
    Ok(())
}
