import glob
import os
import subprocess


def generate_protos():
    proto_dir = "../proto"
    python_out = "src/agileplus_proto/gen"
    os.makedirs(python_out, exist_ok=True)

    # Get all .proto files recursively
    proto_files = glob.glob(f"{proto_dir}/**/*.proto", recursive=True)

    if not proto_files:
        print("No proto files found")
        return

    command = [
        "uv",
        "run",
        "python",
        "-m",
        "grpc_tools.protoc",
        f"-I{proto_dir}",
        f"--python_out={python_out}",
        f"--grpc_python_out={python_out}",
    ] + proto_files

    print(f"Running: {' '.join(command)}")
    result = subprocess.run(command, capture_output=True, text=True)

    if result.returncode != 0:
        print(f"Error: {result.stderr}")
    else:
        print("Successfully generated protos")
        # Ensure __init__.py exists in all subdirectories of python_out
        for root, dirs, files in os.walk(python_out):
            for d in dirs:
                init_path = os.path.join(root, d, "__init__.py")
                if not os.path.exists(init_path):
                    with open(init_path, "w") as f:
                        pass
        # Also ensure __init__.py in the output dir itself
        init_path = os.path.join(python_out, "__init__.py")
        if not os.path.exists(init_path):
            with open(init_path, "w") as f:
                pass


if __name__ == "__main__":
    generate_protos()
