#!/bin/bash
PYTHON_SCRIPT="main.py"
VENV_PATH="$HOME/app/latestEnv/bin/activate"
if [ -f "$PYTHON_SCRIPT" ]; then
    gnome-terminal -- bash -ic "source '$VENV_PATH' && python3 '$PYTHON_SCRIPT'; exec bash"
else
    gnome-terminal -- bash -c "echo 'Error: $PYTHON_SCRIPT not found!'; exec bash"
fi   