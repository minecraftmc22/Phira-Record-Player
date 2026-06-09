#!/usr/bin/env python3
"""
Decoder for .phirarec binary files.
Format: PHIRAREC + u32(version=1) + BinaryData(RecordData)
"""

import struct
import sys
import os
from datetime import datetime, timezone


def uleb(data: bytes, pos: int):
    """Read ULEB128 from bytes starting at pos. Returns (value, new_position)."""
    result = 0
    shift = 0
    while True:
        byte = data[pos]
        pos += 1
        result |= (byte & 0x7F) << shift
        if not (byte & 0x80):
            break
        shift += 7
    return result, pos


def f16_to_f32(raw: int) -> float:
    """Convert IEEE 754 half-precision (16-bit) to float."""
    sign = -1 if (raw >> 15) else 1
    exp = (raw >> 10) & 0x1F
    mant = raw & 0x3FF
    if exp == 0:
        return sign * (mant / 1024.0) * (2 ** -14)
    elif exp == 31:
        return float('nan') if mant == 0 else float('inf') * sign
    else:
        return sign * (1 + mant / 1024.0) * (2 ** (exp - 15))


def parse_touch_frames(data: bytes, pos: int) -> tuple:
    """Parse Vec<TouchFrame>. Returns (frames, new_pos)."""
    count, pos = uleb(data, pos)
    frames = []
    for _ in range(count):
        time = struct.unpack_from('<f', data, pos)[0]
        pos += 4
        n_points, pos = uleb(data, pos)
        points = []
        for _ in range(n_points):
            pid = struct.unpack_from('<b', data, pos)[0]
            pos += 1
            x_raw = struct.unpack_from('<H', data, pos)[0]
            pos += 2
            y_raw = struct.unpack_from('<H', data, pos)[0]
            pos += 2
            points.append({"id": pid, "pos": {"x": round(f16_to_f32(x_raw), 4), "y": round(f16_to_f32(y_raw), 4)}})
        frames.append({"time": round(time, 4), "points": points})
    return frames, pos


def parse_judge_events(data: bytes, pos: int) -> tuple:
    """Parse Vec<JudgeEvent>. Returns (events, new_pos)."""
    JUDGEMENTS = {0: "Perfect", 1: "Good", 2: "Bad", 3: "Miss", 4: "HoldPerfect", 5: "HoldGood"}
    count, pos = uleb(data, pos)
    events = []
    for _ in range(count):
        time = struct.unpack_from('<f', data, pos)[0]
        pos += 4
        line_id = struct.unpack_from('<I', data, pos)[0]
        pos += 4
        note_id = struct.unpack_from('<I', data, pos)[0]
        pos += 4
        judgement = data[pos]
        pos += 1
        events.append({
            "time": round(time, 4),
            "line_id": line_id,
            "note_id": note_id,
            "judgement": JUDGEMENTS.get(judgement, f"Unknown({judgement})"),
        })
    return events, pos


def parse_string(data: bytes, pos: int) -> tuple:
    """Parse ULEB-length-prefixed UTF-8 string. Returns (string, new_pos)."""
    length, pos = uleb(data, pos)
    s = data[pos:pos + length].decode('utf-8', errors='replace')
    pos += length
    return s, pos


