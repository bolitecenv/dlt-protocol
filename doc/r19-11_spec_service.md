# DLT Protocol Service Messages

## Overview

This document describes the DLT (Diagnostic Log and Trace) Protocol service messages/commands as defined in the AUTOSAR specification. Control messages are normal DLT messages with a Standard Header, Extended Header, and payload consisting of a 32-bit Service ID and parameters.

## Service Message Format

Control messages contain:
- **Standard Header**
- **Extended Header** 
- **Payload**: Service ID (32-bit unsigned integer) + parameters

---

## Supported Service Commands

| Service ID | Command Name | Description |
|------------|--------------|-------------|
| 0x01 | SetLogLevel | Set the Log Level |
| 0x02 | SetTraceStatus | Enable/Disable Trace Messages |
| 0x03 | GetLogInfo | Returns the LogLevel for applications |
| 0x04 | GetDefaultLogLevel | Returns the LogLevel for wildcards |
| 0x05 | StoreConfiguration | Stores the current configuration non-volatile |
| 0x06 | RestoreToFactoryDefault | Sets the configuration back to default |
| 0x0A | SetMessageFiltering | Enable/Disable message filtering |
| 0x11 | SetDefaultLogLevel | Sets the LogLevel for wildcards |
| 0x12 | SetDefaultTraceStatus | Enable/Disable TraceMessages for wildcards |
| 0x13 | GetSoftwareVersion | Get the ECU software version |
| 0x15 | GetDefaultTraceStatus | Get the current TraceLevel for wildcards |
| 0x17 | GetLogChannelNames | Returns the LogChannel's name |
| 0x1F | GetTraceStatus | Returns the current TraceStatus |
| 0x20 | SetLogChannelAssignment | Adds/Removes the given LogChannel as output path |
| 0x21 | SetLogChannelThreshold | Sets the filter threshold for the given LogChannel |
| 0x22 | GetLogChannelThreshold | Returns the current LogLevel for a given LogChannel |
| 0x23 | BufferOverflowNotification | Report that a buffer overflow occurred |
| 0x24 | SyncTimeStamp | Reports synchronized absolute time |
| 0xFFF - 0xFFFFFFFF | CallSWCInjection | Used to call a function in an application |

---

## Service Definitions

### 0x01 - SetLogLevel

Sets the log level for a specific Application ID/Context ID combination.

**Request Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | applicationId | 4×uint8 | Application ID (NULL = all apps) |
| 2 | contextId | 4×uint8 | Context ID (NULL = all contexts for the app) |
| 3 | newLogLevel | sint8 | New log level:<br>• DLT_LOG_FATAL to DLT_LOG_VERBOSE<br>• 0 = block all messages<br>• -1 = use default log level |
| 4 | reserved | 4×uint8 | Reserved (fill with zeros) |

**Response Parameters:**

| # | Name | Type | Values |
|---|------|------|--------|
| 1 | status | uint8 | 0=OK, 1=NOT_SUPPORTED, 2=ERROR |

---

### 0x02 - SetTraceStatus

Enables or disables trace messages for a given Application ID/Context ID.

**Request Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | applicationId | 4×uint8 | Application ID (NULL = all apps) |
| 2 | contextId | 4×uint8 | Context ID (NULL = all contexts for the app) |
| 3 | newTraceStatus | sint8 | 1=On, 0=Off, -1=use default |
| 4 | reserved | 4 bytes | Reserved (fill with zeros) |

**Response Parameters:**

| # | Name | Type | Values |
|---|------|------|--------|
| 1 | status | uint8 | 0=OK, 1=NOT_SUPPORTED, 2=ERROR |

---

### 0x03 - GetLogInfo

Requests information about registered Applications and Contexts including IDs, log levels, trace statuses, and descriptions.

**Request Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | options | uint8 | 6 = with log level and trace status<br>7 = with log level, trace status, and descriptions |
| 2 | applicationId | 4×uint8 | Application ID (NULL = all apps) |
| 3 | contextId | 4×uint8 | Context ID (NULL = all contexts) |
| 4 | reserved | 4×uint8 | Reserved (fill with zeros) |

**Response Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | status | uint8 | 1=NOT_SUPPORTED, 2=ERROR, 6/7=matching request option, 8=NO matching contexts, 9=OVERFLOW |
| 2 | applicationIds | LogInfoType | Complex structure with app/context info |
| 3 | reserved | 4×uint8 | Reserved (fill with zeros) |

**LogInfoType Structure (status 6 or 7):**
```
appIdCount (uint16)
appIdInfo[] {
  appID (uint8[4])
  contextIdCount (uint16)
  contextIdInfoList[] {
    contextId (uint8[4])
    logLevel (enum 0x00..0x06)
    traceStatus (uint8)
    [if option 7:]
      lenContextDescription (uint16)
      contextDesc[] (uint8[])
  }
  [if option 7:]
    appDescLen (uint16)
    appDesc[] (uint8[])
}
```

