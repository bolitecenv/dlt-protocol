# DLT Protocol Specification

## Overview

The Diagnostic Log and Trace (DLT) protocol uses a unified message format for both debug data and control information. Each DLT message consists of:

- **Standard Header** (mandatory)
- **Extended Header** (optional)
- **Payload** (data segment)

---

## 1. Standard Header

The DLT Standard Header is present in every message and contains core routing and metadata.

### Structure

| Byte Range | Field | Name | Description |
|------------|-------|------|-------------|
| 0 | HTYP | Header Type | Configuration flags and version |
| 1 | MCNT | Message Counter | Sequential message counter |
| 2-3 | LEN | Length | Total message length |
| 4-7 | ECU | ECU ID | Electronic Control Unit identifier (optional) |
| 8-11 | SEID | Session ID | Session identifier (optional) |
| 12-15 | TMSP | Timestamp | Message timestamp (optional) |

### 1.1 Header Type (HTYP)

**Encoding:**

| Bit | Field | Name | Description |
|-----|-------|------|-------------|
| 0 | UEH | Use Extended Header | 1 = Extended Header present |
| 1 | MSBF | Most Significant Byte First | Payload endianness (0=little, 1=big) |
| 2 | WEID | With ECU ID | 1 = ECU ID field present |
| 3 | WSID | With Session ID | 1 = Session ID field present |
| 4 | WTMS | With Timestamp | 1 = Timestamp field present |
| 5-7 | VERS | Version Number | Protocol version |

**Important Notes:**
- Standard Header and Extended Header are **always big endian** (MSB first)
- MSBF bit controls **payload** endianness only:
  - `0` = Payload in little endian
  - `1` = Payload in big endian

### 1.2 Message Counter (MCNT)

- **Type:** 8-bit unsigned integer (0-255)
- **Initialization:** Set to 0 after DLT module initialization
- **Behavior:** Increments by 1 for each transmitted message
- **Overflow:** Wraps to 0 after reaching 255

---

## 2. Extended Header

Present when the UEH bit in the Standard Header is set to `1`.

### Structure

| Byte Range | Field | Name | Description |
|------------|-------|------|-------------|
| 0 | MSIN | Message Info | Message type and verbosity flags |
| 1 | NOAR | Number of Arguments | Argument count in payload |
| 2-5 | APID | Application ID | Source application identifier |
| 6-9 | CTID | Context ID | Context/module identifier |

### 2.1 Message Info (MSIN)

| Bit | Field | Name | Description |
|-----|-------|------|-------------|
| 0 | VERB | Verbose | 1=verbose mode, 0=non-verbose mode |
| 1-3 | MSTP | Message Type | Type category (3-bit unsigned) |
| 4-7 | MTIN | Message Type Info | Subtype within category (4-bit unsigned) |

### 2.2 Message Type (MSTP)

| Value | Name | Description |
|-------|------|-------------|
| 0x0 | DLT_TYPE_LOG | Log message |
| 0x1 | DLT_TYPE_APP_TRACE | Application trace message |
| 0x2 | DLT_TYPE_NW_TRACE | Network trace message |
| 0x3 | DLT_TYPE_CONTROL | Control message |
| 0x4-0x7 | - | Reserved |

### 2.3 Message Type Info (MTIN)

**For Log Messages (MSTP = 0x0):**

| Value | Name | Description |
|-------|------|-------------|
| 0x1 | DLT_LOG_FATAL | Fatal system error |
| 0x2 | DLT_LOG_ERROR | Application error |
| 0x3 | DLT_LOG_WARN | Warning - correct behavior not ensured |
| 0x4 | DLT_LOG_INFO | Information message |
| 0x5 | DLT_LOG_DEBUG | Debug message |
| 0x6 | DLT_LOG_VERBOSE | Verbose debug message |
| 0x7-0xF | - | Reserved |

**For Trace Messages (MSTP = 0x1):**