def parse_record(data: bytes) -> dict:
    """Parse the RecordData from body bytes."""
    result = {}
    pos = 0

    # Field 1: u8 - record version/type marker
    result["version_marker"] = data[pos]
    pos += 1

    # Field 2: u8 - unknown flag/type byte (observed as 0x28=40 in tested files)
    result["flag_byte"] = data[pos]
    pos += 1

    # Field 3: u64 LE - nonce/hash
    result["nonce"] = struct.unpack_from('<Q', data, pos)[0]
    pos += 8

    # Field 4: u32 LE - reserved (always 0)
    result["reserved"] = struct.unpack_from('<I', data, pos)[0]
    pos += 4

    # Field 5: i64 LE - timestamp in milliseconds
    ts_ms = struct.unpack_from('<q', data, pos)[0]
    pos += 8
    if ts_ms > 0:
        try:
            result["timestamp"] = datetime.fromtimestamp(ts_ms / 1000.0, tz=timezone.utc).isoformat()
        except (OSError, ValueError):
            result["timestamp"] = f"invalid({ts_ms})"
    else:
        result["timestamp"] = None

    # Field 6: u32 LE - chart_id
    result["chart_id"] = struct.unpack_from('<I', data, pos)[0]
    pos += 4

    # Field 7: String - chart name
    result["chart_name"], pos = parse_string(data, pos)

    # Field 8: i32 - score
    result["score"] = struct.unpack_from('<i', data, pos)[0]
    pos += 4

    # Diagnostic: show bytes around score position to determine field order
    print(f"\n  --- After score (pos={pos}) ---")
    for start in range(max(0, pos-8), min(len(data), pos+64), 16):
        chunk = data[start:start+16]
        hex_str = ' '.join(f'{b:02x}' for b in chunk)
        ascii_str = ''.join(chr(b) if 32 <= b < 127 else '.' for b in chunk)
        marker = " <--" if start <= pos < start+16 else ""
        print(f"  {start:5d}: {hex_str:<48s} {ascii_str}{marker}")

    # Try to interpret next bytes
    next_bytes = data[pos:pos+12]
    print(f"\n  Next 12 bytes at pos {pos}: {' '.join(f'0x{b:02x}' for b in next_bytes)}")
    if next_bytes[0] < 0x80:
        print(f"  First byte < 0x80 → probably a ULEB count, value={next_bytes[0]}")
    print(f"  As f32 LE: {struct.unpack_from('<f', data, pos)[0]:.6f}")
    print(f"  As i32 LE: {struct.unpack_from('<i', data, pos)[0]}")
    
    # Try reading as string
    try:
        s, _ = parse_string(data, pos)
        print(f"  As string: '{s}'")
    except Exception as e:
        print(f"  As string ERROR: {e}")

    # Field 9: Vec<TouchFrame> - touches
    result["touches"], pos = parse_touch_frames(data, pos)

    # Field 10: String - player name
    result["player_name"], pos = parse_string(data, pos)

    # Field 11: Vec<JudgeEvent> - judges
    result["judges"], pos = parse_judge_events(data, pos)

    # Show remaining
    remaining = len(data) - pos
    if remaining > 0:
        print(f"\n  Remaining bytes after full parse: {remaining} at pos {pos}")
        print(f"  Tail: {' '.join(f'{b:02x}' for b in data[pos:pos+min(32, remaining)])}")

    return result


def main():
    if len(sys.argv) < 2:
        print("Usage: python parse_phirarec.py <file.phirarec>")
        sys.exit(1)

    filepath = sys.argv[1]
    with open(filepath, 'rb') as f:
        data = f.read()

    print(f"File: {filepath}")
    print(f"Size: {len(data)} bytes")

    if data[:8] != b'PHIRAREC':
        print("ERROR: Not a valid .phirarec file (missing PHIRAREC header)")
        sys.exit(1)

    version = struct.unpack_from('<I', data, 8)[0]
    print(f"Version: {version}")

    body = data[12:]
    print(f"Body size: {len(body)} bytes")

    result = parse_record(body)

    # Display results
    print(f"\n=== Header Fields ===")
    for k in ["version_marker", "flag_byte", "nonce", "reserved", "timestamp", "chart_id", "chart_name", "score", "player_name"]:
        if k in result:
            print(f"  {k}: {result[k]}")

    touches = result.get("touches", [])
    judges = result.get("judges", [])
    print(f"\n=== Record Data ===")
    print(f"  Touch frames: {len(touches)}")
    if touches:
        duration = max(f["time"] for f in touches) if touches else 0
        print(f"  Duration: {duration:.2f}s")
        print(f"  First 3 frames:")
        for f in touches[:3]:
            pts = ", ".join(f"#{p['id']}({p['pos']['x']:.3f},{p['pos']['y']:.3f})" for p in f["points"])
            print(f"    t={f['time']:.4f}  {pts}")

    print(f"\n  Judge events: {len(judges)}")
    if judges:
        total = len(judges)
        perfect = sum(1 for j in judges if j["judgement"] == "Perfect")
        good = sum(1 for j in judges if j["judgement"] == "Good")
        bad = sum(1 for j in judges if j["judgement"] == "Bad")
        miss = sum(1 for j in judges if j["judgement"] == "Miss")
        print(f"  Judgement breakdown: Perfect={perfect}, Good={good}, Bad={bad}, Miss={miss}")
        print(f"  First 5 events:")
        for j in judges[:5]:
            print(f"    t={j['time']:.4f}  line={j['line_id']}  note={j['note_id']}  {j['judgement']}")


if __name__ == '__main__':
    main()