---

### 0x04 - GetDefaultLogLevel

Returns the actual default log level.

**Request Parameters:** None

**Response Parameters:**

| # | Name | Type | Values |
|---|------|------|--------|
| 1 | status | uint8 | 0=OK, 1=NOT_SUPPORTED, 2=ERROR |
| 2 | logLevel | uint8 | Actual log level |

---

### 0x05 - StoreConfiguration

Stores the current DLT configuration non-volatile.

**Request Parameters:** None

**Response Parameters:**

| # | Name | Type | Values |
|---|------|------|--------|
| 1 | status | uint8 | 0=OK, 1=NOT_SUPPORTED, 2=ERROR |

---

### 0x06 - ResetToFactoryDefault

Resets the DLT configuration to factory defaults.

**Request Parameters:** None

**Response Parameters:**

| # | Name | Type | Values |
|---|------|------|--------|
| 1 | status | uint8 | 0=OK, 1=NOT_SUPPORTED, 2=ERROR |

---

### 0x0A - SetMessageFiltering

Switches message filtering on/off.

**Request Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | newstatus | uint8 | 0=OFF, 1=ON |

**Response Parameters:**

| # | Name | Type | Values |
|---|------|------|--------|
| 1 | status | uint8 | 0=OK, 1=NOT_SUPPORTED, 2=ERROR |

---

### 0x11 - SetDefaultLogLevel

Sets the default log level for all contexts not explicitly configured.

**Request Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | newLogLevel | sint8 | DLT_LOG_FATAL to DLT_LOG_VERBOSE<br>0 = block all<br>-1 = pass all |
| 2 | reserved | 4×uint8 | Reserved (fill with zeros) |

**Response Parameters:**

| # | Name | Type | Values |
|---|------|------|--------|
| 1 | status | uint8 | 0=OK, 1=NOT_SUPPORTED, 2=ERROR |

---

### 0x12 - SetDefaultTraceStatus

Enables or disables trace messages for all contexts not explicitly configured.

**Request Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | newTraceStatus | sint8 | 1=On, 0=Off |
| 2 | reserved | 4 bytes | Reserved (fill with zeros) |

**Response Parameters:**

| # | Name | Type | Values |
|---|------|------|--------|
| 1 | status | uint8 | 0=OK, 1=NOT_SUPPORTED, 2=ERROR |

---

### 0x13 - GetSoftwareVersion

Returns the ECU's software version.

**Request Parameters:** None

**Response Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | status | uint8 | 0=OK, 1=NOT_SUPPORTED, 2=ERROR |
| 2 | length | uint32 | Length of swVersion string |
| 3 | swVersion | char[] | Software version string |

---

### 0x15 - GetDefaultTraceStatus

Returns the actual default trace status.

**Request Parameters:** None

**Response Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | status | uint8 | 0=OK, 1=NOT_SUPPORTED, 2=ERROR |
| 2 | traceStatus | uint8 | 0=off, 1=on |

---

### 0x17 - GetLogChannelNames

Returns all available communication interface (LogChannel) names.

**Request Parameters:** None

**Response Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | status | uint8 | 0=OK, 1=NOT_SUPPORTED, 2=ERROR |
| 2 | countIf | uint8 | Count of transmitted interface names |
| 3 | logChannelNames | 4×uint8[] | List of LogChannel names (4 bytes each) |

---

### 0x1F - GetTraceStatus

Returns the actual trace status for a specific Application ID/Context ID.

**Request Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | applicationId | 4×uint8 | Addressed Application ID |
| 2 | contextId | 4×uint8 | Addressed Context ID |

**Response Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | status | uint8 | 0=OK, 1=NOT_SUPPORTED, 2=ERROR |
| 2 | traceStatus | uint8 | 0=off, 1=on |

---

### 0x20 - SetLogChannelAssignment

Adds or removes an Application ID/Context ID from a LogChannel.

**Request Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | applicationId | 4×uint8 | Addressed Application ID |
| 2 | contextId | 4×uint8 | Addressed Context ID |
| 3 | logChannelName | 4×uint8 | Name of the addressed LogChannel |
| 4 | addRemoveOp | uint8 | 0=Remove, 1=Add |

**Response Parameters:**

| # | Name | Type | Values |
|---|------|------|--------|
| 1 | status | uint8 | 0=OK, 1=NOT_SUPPORTED, 2=ERROR |

---

### 0x21 - SetLogChannelThreshold

Sets the LogLevel and TraceStatus for a LogChannel.