| Value | Name | Description |
|-------|------|-------------|
| 0x1 | DLT_TRACE_VARIABLE | Variable value |
| 0x2 | DLT_TRACE_FUNCTION_IN | Function entry |
| 0x3 | DLT_TRACE_FUNCTION_OUT | Function exit |
| 0x4 | DLT_TRACE_STATE | State machine state |
| 0x5 | DLT_TRACE_VFB | RTE events |
| 0x6-0xF | - | Reserved |

**For Network Messages (MSTP = 0x2):**

| Value | Name | Description |
|-------|------|-------------|
| 0x1 | DLT_NW_TRACE_IPC | Inter-Process Communication |
| 0x2 | DLT_NW_TRACE_CAN | CAN bus |
| 0x3 | DLT_NW_TRACE_FLEXRAY | FlexRay bus |
| 0x4 | DLT_NW_TRACE_MOST | MOST bus |
| 0x5 | DLT_NW_TRACE_ETHERNET | Ethernet |
| 0x6 | DLT_NW_TRACE_SOMEIP | SOME/IP |
| 0x7-0xF | - | User defined |

**For Control Messages (MSTP = 0x3):**

| Value | Name | Description |
|-------|------|-------------|
| 0x1 | DLT_CONTROL_REQUEST | Control request |
| 0x2 | DLT_CONTROL_RESPONSE | Control response |
| 0x3-0xF | - | Reserved |

### 2.4 Number of Arguments (NOAR)

- **Type:** 8-bit unsigned integer
- **Verbose mode (VERB=1):** Contains the number of arguments in payload
- **Non-verbose mode (VERB=0):** Set to 0x00

---

## 3. Payload - Type Info

In verbose mode, each argument in the payload begins with a 32-bit Type Info field containing metadata about the data that follows.

### Type Info Structure (32 bits)

| Bit Range | Field | Name | Description |
|-----------|-------|------|-------------|
| 0-3 | TYLE | Type Length | Data size indicator |
| 4 | BOOL | Type Bool | Boolean data type flag |
| 5 | SINT | Type Signed | Signed integer flag |
| 6 | UINT | Type Unsigned | Unsigned integer flag |
| 7 | FLOA | Type Float | Float data type flag |
| 8 | ARAY | Type Array | Array data type flag |
| 9 | STRG | Type String | String data type flag |
| 10 | RAWD | Type Raw | Raw data type flag |
| 11 | VARI | Variable Info | Includes name/unit metadata |
| 12 | FIXP | Fixed Point | Fixed-point encoding flag |
| 13 | TRAI | Trace Info | Trace information flag |
| 14 | STRU | Type Struct | Structured data flag |
| 15-17 | SCOD | String Coding | String encoding type |
| 18-31 | - | Reserved | Reserved for future use |

### 3.1 Type Length (TYLE)

Specifies the bit length of standard data types:

| Value | Bit Length | Applicable Types |
|-------|------------|------------------|
| 0x00 | Not defined | - |
| 0x01 | 8 bit | BOOL, SINT, UINT |
| 0x02 | 16 bit | SINT, UINT, FLOA |
| 0x03 | 32 bit | SINT, UINT, FLOA |
| 0x04 | 64 bit | SINT, UINT, FLOA |
| 0x05 | 128 bit | SINT, UINT, FLOA |
| 0x06-0x0F | Reserved | - |

### 3.2 String Coding (SCOD)

Defines string encoding (3-bit field):

| Value | Encoding | Description |
|-------|----------|-------------|
| 0x00 | ASCII | 8-bit ASCII characters |
| 0x01 | UTF-8 | UTF-8 encoding |
| 0x02-0x07 | - | Reserved |

**Note:** String coding applies to data strings (Type String, Trace Info). Variable names and units are always ASCII.

---

## 4. Data Types

### 4.1 Type Bool (BOOL)

**Requirements:**
- TYLE must be 1 (8 bit)
- Data: 0x0 = FALSE, any other value = TRUE

