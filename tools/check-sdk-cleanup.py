"""Regression checks for SDK example cleanup when multiple HTTP operations fail."""
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import json
import os
import subprocess
import threading

ROOT = Path(__file__).resolve().parents[1]
COMMAND_TIMEOUT = float(os.environ.get("CRUCIBLE_SDK_COMMAND_TIMEOUT", "600"))
IDS = [f"00000000-0000-0000-0000-{index:012d}" for index in [1, 2]]


class Fixture(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def respond(self, status, body=None):
        content = b"" if body is None else json.dumps(body).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(content)))
        if body and "__type" in body:
            self.send_header("x-crucible-error", body["__type"])
        self.end_headers()
        self.wfile.write(content)

    def error(self, message):
        self.respond(400, {"__type": "ValidationException", "message": message, "reason": "cannotParse"})

    def do_POST(self):
        self.rfile.read(int(self.headers.get("Content-Length", 0)))
        self.server.creates += 1
        if self.server.mode == "partial-seed" and self.server.creates == 2:
            self.error("PRIMARY_CREATE_FAILURE")
        elif self.server.mode == "cleanup-all" and self.server.creates == 3:
            self.error("PRIMARY_CHILD_FAILURE")
        else:
            index = self.server.creates - 1
            self.respond(201, {"item": {"itemId": IDS[index], "name": f"Seed {index}"}})

    def do_GET(self):
        item_id = self.path.rsplit("/", 1)[-1]
        if self.server.mode == "java" or (self.server.mode == "cleanup-all" and item_id == IDS[0]):
            self.respond(404, {"__type": "ResourceNotFoundException", "message": "PRIMARY_READ_FAILURE",
                              "resourceId": item_id, "resourceType": "Item"})
        else:
            self.respond(200, {"item": {"itemId": item_id, "name": "Seed 1"}})

    def do_DELETE(self):
        item_id = self.path.rsplit("/", 1)[-1]
        self.server.deletes.append(item_id)
        if self.server.mode == "java" or (self.server.mode == "cleanup-all" and item_id == IDS[0]):
            self.error("SECONDARY_DELETE_FAILURE")
        else:
            self.respond(204)


def check(mode, arguments, primary, *, timeout=COMMAND_TIMEOUT):
    server = ThreadingHTTPServer(("127.0.0.1", 0), Fixture)
    server.mode, server.creates, server.deletes = mode, 0, []
    worker = threading.Thread(target=server.serve_forever, daemon=True)
    worker.start()
    endpoint = f"http://127.0.0.1:{server.server_port}"
    environment = dict(os.environ, CRUCIBLE_ENDPOINT=endpoint)
    command = list(map(str, arguments))
    if mode == "java":
        command.append(f"--args={endpoint}")
    try:
        try:
            result = subprocess.run(command, cwd=ROOT, env=environment, text=True,
                                    capture_output=True, timeout=timeout)
        except subprocess.TimeoutExpired as error:
            raise RuntimeError(f"Command timed out after {timeout:g}s: {command}") from error
        output = result.stdout + result.stderr
        if result.returncode == 0 or primary not in output:
            raise RuntimeError(f"Expected original {primary}:\n{output}")
        expected = IDS if mode == "cleanup-all" else IDS[:1]
        if server.deletes != expected:
            raise RuntimeError(f"Cleanup stopped early: {server.deletes}, expected {expected}\n{output}")
        if mode == "java" and ("Suppressed:" not in output or "SECONDARY_DELETE_FAILURE" not in output):
            raise RuntimeError(f"Java cleanup replaced the original failure:\n{output}")
        print(f"Passed cleanup fault injection: {mode}", flush=True)
    finally:
        server.shutdown()
        server.server_close()
        worker.join(timeout=10)


def main():
    rust = ["cargo", "test", "--manifest-path", "client/Cargo.toml", "--locked", "--test", "example", "--", "--ignored"]
    check("partial-seed", rust, "PRIMARY_CREATE_FAILURE")
    check("cleanup-all", rust, "PRIMARY_CHILD_FAILURE")
    wrapper = ROOT / "examples/java" / ("gradlew.bat" if os.name == "nt" else "gradlew")
    check("java", [wrapper, "--no-daemon", "--max-workers=1", "-p", "examples/java", "run"], "PRIMARY_READ_FAILURE")


if __name__ == "__main__":
    main()
