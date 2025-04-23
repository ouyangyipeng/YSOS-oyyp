#!/bin/bash

# 检查是否安装了 gnome-terminal 或 xterm
if ! command -v gnome-terminal &> /dev/null && ! command -v xterm &> /dev/null; then
    echo "请安装 gnome-terminal 或 xterm"
    exit 1
fi

# 在第一个终端中运行 ysos.py
gnome-terminal -- bash -c "cd /work/OYOS && python ysos.py launch -d; exec bash"

# 等待 QEMU 启动
sleep 2

# 在第二个终端中运行 GDB 调试
gnome-terminal -- bash -c "cd /work/OYOS && gdb -q -ex 'file esp/KERNEL.ELF' -ex 'gef-remote --qemu-user localhost 1234' -ex 'b _start' -ex 'c'; exec bash"