**Payload Structure:**

| Field | Length | Description |
|-------|--------|-------------|
| *If VARI is set:* | | |
| Length of Name | 16 bit | Unsigned integer (includes null terminator) |
| Name | Variable | Null-terminated ASCII string |
| Data | 8 bit | Boolean value |

### 4.2 Type Signed (SINT) / Type Unsigned (UINT)

**Requirements:**
- TYLE: 1, 2, 3, 4, or 5
- Identical structure, different data interpretation

**Payload Structure:**

| Field | Length | Description |
|-------|--------|-------------|
| *If VARI is set:* | | |
| Length of Name | 16 bit | Unsigned integer (includes null terminator) |
| Length of Unit | 16 bit | Unsigned integer (includes null terminator) |
| Name | Variable | Null-terminated ASCII string |
| Unit | Variable | Null-terminated ASCII string |
| *If FIXP is set:* | | |
| Quantization | 32 bit | IEEE 754-2008 float |
| Offset | 32/64/128 bit | Signed integer (size depends on TYLE) |
| Data | 8/16/32/64/128 bit | Integer value (length per TYLE) |

**Fixed Point Calculation:**
```
logical_value = physical_value * quantization + offset
```

**Offset Size Rules:**
- TYLE ≤ 3: 32-bit offset
- TYLE = 4: 64-bit offset
- TYLE = 5: 128-bit offset

### 4.3 Type Float (FLOA)

**Requirements:**
- TYLE: 2, 3, 4, or 5 (per IEEE 754-2008)
- Binary float representation

**IEEE 754-2008 Formats:**

| TYLE | Total Bits | Mantissa | Exponent | Common Name |
|------|------------|----------|----------|-------------|
| 2 | 16 bit | 10 bit | 5 bit | Half precision |
| 3 | 32 bit | 23 bit | 8 bit | Single precision |
| 4 | 64 bit | 52 bit | 11 bit | Double precision |
| 5 | 128 bit | 112 bit | 15 bit | Quadruple precision |

**Payload Structure:**

| Field | Length | Description |
|-------|--------|-------------|
| *If VARI is set:* | | |
| Length of Name | 16 bit | Unsigned integer |
| Length of Unit | 16 bit | Unsigned integer |
| Name | Variable | Null-terminated ASCII string |
| Unit | Variable | Null-terminated ASCII string |
| Data | 16/32/64/128 bit | IEEE 754 float (length per TYLE) |

### 4.4 Type String (STRG)

**Requirements:**
- SCOD must be specified
- String encoding per SCOD field

**Payload Structure:**

| Field | Length | Description |
|-------|--------|-------------|
| Length of String | 16 bit | String length including null terminator |
| *If VARI is set:* | | |
| Length of Name | 16 bit | Unsigned integer |
| Name | Variable | Null-terminated ASCII string |
| Data String | Variable | Null-terminated string (encoding per SCOD) |

### 4.5 Type Array (ARAY)

**Requirements:**
- Supports BOOL, SINT, UINT, FLOA element types
- N-dimensional arrays
- TYLE and FIXP interpreted as for standard types

**Payload Structure:**

| Field | Length | Description |
|-------|--------|-------------|
| Number of Dimensions | 16 bit | Unsigned integer |
| *Loop for each dimension:* | | |
| Number of Entries | 16 bit | Entries in this dimension |
| *If VARI is set:* | | |
| Length of Name | 16 bit | Unsigned integer |
| Length of Unit | 16 bit | Unsigned integer |
| Name | Variable | Null-terminated ASCII string |
| Unit | Variable | Null-terminated ASCII string |
| *If FIXP is set:* | | |
| Quantization | 32 bit | IEEE 754 float |
| Offset | 32/64/128 bit | Signed integer (size per TYLE) |
| Array Data | Variable | All array elements (C90 ordering) |

**Note:** Fixed-point calculation applies uniformly to all array elements.

