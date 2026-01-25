#!/bin/bash
addLines="-I/usr/include/SDL2 -lSDL2 -lreadline"
g++ -o controllerBekli main.cpp $addLines
if [ $? -eq 0 ]; then
    x-terminal-emulator -e bash -c "stdbuf -o0 ./controllerBekli; exec bash"
else
    x-terminal-emulator -e bash -c "echo Compilation failed.; echo Showing errors below:;
    echo; g++ -o controllerBekli main.cpp $addLines 2>&1; exec bash"
fi