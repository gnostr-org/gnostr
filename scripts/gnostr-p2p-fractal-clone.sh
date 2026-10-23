rm -rf gnostr_web_crlf/
rm -rf gnostr_web_crlf_repo/
cargo run -p gnostr-p2p --example fractal_swarm_perfect_ip -- --recursive /Users/Shared/github/gnostr/web/crlf.git --logging --out gnostr_web_crlf && git clone  gnostr_web_crlf ./gnostr_web_crlf_repo
