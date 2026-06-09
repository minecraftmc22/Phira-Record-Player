//! .phirarec binary record parser — tphira-mp server format.
//!
//! Two formats exist in the wild:
//!
//! ### tphira-mp format (this parser)
//! ```
//! Header (13 bytes):
//!   magic:     [u8;8] "PHIRAREC"
//!   version:   u32 LE (always 1)
//!   compr:     u8      (0=none, 1=zstd, 2=deflate)
//!
//! Payload (compressed, then):
//!   record_id:     i32 LE
//!   timestamp_ms:  i64 LE
//!   chart_id:      i32 LE
//!   chart_name:    string (ULEB128 len + UTF-8)
//!   user_id:       i32 LE
//!   user_name:     string (ULEB128 len + UTF-8)
//!   touch_frames:  array (ULEB128 count, then items)
//!     time:        f32 LE
//!     points:      array (ULEB128 count, then items)
//!       id:        i8
//!       x:         f16 LE (u16 bits)
//!       y:         f16 LE (u16 bits)
//!   judge_events:  array (ULEB128 count, then items)
//!     time:        f32 LE
//!     line_id:     i32 LE   ← SIGNED (compat with old replay files)
//!     note_id:     i32 LE   ← SIGNED
//!     judgement:   u8
//! ```
//!
//! ### Phira client format (encrypted — NOT parsable here)
//! Header: PHIRAREC + u32 LE version + u8 marker + u8 flag + u64 nonce + [encrypted].
//! The closed-source `inner` crate encrypts the payload after `nonce`.
//! We detect this and return a clear "encrypted" error instead of crashing.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const MAGIC: &[u8; 8] = b"PHIRAREC";
const HEADER_SIZE: usize = 13; // magic(8) + version(4) + compression(1)

const COMPRESSION_NONE: u8 = 0;
const COMPRESSION_ZSTD: u8 = 1;
const COMPRESSION_DEFLATE: u8 = 2;

// ---------------------------------------------------------------------------
// Low‑level binary helpers (ULEB128, half‑precision float, fixed‑width reads)
// ---------------------------------------------------------------------------

/// Read an unsigned little‑endian ULEB128 value.
fn read_uleb(data: &[u8], pos: &mut usize) -> Result<u64> {
    let mut result: u64 = 0;
    let mut shift = 0u32;
    loop {
        if *pos >= data.len() {
            bail!("unexpected end of data while reading ULEB128");
        }
        let byte = data[*pos];
        *pos += 1;
        result |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            bail!("ULEB128 value too large");
        }
    }
    Ok(result)
}

/// Read a ULEB128 followed by that many UTF‑8 bytes.
fn read_string(data: &[u8], pos: &mut usize) -> Result<String> {
    let len = read_uleb(data, pos)? as usize;
    if *pos + len > data.len() {
        bail!("string length {len} exceeds remaining data");
    }
    let s = String::from_utf8_lossy(&data[*pos..*pos + len]).into_owned();
    *pos += len;
    Ok(s)
}

macro_rules! read_le {
    ($data:expr, $pos:expr, $ty:ty) => {{
        const SIZE: usize = std::mem::size_of::<$ty>();
        let p = &mut *$pos;
        if *p + SIZE > $data.len() {
            bail!(concat!(
                "unexpected end of data while reading ",
                stringify!($ty)
            ));
        }
        let bytes: [u8; SIZE] = $data[*p..*p + SIZE].try_into().unwrap();
        let val = <$ty>::from_le_bytes(bytes);
        *p += SIZE;
        val
    }};
}

/// Convert IEEE‑754 half‑precision (16‑bit) bits → f32.
fn f16_to_f32(bits: u16) -> f32 {
    let sign = if bits & 0x8000 != 0 { -1.0_f32 } else { 1.0 };
    let exp = (bits >> 10) & 0x1F;
    let mant = bits & 0x3FF;
    if exp == 0 {
        sign * (mant as f32 / 1024.0) * 2.0_f32.powi(-14)
    } else if exp == 31 {
        if mant == 0 {
            sign * f32::INFINITY
        } else {
            f32::NAN
        }
    } else {
        sign * (1.0 + mant as f32 / 1024.0) * 2.0_f32.powi(exp as i32 - 15)
    }
}

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// A single touch point within a frame.
#[derive(Debug, Clone, Serialize)]
pub struct TouchPoint {
    /// Finger / pointer id (i8).
    pub id: i8,
    /// Normalised position: (x, y) ∈ [0, 1] range.
    pub x: f32,
    pub y: f32,
}

/// One frame of recorded touch input.
#[derive(Debug, Clone, Serialize)]
pub struct TouchFrame {
    /// Time in seconds since chart start.
    pub time: f32,
    /// Active touch points in this frame.
    pub points: Vec<TouchPoint>,
}

