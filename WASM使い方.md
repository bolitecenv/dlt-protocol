# DLT Protocol WASM 使い方ガイド

## 概要

DLT Protocol R19-11実装をWebAssemblyにコンパイルし、ブラウザ上でDLTメッセージの解析・生成が可能になります。

**重要:** メッセージ解析には**r19-11の公式`DltHeaderParser`**を使用しており、手動バイト解析ではなくテスト済みの標準準拠パーサーで信頼性の高い解析を実現しています。

## 🔨 WASMのビルド

### 前提条件
```bash
# Rust toolchainのインストール
rustup target add wasm32-unknown-unknown
```

### ビルド実行
```bash
# プロジェクトルートで実行
bash build-wasm.sh
```

**出力ファイル:**
- `target/wasm32-unknown-unknown/release/examples/wasm_demo.wasm` (約43KB)

## 🌐 ブラウザでの使用方法

### 1. ローカルサーバーの起動

```bash
# プロジェクトルートで実行
python3 -m http.server 8000
```

### 2. ブラウザで開く

```
http://localhost:8000/examples/test_messages.html
```

## 📝 提供されるAPI関数

WASMモジュールは以下の関数をエクスポートします：

### メッセージ生成

#### `create_dlt_message(buffer_ptr, buffer_len) -> i32`
シンプルなDLTログメッセージを生成します。

**パラメータ:**
- `buffer_ptr`: 出力バッファへのポインタ
- `buffer_len`: バッファサイズ (最低100バイト)

**戻り値:**
- 正の値: 生成されたメッセージサイズ
- 負の値: エラーコード

**例:**
```javascript
const bufferPtr = wasmModule.allocate(256);
const size = wasmModule.create_dlt_message(bufferPtr, 256);
if (size > 0) {
    const message = new Uint8Array(wasmModule.memory.buffer, bufferPtr, size);
    console.log('生成されたメッセージ:', Array.from(message).map(b => b.toString(16).padStart(2, '0')).join(''));
}
```

### メッセージ解析

#### `analyze_dlt_message(buffer_ptr, buffer_len) -> *mut u8`
**r19-11の`DltHeaderParser`を使用**してDLTメッセージを解析し、詳細情報を返します。

内部で`DltHeaderParser::parse_message()`を呼び出し、標準準拠のパーサーでメッセージを解析します。

**パラメータ:**
- `buffer_ptr`: 解析するメッセージへのポインタ
- `buffer_len`: メッセージサイズ

**戻り値:**
- 解析結果構造体へのポインタ (32バイト)
- NULL: エラー

**解析結果構造体 (32バイト):**
```c
struct AnalysisResult {
    u16 total_len;       // 0-1: 総メッセージ長
    u16 header_len;      // 2-3: ヘッダ長
    u16 payload_len;     // 4-5: ペイロード長
    u16 payload_offset;  // 6-7: ペイロードのオフセット
    u8  msg_type;        // 8: MSIN (メッセージタイプ情報)
    u8  log_level;       // 9: ログレベル (1-6)
    u8  has_serial;      // 10: シリアルヘッダ有無
    u8  has_ecu;         // 11: ECU ID有無
    u8  ecu_id[4];       // 12-15: ECU ID (例: "ECU1" = 0x45435531)
    u8  app_id[4];       // 16-19: アプリケーションID (例: "LOG\0" = 0x4c4f4700)
    u8  ctx_id[4];       // 20-23: コンテキストID (例: "TEST" = 0x54455354)
    u8  mstp;            // 24: メッセージタイプ (0=Log, 1=Trace, 2=Network, 3=Control)
    u8  is_verbose;      // 25: Verboseモードフラグ
    u8  reserved[6];     // 26-31: 予約
};
```

**ID形式について:**
- すべてのIDは**4バイト固定長**のASCII文字列
- 4文字未満の場合は**nullバイト（0x00）でパディング**
- 例:
  - `"ECU1"` → `[0x45, 0x43, 0x55, 0x31]`
  - `"LOG"` → `[0x4c, 0x4f, 0x47, 0x00]` (3文字なので1バイトnull)
  - `"TEST"` → `[0x54, 0x45, 0x53, 0x54]` (4文字なのでパディング不要)

**メッセージタイプ (MSTP):**
- `0`: **ログメッセージ** - ペイロード解析対象
- `1`: トレースメッセージ
- `2`: ネットワークメッセージ - ペイロード解析対象外
- `3`: コントロール/サービスメッセージ - ペイロード解析対象外

