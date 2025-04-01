#!/bin/bash

# 检查是否安装了 tmux
if ! command -v tmux &> /dev/null; then
    echo "安装 tmux 中..."
    sudo apt-get update && sudo apt-get install -y tmux
fi

# 创建 tmux 会话并分割窗口
tmux new-session -d -s oyos-debug "cd /work/OYOS && python ysos.py launch -d"
tmux split-window -h -t oyos-debug
tmux send-keys -t oyos-debug:0.1 "cd /work/OYOS" C-m
tmux send-keys -t oyos-debug:0.1 "sleep 2" C-m  # 等待 QEMU 启动
tmux send-keys -t oyos-debug:0.1 "gdb -q -ex 'file esp/KERNEL.ELF' -ex 'gef-remote --qemu-user localhost 1234' -ex 'b ysos_kernel::init' -ex 'c'" C-m
tmux attach -t oyos-debug