/// A single judge (hit) event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeEvent {
    /// Time in seconds since chart start.
    pub time: f32,
    /// Line index (lane).  SIGNED i32 for compat with old replay files.
    pub line_id: i32,
    /// Note identifier within the line.  SIGNED i32 for compat.
    pub note_id: i32,
    /// Judgement code: 0=Perfect, 1=Good, 2=Bad, 3=Miss, 4=HoldPerfect, 5=HoldGood
    pub judgement: u8,
}

/// Full parsed tphira-mp record header + replay data.
#[derive(Debug, Clone, Serialize)]
pub struct PhiraRecord {
    /// Binary format version (always 1).
    pub version: u32,
    /// Compression algorithm used (0=none, 1=zstd, 2=deflate).
    pub compression: u8,
    /// Record ID (from the Phira record system).
    pub record_id: i32,
    /// Unix timestamp in milliseconds when the recording started.
    pub timestamp_ms: i64,
    /// Chart identifier.
    pub chart_id: i32,
    /// Human‑readable chart name.
    pub chart_name: String,
    /// User / account id.
    pub user_id: i32,
    /// User display name.
    pub user_name: String,
    /// Recorded touch frames (input replay).
    pub touches: Vec<TouchFrame>,
    /// Judge / hit events.
    pub judges: Vec<JudgeEvent>,
}

// ---------------------------------------------------------------------------
// Decompression
// ---------------------------------------------------------------------------

/// Decompress the payload using the algorithm specified in the header.
fn decompress_payload(compression: u8, payload: &[u8]) -> Result<Vec<u8>> {
    match compression {
        COMPRESSION_NONE => Ok(payload.to_vec()),
        COMPRESSION_ZSTD => {
            use ruzstd::decoding::StreamingDecoder;
            use std::io::Read;
            let mut decoder = StreamingDecoder::new(payload)
                .context("ZSTD decompression failed")?;
            let mut out = Vec::new();
            decoder.read_to_end(&mut out)
                .context("ZSTD decompression failed")?;
            Ok(out)
        }
        COMPRESSION_DEFLATE => {
            use flate2::read::DeflateDecoder;
            use std::io::Read;
            let mut decoder = DeflateDecoder::new(payload);
            let mut out = Vec::new();
            decoder
                .read_to_end(&mut out)
                .context("DEFLATE decompression failed")?;
            Ok(out)
        }
        other => bail!("unsupported compression algorithm: {other}"),
    }
}

// ---------------------------------------------------------------------------
// Format detection
// ---------------------------------------------------------------------------

/// Check whether this looks like a Phira client encrypted file.
///
/// Phira client format: PHIRAREC + u32 LE version + u8 marker + u8 flag_byte + u64 nonce + ...
/// The nonce is followed by encrypted data where the "reserved" (4 bytes after nonce) is non-zero.
///
/// tphira-mp format: PHIRAREC + u32 LE version + u8 compression (0/1/2) + compressed payload.
///
/// The first 12 bytes are identical (magic + version).  Byte 12 distinguishes them:
/// - tphira-mp: 0x00, 0x01, or 0x02 (compression alg)
/// - Phira client: 0x01 (version_marker, coincidentally same as COMPRESSION_ZSTD)
///
/// Because the byte values can overlap, the reliable test is: try to decompress.
/// If that fails *and* there are at least 22 bytes with a plausible Phira-client header,
/// it's encrypted.
fn detect_format(data: &[u8]) -> FormatDetection {
    if data.len() < HEADER_SIZE {
        return FormatDetection::TooSmall;
    }
    if &data[..8] != MAGIC {
        return FormatDetection::BadMagic;
    }

    let byte12 = data[12];

    // If byte12 is clearly a compression type, try to decompress it.
    if byte12 <= COMPRESSION_DEFLATE {
        let payload = &data[HEADER_SIZE..];
        match decompress_payload(byte12, payload) {
            Ok(_) => return FormatDetection::TphiraMp { compression: byte12 },
            Err(_) => {
                // Decompression failed — could be Phira client encrypted
            }
        }
    }

    // Check for Phira client encrypted format:
    // Need at least: magic(8) + version(4) + marker(1) + flag(1) + nonce(8) + reserved(4) = 26
    if data.len() >= 26 {
        let version_marker = data[12];
        let _flag_byte = data[13];
        // reserved is at bytes 22-25 AFTER nonce (bytes 14-21)
        let reserved_raw =
            u32::from_le_bytes(data[22..26].try_into().unwrap());

        // In Phira client encrypted format, version_marker is 1 and reserved is non-zero
        // (because it's encrypted, not actually a reserved field).
        if version_marker == 1 && reserved_raw != 0 {
            return FormatDetection::PhiraClientEncrypted;
        }
    }

    FormatDetection::Unknown
}

