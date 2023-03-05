cargo build --release --target x86_64-unknown-linux-musl
ssh ew@animeitor.naquadah.com.br rm simples
scp target/x86_64-unknown-linux-musl/release/simples ew@animeitor.naquadah.com.br:

