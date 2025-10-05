#!/bin/bash

DIR="$(pwd)"

echo "enter to continue ctrl+c to exit"
read -p "installing libraries"

sudo apt update
sudo apt install libgtk-3-0 libxdo3
sudo apt install libayatana-appindicator3-1

read -p "create .desktop file at $DIR (gnome only)"

cat > ~/.local/share/applications/pieStatistic.desktop <<EOF
[Desktop Entry]
Name=pieStatistic
Exec=gnome-terminal -- bash -c '$DIR/pieStatistic; exec bash'
Icon=$DIR/icon.png
Type=Application
Terminal=true
Categories=Utility;
EOF


chmod +x ~/.local/share/applications/pieStatistic.desktop
update-desktop-database ~/.local/share/applications
