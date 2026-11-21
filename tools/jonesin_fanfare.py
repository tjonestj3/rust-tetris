#!/usr/bin/env python3
"""Makes assets/sounds/jonesin.wav, the fanfare for the JONES'IN! (Tetris) celebration.

A quick rising chiptune arpeggio into a bright held chord with a little vibrato, and a few
sparkles on top: about a second, timed so the chord lands as the rainbow text finishes popping
in (src/app.rs, draw_tetris_celebration). Standard library only, and deterministic, so running
it again gives the same file:

    python3 tools/jonesin_fanfare.py
"""

import math
import random
import struct
import wave
from pathlib import Path

RATE = 44_100
OUT = Path(__file__).resolve().parent.parent / "assets" / "sounds" / "jonesin.wav"


def note(name: str) -> float:
    """Frequency of a note like 'C5' or 'G#4' (A4 = 440 Hz)."""
    steps = {"C": -9, "D": -7, "E": -5, "F": -4, "G": -2, "A": 0, "B": 2}
    letter, rest = name[0], name[1:]
    sharp = rest.startswith("#")
    octave = int(rest[1:] if sharp else rest)
    semis = steps[letter] + (1 if sharp else 0) + 12 * (octave - 4)
    return 440.0 * 2 ** (semis / 12)


def pulse(phase: float, harmonics: int = 7) -> float:
    """A soft square wave: odd harmonics only, so it's chiptune-ish without the harsh buzz."""
    return sum(math.sin(phase * k) / k for k in range(1, harmonics + 1, 2)) * (4 / math.pi) * 0.6


def tone(buf, start, length, freq, volume, attack=0.005, decay=6.0, vibrato=0.0, wave_fn=pulse):
    first = int(start * RATE)
    for i in range(int(length * RATE)):
        t = i / RATE
        env = min(1.0, t / attack) * math.exp(-decay * t)
        # Fade the last 20 ms so nothing clicks off.
        env *= min(1.0, (length - t) / 0.02)
        wobble = 1 + vibrato * math.sin(2 * math.pi * 6 * t) * min(1.0, t / 0.15)
        phase = 2 * math.pi * freq * wobble * t
        j = first + i
        if j < len(buf):
            buf[j] += volume * env * wave_fn(phase)


def main() -> None:
    length = 1.1
    buf = [0.0] * int(length * RATE)

    # Rising arpeggio: G4 C5 E5 G5, 65 ms apart.
    for k, n in enumerate(["G4", "C5", "E5", "G5"]):
        tone(buf, 0.065 * k, 0.16, note(n), 0.32, decay=9.0)

    # The chord lands: C major up high, a C4 underneath, held with vibrato.
    land = 0.26
    for n, vol in [("C6", 0.26), ("E6", 0.2), ("G6", 0.18), ("C4", 0.22)]:
        tone(buf, land, length - land, note(n), vol, attack=0.008, decay=2.6, vibrato=0.006)

    # Sparkles: short high sine pings scattered over the chord.
    rng = random.Random(1987)
    for _ in range(9):
        at = land + 0.05 + rng.random() * 0.55
        freq = rng.choice([note("C7"), note("E7"), note("G7"), note("C8")])
        tone(buf, at, 0.09, freq, 0.07, attack=0.002, decay=30.0, wave_fn=math.sin)

    # Normalise to -3 dBFS.
    peak = max(abs(s) for s in buf)
    scale = 0.707 / peak
    frames = b"".join(struct.pack("<h", int(max(-1.0, min(1.0, s * scale)) * 32767)) for s in buf)

    with wave.open(str(OUT), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(RATE)
        w.writeframes(frames)
    print(f"Wrote {OUT} ({length:.2f}s)")


if __name__ == "__main__":
    main()
