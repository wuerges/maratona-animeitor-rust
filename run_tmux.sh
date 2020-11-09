
#session1
tmux new -d cargo make watch
tmux split-window -h cargo run --bin simples -p lib-server 3030 "$@"
tmux attach