**Request Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | logChannelName | 4×uint8 | Name of the addressed LogChannel |
| 2 | logLevelThreshold | uint8 | 0=OFF, 1=FATAL, 2=ERROR, 3=WARN, 4=INFO, 5=DEBUG, 6=VERBOSE |
| 3 | traceStatus | uint8 | 0=blocked, 1=can pass |

**Response Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | status | uint8 | 0=OK, 1=NOT_SUPPORTED, 2=ERROR |
| 2 | traceStatus | uint8 | Actual trace status (0=off, 1=on) |

---

### 0x22 - GetLogChannelThreshold

Returns the LogLevel and TraceStatus for a LogChannel.

**Request Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | logChannelName | 4×uint8 | Name of the addressed LogChannel |

**Response Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | status | uint8 | 0=OK, 1=NOT_SUPPORTED, 2=ERROR |
| 2 | logLevelThreshold | uint8 | 0=OFF, 1=FATAL, 2=ERROR, 3=WARN, 4=INFO, 5=DEBUG, 6=VERBOSE |
| 3 | traceStatus | uint8 | 0=blocked, 1=can pass |

---

### 0x23 - BufferOverflowNotification

Sent by the DLT module when the message buffer overflows.

**Request Parameters:** None

**Response Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | status | uint8 | 0=OK, 1=NOT_SUPPORTED, 2=ERROR |
| 2 | overflowCounter | uint32 | Count of lost messages since last overflow notification |

---

### 0x24 - SyncTimeStamp

Reports the synchronized absolute time starting from 1970-01-01.

**Request Parameters:** None

**Response Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | status | uint8 | 0=OK, 1=NOT_SUPPORTED, 2=ERROR |
| 2 | synctimestamp | TimeStamp | Time structure (see below) |

**TimeStamp Structure:**
- uint32: Nanoseconds part of the time
- uint32: Seconds part of the time
- uint16: Seconds part of the time (MSB)

**Note:** This message is sent once at transmission start, then every 10 minutes, and after time jumps.

---

### 0xFFF - 0xFFFFFFFF - CallSWCInjection

Calls a function in an application. Service IDs from 0xFFF to 0xFFFFFFFF are reserved for this purpose.

**Request Parameters:**

| # | Name | Type | Description |
|---|------|------|-------------|
| 1 | dataLength | uint32 | Length of provided data |
| 2 | data[] | uint8[] | Data to provide to the application |

**Response Parameters:**

| # | Name | Type | Values |
|---|------|------|--------|
| 1 | status | uint8 | 0=OK, 1=NOT_SUPPORTED, 2=ERROR, 3=PENDING |

**Note:** The header must contain Application ID (APID), Context ID (CTID), and Session ID (SEID). The pair of APID/CTID with SEID identifies a unique client-server interface.

---

## Log Levels

| Value | Level | Description |
|-------|-------|-------------|
| 0 | DLT_LOG_OFF | All messages blocked |
| 1 | DLT_LOG_FATAL | Fatal errors only |
| 2 | DLT_LOG_ERROR | Errors and fatal |
| 3 | DLT_LOG_WARN | Warnings, errors, and fatal |
| 4 | DLT_LOG_INFO | Info and above |
| 5 | DLT_LOG_DEBUG | Debug and above |
| 6 | DLT_LOG_VERBOSE | All messages |

---

## Deprecated Commands

The following service IDs are deprecated and no longer supported:
- 0x07 - SetComInterfaceStatus
- 0x08 - SetComInterfaceMaxBandwidth
- 0x09 - SetVerboseMode
- 0x0C - GetLocalTime
- 0x0D - SetUseECUID
- 0x0E - SetUseSessionID
- 0x0F - SetUseTimestamp
- 0x10 - SetUseExtendedHeader
- 0x14 - MessageBufferOverflow
- 0x16 - GetComInterfaceStatus
- 0x18 - GetComInterfaceMaxBandwidth
- 0x19 - GetVerboseModeStatus
- 0x1A - GetMessageFilteringStatus
- 0x1B - GetUseECUID
- 0x1C - GetUseSessionID
- 0x1D - GetUseTimestamp
- 0x1E - GetUseExtendedHeader

---

## Storage Header Format

External clients must add a Storage Header before storing received DLT messages:

| Offset | Length | Name | Description |
|--------|--------|------|-------------|
| 0 | 4 | DLT-Pattern | 0x44 4C 54 01 ("DLT"+0x01) |
| 4 | 4 | seconds | Unix timestamp (seconds since 1970-01-01) |
| 8 | 4 | microseconds | Microseconds of the second (0-999,999) |
| 12 | 4 | ECU ID | Four character ECU identifier |
| 16 | variable | Message | DLT message (Header + Extended Header + Payload) |

---

*Document based on AUTOSAR Log and Trace Protocol Specification (Document ID 787)*