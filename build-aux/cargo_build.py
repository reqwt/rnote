import os
import shutil
import subprocess
from subprocess import CalledProcessError
import sys

project_build_root = sys.argv[1]
project_src_root = sys.argv[2]
cargo_env_str = sys.argv[3]
cargo_cmd = sys.argv[4]
cargo_options = sys.argv[5]
bin_output = sys.argv[6]
output_file = sys.argv[7]

print(
    f"""
### executing cargo build script with arguments: ###
    project_build_root: {project_build_root}
    project_src_root: {project_src_root}
    cargo_env: {cargo_env_str}
    cargo_cmd: {cargo_cmd}
    cargo_options: {cargo_options}
    bin_output: {bin_output}
    output_file: {output_file}
""",
    file=sys.stderr,
)

env = os.environ.copy()
ucrt64_bin = "C:\\msys64\\ucrt64\\bin"
if ucrt64_bin not in env.get("PATH", ""):
    env["PATH"] = ucrt64_bin + os.pathsep + env.get("PATH", "")

for item in cargo_env_str.split():
    if "=" in item:
        k, v = item.split("=", 1)
        env[k] = v

cargo_call = f'"{cargo_cmd}" build {cargo_options}'

print(cargo_call, file=sys.stderr)
try:
    subprocess.run(cargo_call, env=env, shell=True, check=True)
except CalledProcessError as e:
    print(f"cargo call failed: {e}", file=sys.stderr)
    sys.exit(1)

src = bin_output
if not os.path.exists(src) and os.path.exists(src + ".exe"):
    src = src + ".exe"

print(f"copying {src} to {output_file}", file=sys.stderr)
try:
    shutil.copy2(src, output_file)
except Exception as e:
    print(f"copy failed: {e}", file=sys.stderr)
    sys.exit(1)

print("### cargo build script finished ###", file=sys.stderr)
