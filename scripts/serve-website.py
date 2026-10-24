#!/usr/bin/env python3
"""Serve only the built public website on the local loopback interface."""

import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


class Handler(SimpleHTTPRequestHandler):
    extensions_map = {**SimpleHTTPRequestHandler.extensions_map, ".wasm": "application/wasm", ".js": "text/javascript"}

    def end_headers(self):
        self.send_header("Cache-Control", "no-cache")
        self.send_header("X-Content-Type-Options", "nosniff")
        self.send_header("Referrer-Policy", "same-origin")
        super().end_headers()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=8787)
    args = parser.parse_args()
    public = Path(__file__).resolve().parents[1] / "target/website"
    if not (public / "index.html").is_file():
        parser.error("Build first: python3 scripts/build-website.py")
    handler = partial(Handler, directory=str(public))
    with ThreadingHTTPServer(("127.0.0.1", args.port), handler) as server:
        print(f"DiggerChain preview: http://127.0.0.1:{args.port}", flush=True)
        print("Only target/website is served. Press Ctrl+C to stop.", flush=True)
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            pass


if __name__ == "__main__":
    main()
