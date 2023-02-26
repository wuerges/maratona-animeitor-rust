# Notes

Cross compiling to linux with MUSL:

```bash
cargo build --release --target x86_64-unknown-linux-musl
scp target/x86_64-unknown-linux-musl/release/simples ew@animeitor.naquadah.com.br:
```

Setting capabilities to binary so it can open port 80:

```bash
sudo setcap 'cap_net_bind_service=+ep' ../simples
```

Increasing the limit of file descriptors for the shell:

```bash
ulimit -n unlimited
```

# MUSL

Install toolchain:

```bash
# Add target with rustup
rustup target list

# install musl cross
brew install filosottile/musl-cross/musl-cross

# link for easy access
ln -s /usr/local/opt/musl-cross/bin/x86_64-linux-musl-gcc /usr/local/bin/musl-gcc
```

Add this to `.cargo/config`:

```toml
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"
```
