#!/bin/bash

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE}" )" && pwd )"

# Check if nohup mode is requested (first parameter)
NOHUP_MODE="${1:-false}"

if [ "$NOHUP_MODE" = "true" ]; then
    # Run invisibly with nohup
    nohup bash -c "cd '$SCRIPT_DIR' && ./pieStatistic" > /dev/null 2>&1 &
else
    # Run in GNOME Terminal
    gnome-terminal --geometry=46x24+900+0 -- bash -c "cd '$SCRIPT_DIR' && ./pieStatistic; bash"
fi
