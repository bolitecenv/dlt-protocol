//! Example: Parsing incoming DLT packets with unknown payload types
//! 
//! This example demonstrates how to parse DLT verbose payloads when you don't
//! know the types ahead of time, which is the typical case when receiving
//! DLT packets over the network or from a file.

use dlt_protocol::*;

fn main() {
    println!("DLT Packet Parser Example");
    println!("=========================\n");
    
    // Create a sample telemetry packet
    let mut buffer = [0u8; 256];
    let payload_len = {
        let mut builder = PayloadBuilder::new(&mut buffer);
        builder.add_string("Temperature").unwrap();
        builder.add_f32(23.5).unwrap();
        builder.add_string("Pressure").unwrap();
        builder.add_u32(1013).unwrap();
        builder.add_string("Status").unwrap();
        builder.add_string("OK").unwrap();
        builder.len()
    };
    
    println!("Parsing incoming packet with {} bytes of payload...\n", payload_len);
    
    // Parse the packet without knowing types ahead of time
    let mut parser = PayloadParser::new(&buffer[..payload_len]);
    let mut args = [None; 10];
    
    match parser.read_all_args(&mut args) {
        Ok(count) => {
            println!("Successfully parsed {} arguments:", count);
            
            for i in 0..count {
                if let Some(value) = args[i] {
                    print!("  Arg {}: ", i);
                    match value {
                        DltValue::String(s) => println!("String(\"{}\")", s),
                        DltValue::F32(v) => println!("Float({:.2})", v),
                        DltValue::U32(v) => println!("U32({})", v),
                        DltValue::Bool(v) => println!("Bool({})", v),
                        DltValue::I8(v) => println!("I8({})", v),
                        DltValue::I16(v) => println!("I16({})", v),
                        DltValue::I32(v) => println!("I32({})", v),
                        DltValue::I64(v) => println!("I64({})", v),
                        DltValue::U8(v) => println!("U8({})", v),
                        DltValue::U16(v) => println!("U16({})", v),
                        DltValue::U64(v) => println!("U64({})", v),
                        DltValue::U128(v) => println!("U128({})", v),
                        DltValue::F64(v) => println!("F64({:.6})", v),
                        DltValue::Raw(bytes) => println!("Raw({:?})", bytes),
                    }
                }
            }
            
            // Interpret as key-value pairs
            println!("\nInterpreted as telemetry data:");
            let mut i = 0;
            while i + 1 < count {
                if let Some(DltValue::String(key)) = args[i] {
                    if let Some(value) = args[i + 1] {
                        print!("  {} = ", key);
                        match value {
                            DltValue::F32(v) => println!("{:.2}", v),
                            DltValue::U32(v) => println!("{}", v),
                            DltValue::String(s) => println!("\"{}\"", s),
                            _ => println!("{:?}", value),
                        }
                    }
                }
                i += 2;
            }
        }
        Err(e) => {
            println!("Error parsing packet: {:?}", e);
        }
    }
}

// Additional helper functions for reference (not used in main, but useful patterns)

#[allow(dead_code)]
fn handle_incoming_packet(payload_data: &[u8], expected_args: u8) {
    let mut parser = PayloadParser::new(payload_data);
    
    // Method 1: Parse arguments one by one
    let mut arg_count = 0;
    while !parser.is_empty() && arg_count < expected_args {
        match parser.read_next() {
            Ok(value) => {
                // Handle each value based on its runtime type
                match value {
                    DltValue::Bool(v) => {
                        // Process boolean value
                        let _ = v;
                    }
                    DltValue::U32(v) => {
                        // Process u32 value
                        let _ = v;
                    }
                    DltValue::String(s) => {
                        // Process string value
                        let _ = s;
                    }
                    DltValue::F32(v) => {
                        // Process float value
                        let _ = v;
                    }
                    _ => {
                        // Handle other types
                    }
                }
                arg_count += 1;
            }
            Err(_e) => {
                // Handle parse error
                break;
            }
        }
    }
}

// Simulated batch parsing
#[allow(dead_code)]
fn parse_all_arguments(payload_data: &[u8], max_args: usize) -> usize {
    let mut parser = PayloadParser::new(payload_data);
    let mut args_buffer = [None; 16]; // Stack-allocated buffer for up to 16 arguments
    
    match parser.read_all_args(&mut args_buffer[..max_args.min(16)]) {
        Ok(count) => {
            // Process all parsed arguments
            for i in 0..count {
                if let Some(value) = args_buffer[i] {
                    // Handle each argument
                    let _ = value;
                }
            }
            count
        }
        Err(_e) => {
            // Handle error
            0
        }
    }
}

// Example: Real-world telemetry data parsing
#[allow(dead_code)]
fn parse_telemetry_message(payload_data: &[u8]) {
    let mut parser = PayloadParser::new(payload_data);
    let mut values = [None; 10];
    
    if let Ok(count) = parser.read_all_args(&mut values) {
        // Assuming the message format is:
        // [field_name: String, value: variant, field_name: String, value: variant, ...]
        let mut i = 0;
        while i + 1 < count {
            // Get field name
            if let Some(DltValue::String(field_name)) = values[i] {
                // Get field value
                if let Some(field_value) = values[i + 1] {
                    // Process field name and value pair
                    let _ = (field_name, field_value);
                }
            }
            i += 2;
        }
    }
}

// Main is not used in no_std, but we can provide a test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_usage() {
        // Create a sample payload
        let mut buffer = [0u8; 256];
        let payload_len = {
            let mut builder = PayloadBuilder::new(&mut buffer);
            builder.add_string("Temperature").unwrap();
            builder.add_f32(25.5).unwrap();
            builder.add_string("Humidity").unwrap();
            builder.add_u32(65).unwrap();
            builder.len()
        };

        // Parse it
        let count = parse_all_arguments(&buffer[..payload_len], 10);
        assert_eq!(count, 4);
    }
}
