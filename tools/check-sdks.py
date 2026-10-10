"""Build the template and exercise both independent SDK consumers over HTTP."""
from pathlib import Path
import json
import os
import socket
import subprocess
import sys
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
COMMAND_TIMEOUT = float(os.environ.get("CRUCIBLE_SDK_COMMAND_TIMEOUT", "600"))


def run(arguments, *, capture=False, env=None, timeout=COMMAND_TIMEOUT):
    print("Running " + " ".join(map(str, arguments)), flush=True)
    try:
        result = subprocess.run(list(map(str, arguments)), cwd=ROOT, env=env,
                                text=True, stdout=subprocess.PIPE if capture else None,
                                timeout=timeout)
    except subprocess.TimeoutExpired as error:
        raise RuntimeError(f"Command timed out after {timeout:g}s: {arguments}") from error
    if result.returncode:
        raise RuntimeError(f"Command failed ({result.returncode}): {arguments}")
    return result.stdout


def main():
    output = run(["cargo", "build", "--workspace", "--locked", "--message-format=json"], capture=True)
    executable = None
    for line in output.splitlines():
        message = json.loads(line)
        if message.get("reason") == "compiler-artifact" and message["target"]["name"] == "crucible-service":
            executable = message.get("executable") or executable
        if message.get("reason") == "compiler-message":
            print(message["message"].get("rendered", ""), file=sys.stderr)
    if not executable:
        raise RuntimeError("Cargo did not report the service executable")
    for manifest in ["target/rust-sdk/Cargo.toml", "target/java-sdk/build.gradle.kts"]:
        if not (ROOT / manifest).is_file():
            raise RuntimeError(f"Model build did not generate {manifest}")
    run(["cargo", "fmt", "--all", "--check"])
    run(["cargo", "fmt", "--manifest-path", "client/Cargo.toml", "--all", "--check"])
    run(["cargo", "test", "--workspace", "--locked"])
    run(["cargo", "test", "--manifest-path", "client/Cargo.toml", "--locked"])

    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        port = listener.getsockname()[1]
    endpoint = f"http://127.0.0.1:{port}"
    environment = dict(os.environ, CRUCIBLE_ADDR=f"127.0.0.1:{port}", CRUCIBLE_ENDPOINT=endpoint)
    server = subprocess.Popen([executable], cwd=ROOT, env=environment)
    try:
        for _ in range(200):
            try:
                with socket.create_connection(("127.0.0.1", port), timeout=0.1):
                    break
            except OSError:
                if server.poll() is not None:
                    raise RuntimeError("Service exited before accepting requests")
                time.sleep(0.1)
        else:
            raise RuntimeError("Service did not start")
        run([sys.executable, "service/tools/smoke.py", endpoint])
        run(["cargo", "test", "--manifest-path", "client/Cargo.toml", "--locked",
             "--test", "example", "--", "--ignored"], env=environment)
        wrapper = ROOT / "examples/kotlin" / ("gradlew.bat" if os.name == "nt" else "gradlew")
        output = run([wrapper, "--no-daemon", "--max-workers=1", "-p", "examples/kotlin",
                      "clean", "run", f"--args={endpoint}"], capture=True)
        print(output)
        for expected in ["Created ", "Read: ", "Updated: ", "Listed: ", "Deleted "]:
            if expected not in output:
                raise RuntimeError(f"Kotlin example did not report {expected!r}")
        with urllib.request.urlopen(endpoint + "/items?maxResults=100", timeout=10) as response:
            if json.load(response)["items"]:
                raise RuntimeError("SDK examples left temporary items behind")
        print("Service, independent Rust SDK consumer, Gradle Kotlin consumer of the Java SDK, pagination and cleanup passed")
    finally:
        server.terminate()
        try:
            server.wait(timeout=20)
        except subprocess.TimeoutExpired:
            server.kill()
            server.wait()
    run([sys.executable, "tools/check-sdk-cleanup.py"])


if __name__ == "__main__":
    main()
