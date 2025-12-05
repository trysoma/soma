"""Script to generate Python API client from OpenAPI spec.

This uses openapi-generator-cli to generate the client, similar to how
the TypeScript client is generated in js/packages/api-client.
"""

import subprocess
import sys
from pathlib import Path


def main() -> None:
    """Generate the API client from OpenAPI spec."""
    # Find the project root (where openapi.json is)
    current = Path(__file__).parent
    project_root = current.parent.parent.parent.parent  # py/packages/api-client -> root

    openapi_spec = project_root / "openapi.json"
    output_dir = current

    if not openapi_spec.exists():
        print(f"Error: OpenAPI spec not found at {openapi_spec}")
        sys.exit(1)

    # Read VERSION file
    version_file = project_root / "VERSION"
    version = "0.0.4"
    if version_file.exists():
        version = version_file.read_text().strip()

    print(f"Generating Python API client v{version} from {openapi_spec}...")

    # Run openapi-generator
    cmd = [
        "npx",
        "--yes",
        "@openapitools/openapi-generator-cli@latest",
        "generate",
        "-i",
        str(openapi_spec),
        "-g",
        "python",
        "-o",
        str(output_dir.parent),
        "--additional-properties",
        f"packageName=soma_api_client,packageVersion={version},generateSourceCodeOnly=true",
    ]

    print(f"Running: {' '.join(cmd)}")

    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        print(result.stdout)
        if result.stderr:
            print(result.stderr, file=sys.stderr)
        print("API client generated successfully!")
    except subprocess.CalledProcessError as e:
        print(f"Error generating API client: {e}")
        if e.stdout:
            print(e.stdout)
        if e.stderr:
            print(e.stderr, file=sys.stderr)
        sys.exit(1)
    except FileNotFoundError:
        print("Error: npx not found. Please install Node.js.")
        sys.exit(1)


if __name__ == "__main__":
    main()