**ログレベル (Log Messageの場合):**
- `1`: Fatal
- `2`: Error
- `3`: Warn
- `4`: Info
- `5`: Debug
- `6`: Verbose

**例 (r19-11パーサー使用):**
```javascript
// メッセージをメモリに配置
const msgBytes = hexToBytes('3d01002e454355310000001e646c1a6d31024c4f470054455354');
const bufferPtr = wasmModule.allocate(msgBytes.length);
const buffer = new Uint8Array(wasmModule.memory.buffer, bufferPtr, msgBytes.length);
buffer.set(msgBytes);

// r19-11パーサーで解析実行
const resultPtr = wasmModule.analyze_dlt_message(bufferPtr, msgBytes.length);
if (resultPtr !== 0) {
    const result = new Uint8Array(wasmModule.memory.buffer, resultPtr, 32);
    
    // 結果の読み取り
    const totalLen = result[0] | (result[1] << 8);
    const payloadLen = result[4] | (result[5] << 8);
    const mstp = result[24];  // メッセージタイプ (MstpType::parse()で解析済み)
    const isVerbose = result[25];
    
    console.log('総長:', totalLen);
    console.log('ペイロード長:', payloadLen);
    console.log('メッセージタイプ:', mstp === 0 ? 'Log' : mstp === 3 ? 'Control' : 'Other');
    console.log('Verbose:', isVerbose ? 'Yes' : 'No');
    
    // r19-11パーサーが自動的にすべてを解析
    // - シリアルヘッダの検出
    // - 標準ヘッダのパース
    // - 拡張ヘッダのパース (存在する場合)
    // - MSTPの正確な識別
    
    // メモリ解放
    wasmModule.deallocate(bufferPtr);
    wasmModule.deallocate(resultPtr);
}
```

### Verboseペイロード解析 (Logメッセージのみ)

#### `format_verbose_payload(buffer_ptr, buffer_len, payload_offset, payload_len, mstp) -> i32`
ログメッセージのVerboseペイロードを解析・フォーマットします。

**重要:** MSTP=0 (Logメッセージ) のみ解析します。Service/Network/Traceメッセージはエラーを返します。

**パラメータ:**
- `buffer_ptr`: メッセージバッファ
- `buffer_len`: バッファサイズ
- `payload_offset`: ペイロードの開始オフセット
- `payload_len`: ペイロード長
- `mstp`: メッセージタイプ (0=Log)

**戻り値:**
- 正の値: フォーマット済み文字列の長さ
- 負の値: エラーコード

### ID抽出関数

#### `get_ecu_id(buffer_ptr, buffer_len) -> u32`
ECU IDを32ビット整数で取得します。

#### `get_app_id(buffer_ptr, buffer_len) -> u32`
アプリケーションIDを32ビット整数で取得します。

#### `get_context_id(buffer_ptr, buffer_len) -> u32`
コンテキストIDを32ビット整数で取得します。

### メモリ管理

#### `allocate(size) -> *mut u8`
指定サイズのメモリを確保します（8バイトアライメント）。

#### `deallocate(ptr)`
確保したメモリを解放します。

#### `reset_allocator()`
アロケータをリセット（全メモリクリア）します。

#### `get_heap_usage() -> usize`
現在のヒープ使用量を取得します。

#### `get_heap_capacity() -> usize`
総ヒープ容量を取得します（8192バイト）。

## 📊 テストページの使い方

`examples/test_messages.html` でメッセージ解析をテストできます：

### 機能:
1. **複数メッセージの一括解析**: Hex文字列を1行1メッセージで入力
2. **メッセージタイプ別表示**: Log/Control/Network/Traceを色分け表示
3. **統計表示**: タイプ別メッセージ数を集計
4. **詳細情報表示**: ECU/App/Context ID、ログレベルなど