### 4.6 Type Struct (STRU)

**Requirements:**
- Contains one or more standard argument types
- Recursive structure allowed

**Payload Structure:**

| Field | Length | Description |
|-------|--------|-------------|
| Number of Entries | 16 bit | Count of structure members |
| *If VARI is set:* | | |
| Length of Name | 16 bit | Unsigned integer |
| Name | Variable | Null-terminated ASCII string |
| *For each entry:* | | |
| Type Info | 32 bit | Entry type metadata |
| Data Payload | Variable | Entry data (any standard type) |

### 4.7 Type Raw (RAWD)

**Requirements:**
- Uninterpreted binary data
- Optional variable info

**Payload Structure:**

| Field | Length | Description |
|-------|--------|-------------|
| Length of Raw Data | 16 bit | Data length in bytes |
| *If VARI is set:* | | |
| Length of Name | 16 bit | Unsigned integer |
| Name | Variable | Null-terminated ASCII string |
| Raw Data | Variable | Uninterpreted binary data |

### 4.8 Type Trace Info (TRAI)

**Requirements:**
- SCOD must be specified
- Contains trace context (module/function names)

**Payload Structure:**

| Field | Length | Description |
|-------|--------|-------------|
| Length of String | 16 bit | String length including null terminator |
| Trace Data String | Variable | Null-terminated string (encoding per SCOD) |

---

## 5. Variable Info (VARI)

When the VARI bit is set, additional metadata describes the variable:

- **Name:** Identifier of the variable
- **Unit:** Measurement unit (for numeric types)
- **Encoding:** Always ASCII (8-bit characters)
- **Termination:** All strings are null-terminated
- **Length Fields:** 16-bit unsigned integers specifying character count including null terminator

---

## 6. Recommended Arguments

For source code traceability, the first two arguments should identify the code location:

### Without Long ID Arguments:

1. **Argument 1:** String with Variable Info
   - Name field: `"source_file"`
   - Data field: Source file identifier

2. **Argument 2:** UINT (32-bit) with Variable Info
   - Name field: `"line_number"`
   - Data field: Line number (starting from 1)

### With Long ID Arguments:

The source file and line number arguments follow immediately after the long ID arguments, using the same format.

---

## 7. Example: 8-bit Unsigned Integer with Variable Info

```
Type Info:        0001 0010 0001 0000 0000 0000 0000 0000
                  (TYLE=0x1, UINT=1, VARI=1)

Length of Name:   12 (0x000C)
Length of Unit:   8 (0x0008)
Name:             "temperature\0" (96 bits)
Unit:             "Celsius\0" (64 bits)
Data:             25 (0x19)
```

---

## 8. Valid Type Info Combinations

### Mandatory/Optional Field Matrix

| Type | TYLE | VARI | FIXP | SCOD |
|------|------|------|------|------|
| BOOL | x | o | - | - |
| SINT | x | o | o | - |
| UINT | x | o | o | - |
| FLOA | x | o | - | - |
| ARAY | x | o | o | - |
| STRG | - | o | - | x |
| RAWD | - | o | - | - |
| TRAI | - | - | - | x |
| STRU | - | o | - | - |

**Legend:**
- `x` = Mandatory
- `o` = Optional
- `-` = Not allowed

---

## Key Specifications

1. **Endianness:**
   - Headers: Always big endian
   - Payload: Per MSBF bit (0=little, 1=big)

2. **String Encoding:**
   - Data strings: Per SCOD field (ASCII or UTF-8)
   - Metadata strings: Always ASCII

3. **Null Termination:**
   - All strings are null-terminated
   - Length fields include the null terminator

4. **Fixed-Point:**
   - Only valid with SINT or UINT
   - Quantization: 32-bit IEEE 754 float
   - Offset: Signed integer (size depends on TYLE)

5. **Message Counter:**
   - 8-bit, wraps at 255
   - Per-LogChannel basis