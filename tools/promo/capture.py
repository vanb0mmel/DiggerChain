#!/usr/bin/env python3
"""Record real terminal output for the promo; never read the player's own saves."""

import codecs
import fcntl
import json
import os
from pathlib import Path
import pty
import select
import struct
import subprocess
import termios
import time


PROJECT = Path(__file__).resolve().parent
ROOT = PROJECT.parents[1]


def main():
    route = json.loads(subprocess.check_output([str(PROJECT / "target/release/capture-route")]))
    recording = PROJECT / "recordings/gameplay.json"
    if recording.exists():
        raise SystemExit("Recording already exists; archive it before capturing another take.")
    temporary = ROOT / "target/promo-capture"
    temporary.mkdir(parents=True, exist_ok=True)
    master, slave = pty.openpty()
    cols, rows = 100, 34
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))
    env = {**os.environ, "TERM": "xterm-256color", "COLORTERM": "truecolor"}
    process = subprocess.Popen([
        str(ROOT / "target/release/diggerchain"), "--new", "--seed", "0",
        "--save", str(temporary / "expedition.json"),
    ], stdin=slave, stdout=slave, stderr=slave, env=env, start_new_session=True)
    os.close(slave)
    start = time.monotonic()
    events, markers = [], []
    decoder = codecs.getincrementaldecoder("utf-8")("strict")

    def drain(seconds):
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            ready, _, _ = select.select([master], [], [], min(0.02, max(0, deadline - time.monotonic())))
            if not ready:
                continue
            try:
                data = os.read(master, 65536)
            except OSError:
                break
            if not data:
                break
            text = decoder.decode(data)
            if text:
                events.append({"time": round(time.monotonic() - start, 4), "data": text})

    try:
        drain(0.8)
        view = "glyphs"
        for index, item in enumerate(route["inputs"]):
            if process.poll() is not None:
                raise RuntimeError("Game exited before the recording route completed.")
            if item["floor"] == 1 and view == "glyphs":
                os.write(master, b"g")
                drain(0.25)
                view = "sprites"
            markers.append({"time": round(time.monotonic() - start, 4), "view": view, **item})
            os.write(master, item["key"].encode())
            drain(0.14 if item["action"] != 10 else 0.55)
        drain(0.8)
        duration = time.monotonic() - start
        os.write(master, b"q")
        drain(0.25)
        process.wait(timeout=5)
        if process.returncode != 0:
            raise RuntimeError(f"Game exited with code {process.returncode}.")
    finally:
        if process.poll() is None:
            process.terminate()
            process.wait(timeout=5)
        os.close(master)
    recording.parent.mkdir(parents=True, exist_ok=True)
    recording.write_text(json.dumps({
        "source": "Native DiggerChain alpha, captured through a real PTY; no simulated footage.",
        "cols": cols, "rows": rows, "seed": 0, "duration": duration,
        "events": events, "markers": markers,
    }, ensure_ascii=False, separators=(",", ":")) + "\n")
    print(f"Captured {len(markers)} inputs, {duration:.2f}s: {recording}")


if __name__ == "__main__":
    main()
