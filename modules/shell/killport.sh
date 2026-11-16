#!/usr/bin/env bash
set -euo pipefail

if [ $# -ne 1 ] || [ -z "$1" ]; then
    echo "Usage: killport <port>" >&2
    exit 2
fi

pids="$(lsof -ti "tcp:$1" 2>/dev/null || true)"
if [ -z "$pids" ]; then
    echo "No process found on port $1"
    exit 0
fi

printf '%s\n' "$pids" | xargs kill -9
echo "Port $1 freed"