### サンプルデータ:
```
# Log Message (ECU1, LOG, TEST)
3d01002e454355310000001e646c1a6d31024c4f470054455354

分解:
  3d          - HTYP (標準ヘッダ)
  01          - MCNT (メッセージカウンタ)
  002e        - LEN (46バイト)
  45435531    - ECU ID = "ECU1"
  0000001e    - Session ID
  646c1a6d    - Timestamp
  31          - MSIN (拡張ヘッダ)
  02          - NOAR (引数数)
  4c4f4700    - App ID = "LOG\0" (表示: "LOG")
  54455354    - Context ID = "TEST"
  
# Control Message (ECU1, DLTD, INTM)
3d040074454355310000000e648c89ab4101444c5444494e544d

分解:
  3d          - HTYP
  04          - MCNT
  0074        - LEN (116バイト)
  45435531    - ECU ID = "ECU1"
  0000000e    - Session ID
  648c89ab    - Timestamp  41          - MSIN (Control message)
  01          - NOAR
  444c5444    - App ID = "DLTD"
  494e544d    - Context ID = "INTM"
```

## 🔍 実装の特徴

### R19-11準拠のパーサー使用
- **`DltHeaderParser`**: DLT Protocol R19-11の標準パーサーを使用
- `src/r19_11/header.rs`で実装された`parse_message()`メソッドを活用
- 手動バイト解析ではなく、テスト済みの公式パーサーで信頼性向上

### R19-11構造体の活用
解析時に以下のr19-11構造体を使用:
- `DltMessage`: 完全な解析済みメッセージ構造
- `DltStandardHeader`: 標準ヘッダ情報
- `DltExtendedHeader`: 拡張ヘッダ情報
- `MstpType::parse()`: メッセージタイプの標準パーサー
- `DltHTYP`: HTYPフィールドの構造化表現

### メッセージタイプ判別
MSINバイトから以下を抽出:
- **Bit 7-4 (MSTP)**: メッセージタイプ識別
  - `0000` = Log (ペイロード解析あり)
  - `0001` = Application Trace
  - `0010` = Network Trace (ペイロード解析なし)
  - `0011` = Control/Service (ペイロード解析なし)
- **Bit 3-1 (MTIN)**: Logメッセージのログレベル
- **Bit 0 (VERB)**: Verboseモードフラグ

### no_std設計 (r19-11準拠)
- r19-11モジュールの`DltHeaderParser`を使用
- ヒープアロケーション不使用（パーサー自体はスタックのみ使用）
- スタックベースの固定バッファ (8192バイト) - メモリ管理用
- 組み込みシステムでも動作可能
- AUTOSAR準拠のエラーハンドリング

## 💡 実用例

### WebSocketからのストリーミング解析

```javascript
const ws = new WebSocket('ws://localhost:8765');
ws.binaryType = 'arraybuffer';

ws.onmessage = (event) => {
    const data = new Uint8Array(event.data);
    
    // WASMメモリにコピー
    const ptr = wasmModule.allocate(data.length);
    const buffer = new Uint8Array(wasmModule.memory.buffer, ptr, data.length);
    buffer.set(data);
    
    // 解析
    const resultPtr = wasmModule.analyze_dlt_message(ptr, data.length);
    if (resultPtr !== 0) {
        const result = new Uint8Array(wasmModule.memory.buffer, resultPtr, 32);
        const mstp = result[24];
        
        // Logメッセージのみ詳細表示
        if (mstp === 0) {
            const appId = String.fromCharCode(...result.slice(16, 20));
            const ctxId = String.fromCharCode(...result.slice(20, 24));
            console.log(`[${appId}:${ctxId}] Log message received`);
        }
        
        wasmModule.deallocate(resultPtr);
    }
    wasmModule.deallocate(ptr);
};
```

## 🐛 エラーコード

- `-1` (ERROR_NULL_POINTER): NULLポインタ
- `-2` (ERROR_BUFFER_TOO_SMALL): バッファ不足
- `-3` (ERROR_INVALID_FORMAT): 無効なフォーマット
- `-4` (ERROR_OUT_OF_MEMORY): メモリ不足

## 📚 参考資料

- [AUTOSAR DLT Specification R19-11](https://www.autosar.org/)
- プロジェクトドキュメント: `.github/copilot-instructions.md`
- テストコード: `tests/r19_11_it.rs`
- r19-11パーサー実装: `src/r19_11/header.rs` (`DltHeaderParser`)
- r19-11ヘッダ構造: `src/r19_11/header.rs` (`DltMessage`, `DltStandardHeader`, `DltExtendedHeader`)

## 🎯 今後の拡張予定

- [ ] R22-11サポート (`src/r22_11/`)
- [ ] サービスメッセージのデコード実装
- [ ] ストリーミングパーサー実装
- [ ] より高度なVerboseペイロード解析