#[derive(Debug, PartialEq)]
enum FormatDetection {
    TphiraMp { compression: u8 },
    PhiraClientEncrypted,
    TooSmall,
    BadMagic,
    Unknown,
}

// ---------------------------------------------------------------------------
// Content parsing (tphira-mp decompressed payload)
// ---------------------------------------------------------------------------

/// Parse touch frames from decompressed payload.
fn parse_touch_frames(data: &[u8], pos: &mut usize) -> Result<Vec<TouchFrame>> {
    let count = read_uleb(data, pos)? as usize;
    let mut frames = Vec::with_capacity(count.min(32768));
    for _ in 0..count {
        let time = read_le!(data, pos, f32);
        let n_points = read_uleb(data, pos)? as usize;
        let mut points = Vec::with_capacity(n_points.min(10));
        for _ in 0..n_points {
            let id = read_le!(data, pos, i8);
            let x_bits = read_le!(data, pos, u16);
            let y_bits = read_le!(data, pos, u16);
            points.push(TouchPoint {
                id,
                x: f16_to_f32(x_bits),
                y: f16_to_f32(y_bits),
            });
        }
        frames.push(TouchFrame { time, points });
    }
    Ok(frames)
}

/// Parse judge events from decompressed payload.
fn parse_judge_events(data: &[u8], pos: &mut usize) -> Result<Vec<JudgeEvent>> {
    let count = read_uleb(data, pos)? as usize;
    let mut events = Vec::with_capacity(count.min(65536));
    for _ in 0..count {
        let time = read_le!(data, pos, f32);
        let line_id = read_le!(data, pos, i32); // SIGNED (tphira-mp compat)
        let note_id = read_le!(data, pos, i32); // SIGNED
        let judgement = read_le!(data, pos, u8);
        events.push(JudgeEvent {
            time,
            line_id,
            note_id,
            judgement,
        });
    }
    Ok(events)
}

