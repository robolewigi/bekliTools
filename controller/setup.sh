#!/bin/bash

DIR="$(pwd)"

echo "enter to continue ctrl+c to exit"
read -p "installing libraries"

sudo apt update
sudo apt install -y libsdl2-2.0-0 libsdl2-dev libsdl2-image-2.0-0 libsdl2-image-dev x11-utils gnome-terminal

read -p "create .desktop file at $DIR (gnome only)"

cat > ~/.local/share/applications/controllerCT.desktop <<EOF
[Desktop Entry]
Name=controllerCT
Exec=gnome-terminal -- bash -c '$DIR/controllerCT; exec bash'
Path=$DIR
Icon=$DIR/main.png
Type=Application
Terminal=true
Categories=Utility;
EOF


chmod +x ~/.local/share/applications/controllerCT.desktop
update-desktop-database ~/.local/share/applications

read -p "done"
