#!/bin/bash
set -e

# Parse arguments
TARGET_VALUE=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --target)
            if [ -n "$2" ]; then
                TARGET_VALUE="$2"
                shift 2
            else
                echo "Error: --target requires a value"
                exit 1
            fi
            ;;
        *)
            # Unknown argument, ignore it
            shift
            ;;
    esac
done

# Build with napi
if [ -n "$TARGET_VALUE" ]; then
    napi build --platform --release --target "$TARGET_VALUE"
else
    napi build --platform --release
fi

# Run lint:fix (without any target argument)
pnpm run lint:fix