/// Parse the decompressed tphira-mp content.
fn parse_tphira_content(data: &[u8]) -> Result<PhiraRecord> {
    let mut pos = 0usize;

    let record_id = read_le!(data, &mut pos, i32);
    let timestamp_ms = read_le!(data, &mut pos, i64);
    let chart_id = read_le!(data, &mut pos, i32);
    let chart_name = read_string(data, &mut pos)?;
    let user_id = read_le!(data, &mut pos, i32);
    let user_name = read_string(data, &mut pos)?;
    let touches = parse_touch_frames(data, &mut pos)?;
    let judges = parse_judge_events(data, &mut pos)?;

    Ok(PhiraRecord {
        version: 1, // always 1 for known tphira-mp files
        compression: COMPRESSION_ZSTD, // caller fills actual value
        record_id,
        timestamp_ms,
        chart_id,
        chart_name,
        user_id,
        user_name,
        touches,
        judges,
    })
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Parse a .phirarec file (tphira-mp format).
///
/// Automatically detects format:
/// - tphira-mp (unencrypted, with optional ZSTD/DEFLATE compression): parsed normally
/// - Phira client (encrypted): returns a clear "此文件已加密" error
/// - Unknown/broken: returns an appropriate error
pub fn parse_record(data: &[u8]) -> Result<PhiraRecord> {
    match detect_format(data) {
        FormatDetection::TphiraMp { compression } => {
            let payload = &data[HEADER_SIZE..];
            let decompressed = decompress_payload(compression, payload)?;
            parse_tphira_content(&decompressed)
                .map(|mut rec| {
                    rec.compression = compression;
                    rec
                })
        }
        FormatDetection::PhiraClientEncrypted => {
            bail!("此文件已加密，需要 Phira 客户端才能解密。");
        }
        FormatDetection::TooSmall => {
            bail!("file too small for .phirarec header (need at least {} bytes)", HEADER_SIZE);
        }
        FormatDetection::BadMagic => {
            bail!("not a valid .phirarec file (missing PHIRAREC magic)");
        }
        FormatDetection::Unknown => {
            bail!("unknown .phirarec format — file may be corrupted or from an unsupported version");
        }
    }
}

/// Quick validity check without full parsing.
pub fn is_phirarec(data: &[u8]) -> bool {
    data.len() >= 8 && &data[..8] == MAGIC
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f16_zero() {
        assert_eq!(f16_to_f32(0x0000), 0.0);
    }

    #[test]
    fn test_f16_one() {
        assert_eq!(f16_to_f32(0x3C00), 1.0);
    }

    #[test]
    fn test_f16_neg_two() {
        assert_eq!(f16_to_f32(0xC000), -2.0);
    }

    #[test]
    fn test_uleb_small() {
        let data = [0x2A]; // 42
        let mut pos = 0;
        assert_eq!(read_uleb(&data, &mut pos).unwrap(), 42);
        assert_eq!(pos, 1);
    }

    #[test]
    fn test_uleb_two_byte() {
        let data = [0xE5, 0x8E]; // 0x1E65 = 7781
        let mut pos = 0;
        assert_eq!(read_uleb(&data, &mut pos).unwrap(), 0x1E65);
        assert_eq!(pos, 2);
    }

    #[test]
    fn test_invalid_magic() {
        let data = b"NOTREC\0\0\0\0\0\0\0\0";
        assert!(parse_record(data).is_err());
    }

    #[test]
    fn test_is_phirarec() {
        let mut data = vec![0u8; 13];
        data[..8].copy_from_slice(b"PHIRAREC");
        data[8..12].copy_from_slice(&1u32.to_le_bytes());
        assert!(is_phirarec(&data));
    }

    /// Build a minimal tphira-mp record with no compression, no touches, no judges.
    #[test]
    fn test_parse_minimal_tphira() {
        // Build content
        let mut content = Vec::new();
        content.extend(&42i32.to_le_bytes()); // record_id
        content.extend(&1234567890i64.to_le_bytes()); // timestamp_ms
        content.extend(&100i32.to_le_bytes()); // chart_id
        content.push(3u8); // chart_name len
        content.extend(b"SP."); // chart_name
        content.extend(&999i32.to_le_bytes()); // user_id
        content.push(4u8); // user_name len
        content.extend(b"Test"); // user_name
        content.push(0u8); // touch_frames count = 0
        content.push(0u8); // judge_events count = 0

        // Build file: header + uncompressed payload
        let mut data = Vec::new();
        data.extend(MAGIC);
        data.extend(&1u32.to_le_bytes()); // version
        data.push(COMPRESSION_NONE); // compression
        data.extend(&content);

        let record = parse_record(&data).unwrap();
        assert_eq!(record.version, 1);
        assert_eq!(record.compression, COMPRESSION_NONE);
        assert_eq!(record.record_id, 42);
        assert_eq!(record.timestamp_ms, 1234567890);
        assert_eq!(record.chart_id, 100);
        assert_eq!(record.chart_name, "SP.");
        assert_eq!(record.user_id, 999);
        assert_eq!(record.user_name, "Test");
        assert!(record.touches.is_empty());
        assert!(record.judges.is_empty());
    }

    /// Build a tphira-mp record with ZSTD compression, no touches, no judges.
    #[test]
    fn test_parse_minimal_zstd() {
        let mut content = Vec::new();
        content.extend(&42i32.to_le_bytes());
        content.extend(&0i64.to_le_bytes());
        content.extend(&1i32.to_le_bytes());
        content.push(1u8);
        content.extend(b"A");
        content.extend(&0i32.to_le_bytes());
        content.push(1u8);
        content.extend(b"B");
        content.push(0u8); // no touches
        content.push(0u8); // no judges

        // Pre-compressed ZSTD payload (ruzstd is decoder-only, can't encode)
        let compressed: &[u8] = &[
            0x28, 0xb5, 0x2f, 0xfd, 0x20, 0x1a, 0xb5, 0x00, 0x00, 0x80,
            0x2a, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x41, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x42, 0x00, 0x00, 0x01, 0x00, 0x1b, 0xc0,
            0x02,
        ];

        let mut data = Vec::new();
        data.extend(MAGIC);
        data.extend(&1u32.to_le_bytes());
        data.push(COMPRESSION_ZSTD);
        data.extend(&compressed);

        let record = parse_record(&data).unwrap();
        assert_eq!(record.record_id, 42);
        assert_eq!(record.chart_name, "A");
        assert_eq!(record.user_name, "B");
    }

    /// Phira client encrypted file should trigger a clear error.
    #[test]
    fn test_encrypted_detection() {
        let mut data = Vec::new();
        data.extend(MAGIC);
        data.extend(&1u32.to_le_bytes()); // version
        data.push(1u8); // version_marker
        data.push(0x28u8); // flag_byte
        data.extend(&12345u64.to_le_bytes()); // nonce
        data.extend(&0xDEADBEEFu32.to_le_bytes()); // reserved (non-zero → encrypted)
        data.extend(&0i64.to_le_bytes()); // more random encrypted-looking data
        data.extend(&0u32.to_le_bytes());
        data.push(0u8);

        let err = parse_record(&data).unwrap_err();
        assert!(
            err.to_string().contains("已加密"),
            "expected encryption error, got: {err}"
        );
    }
}
