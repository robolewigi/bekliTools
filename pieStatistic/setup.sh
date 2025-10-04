#!/bin/bash

DIR="$(pwd)"

echo "enter to continue ctrl+c to exit"
read -p "installing libraries"

sudo apt update
sudo apt install libgtk-3-0 libxdo3

read -p "create .desktop file at $DIR"

cat > ~/.local/share/applications/piestatistic.desktop <<EOF
[Desktop Entry]
Name=pieStatistic
Exec=$DIR/pieStatistic
Icon=$DIR/icon.png
Type=Application
Terminal=true
Categories=Utility;
EOF

chmod +x ~/.local/share/applications/piestatistic.desktop
update-desktop-database ~/.local/share/applications