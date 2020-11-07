
#session1
tmux new  -s SessionName -n URL cargo make watch
tmux split-window -h cargo run -p lib-server 3030 "$@"
tmux attach
