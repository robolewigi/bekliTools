#!/bin/bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE}" )" && pwd )"
NOHUP_MODE="${1:-false}"
PACKAGES=("xdotool" "libayatana-appindicator3-1")

MISSING_PACKAGES=()

for pkg in "${PACKAGES[@]}"; do
    if ! dpkg -l | grep -q "^ii.*$pkg"; then
        echo "$pkg is not installed"
        MISSING_PACKAGES+=("$pkg")
    fi
done

if [ ${#MISSING_PACKAGES[@]} -gt 0 ]; then
    echo ""
    echo "sudo apt-get update && sudo apt-get install -y (packageName)"
    sudo apt-get update
    sudo apt-get install -y "${MISSING_PACKAGES[@]}"
    
    if [ $? -eq 0 ]; then
        echo ""
        echo "success"
    else
        echo ""
        echo "error"
        exit 1
    fi
else
    echo ""
    echo "all installed"
fi

if [ "$NOHUP_MODE" = "true" ]; then
    # Run invisibly with nohup
    nohup bash -c "cd '$SCRIPT_DIR' && ./pieStatistic" > /dev/null 2>&1 &
else
    # Run in GNOME Terminal
    gnome-terminal --geometry=46x24+900+0 -- bash -c "cd '$SCRIPT_DIR' && ./pieStatistic; bash"
fi
