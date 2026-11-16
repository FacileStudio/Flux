#!/usr/bin/env bash
set -euo pipefail

cyan='\033[0;36m'
green='\033[0;32m'
yellow='\033[1;33m'
red='\033[0;31m'
reset='\033[0m'
bold='\033[1m'

df -hP | awk 'NR > 1 && $1 ~ /^\/dev\// {print $1 "|" $4 "|" $6}' | while IFS='|' read -r dev avail mount; do
    name="$(basename "$dev")"
    if [ -n "$mount" ] && [ "$mount" != "/" ]; then
        name="$name ($mount)"
    fi

    size_num="${avail%%[!0-9.]*}"
    size_unit="${avail#"$size_num"}"
    color="$green"

    case "$size_unit" in
        G|GB|Gi)
            whole="${size_num%.*}"
            [ -z "$whole" ] && whole=0
            if [ "$whole" -lt 20 ]; then
                color="$red"
            elif [ "$whole" -lt 50 ]; then
                color="$yellow"
            fi
            ;;
        M|MB|Mi|K|KB|Ki)
            color="$red"
            ;;
    esac

    printf '%b\n' "${cyan}${bold}${name}${reset}: ${color}${bold}${avail}${reset} left"
done
