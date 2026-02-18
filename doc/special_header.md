## Special Headers: Serial header and DLT file header

This project supports two small, 4-byte headers used to identify DLT data formats:

- Serial header: used for streaming transports (per-message framing)
- DLT file header: used at the start of DLT files on disk

Both headers are 4 bytes long and are fixed byte sequences. They are not part
of the DLT standard header `len` field — the `len` field in the standard
header counts bytes from the standard header through the end of the payload,
and therefore does not include the serial or file header bytes.

### Serial header ("DLS\x01")

- Bytes: `0x44 0x4C 0x53 0x01` (ASCII "DLS" followed by `0x01`)
- Constant in code: `DLT_SERIAL_HEADER_ARRAY`
- Purpose: marks the start of an on-the-wire DLT message when messages are
	transported over serial-like links. When present, the parser will detect and
	skip this 4-byte header before parsing the standard DLT headers.

Notes:

- The serial header may appear before each message in a stream.
- The standard header length (`len` field) does NOT include these 4 bytes.

### DLT file header ("DLT\x01")

- Bytes: `0x44 0x4C 0x54 0x01` (ASCII "DLT" followed by `0x01`)
- Constant in code: `DLT_FILE_HEADER_ARRAY`
- Purpose: marks the start of a DLT file on disk. It is intended to help
	tools quickly detect DLT-formatted files. When present at the start of the
	buffer, the parser will detect and skip this 4-byte header before parsing
	the first DLT message.

Notes:

- The file header is only expected at the very beginning of a file (or the
	first buffer chunk). It is NOT part of the DLT message length, and the
	parser subtracts the file-header size before computing payload offsets.
- Tests: `tests/r19_11_it.rs` includes a test that generates a message with the
	file header and verifies parsing.

### Interaction and ordering

- If both headers are used together (rare), the file header is written first
	(at file start), and the serial header may appear before individual messages
	inside the file or stream. The generator in this repository writes the file
	header before the serial header when both are enabled.

### Implementation pointers

- Header constants are in `src/r19_11/header.rs`:
	- `DLT_SERIAL_HEADER_ARRAY`
	- `DLT_FILE_HEADER_ARRAY`
- Parser helpers `check_serial_header`/`skip_serial_header` and
	`check_file_header`/`skip_file_header` are implemented in the header parser
	and used when parsing messages.
- The message builder can be configured to emit the file header via
	`DltMessageBuilder::add_file_header()`.

If you want, I can add an example showing how to write a DLT file with the
file header and multiple messages. Would you like that?
