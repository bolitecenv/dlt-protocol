use dlt_protocol::r19_11::*;

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    let mut v = Vec::new();
    let mut chars = hex.chars().filter(|c| !c.is_whitespace());
    while let (Some(hi), Some(lo)) = (chars.next(), chars.next()) {
        let s = [hi, lo].iter().collect::<String>();
        let byte = u8::from_str_radix(&s, 16).unwrap_or(0);
        v.push(byte);
    }
    v
}

fn try_parse(name: &str, hex: &str) {
    let bytes = hex_to_bytes(hex);
    println!("{} -> {} bytes", name, bytes.len());
    println!("  raw: {:02x?}", bytes);
    // check file/serial headers
    const FILE_HDR: [u8; 4] = [0x44, 0x4C, 0x54, 0x01]; // "DLT\x01"
    const SERIAL_HDR: [u8; 4] = [0x44, 0x4C, 0x53, 0x01]; // "DLS\x01"
    if bytes.len() >= 4 && bytes[0..4] == FILE_HDR {
        println!("  starts with DLT file header");
    }
    if bytes.len() >= 4 && bytes[0..4] == SERIAL_HDR {
        println!("  starts with DLS serial header");
    }
    let mut parser = DltHeaderParser::new(&bytes);
    match parser.parse_message() {
        Ok(msg) => {
            println!(
                "Parsed {}: has_serial={} has_file={}",
                name, msg.has_serial_header, msg.has_file_header
            );
            println!(
                "  HTYP={:#04x} MCNT={} LEN={}",
                msg.standard_header.htyp, msg.standard_header.mcnt, msg.standard_header.len
            );
            if let Some(ecu) = msg.ecu_id {
                println!("  ECU={:?}", ecu);
            }
            if let Some(ext) = msg.extended_header {
                println!("  EXT MSIN={:#04x} NOAR={}", ext.msin, ext.noar);
            }
            println!(
                "  payload ({} bytes): {:02x?}",
                msg.payload.len(),
                msg.payload
            );
        }
        Err(e) => {
            println!("Parse error for {}: {:?}", name, e);
            // Try to give a diagnostic: if we can read the standard header, show expected size
            if bytes.len() >= 4 {
                let htyp = bytes[0];
                let mcnt = bytes[1];
                let len = u16::from_be_bytes([bytes[2], bytes[3]]) as usize;
                let header_type = {
                    let UEH = (htyp & 0x01) != 0;
                    let WEID = (htyp & 0x04) != 0;
                    let WSID = (htyp & 0x08) != 0;
                    let WTMS = (htyp & 0x10) != 0;
                    (UEH, WEID, WSID, WTMS)
                };
                println!(
                    "  Detected standard header: HTYP={:#04x} MCNT={} LEN={}",
                    htyp, mcnt, len
                );
                println!(
                    "  Flags: UEH={}, WEID={}, WSID={}, WTMS={}",
                    header_type.0, header_type.1, header_type.2, header_type.3
                );
                let mut needed = 4; // standard header
                if header_type.1 {
                    needed += 4;
                }
                if header_type.2 {
                    needed += 4;
                }
                if header_type.3 {
                    needed += 4;
                }
                if header_type.0 {
                    needed += 10;
                }
                println!(
                    "  Header bytes expected (without serial/file header) = {}",
                    needed
                );
                println!(
                    "  Standard header LEN field says total bytes from standard header = {}",
                    len
                );
                println!("  Available bytes in buffer = {}", bytes.len());
                if bytes.len() < len {
                    println!(
                        "  -> Buffer is truncated: need at least {} bytes but have {}",
                        len,
                        bytes.len()
                    );
                } else {
                    println!("  -> Buffer length >= LEN but other mismatch caused parser error");
                }
            }
        }
    }
}

fn main() {
    let msgs = [
        (
            "Message #1",
            "3d01002e454355310000001e646c1a6d31024c4f470054455354",
        ),
        (
            "Message #2",
            "3d02002e454355310000001e646c2dfe31024c4f470054455354230000000200",
        ),
        (
            "Message #3",
            "3d040074454355310000000e648c89ab4101444c5444494e544d",
        ),
        ("Message #4", "3500002745435531648d75ae26014441310044433100"),
    ];

    for (name, hex) in msgs.iter() {
        try_parse(name, hex);
        println!("");
    }
}
