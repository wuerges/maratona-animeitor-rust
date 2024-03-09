# Notes

Cross compiling to linux with MUSL:

```bash
cargo build --release --target x86_64-unknown-linux-musl
ssh ew@animeitor.naquadah.com.br rm simples
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

# NGINX ws configuration

```
##
server {
    listen [::]:443 ssl ipv6only=on; # managed by Certbot
    listen 443 ssl; # managed by Certbot
    ssl_certificate /etc/letsencrypt/live/animeitor.naquadah.com.br/fullchain.pem; # managed by Certbot
    ssl_certificate_key /etc/letsencrypt/live/animeitor.naquadah.com.br/privkey.pem; # managed by Certbot
    include /etc/letsencrypt/options-ssl-nginx.conf; # managed by Certbot
    ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem; # managed by Certbot

    location / {
        proxy_pass http://animeitor.naquadah.com.br;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto https;

        # websocket upgrade
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "Upgrade";
        # proxy_set_header Host $host;

        # websocket timeouts:
        proxy_read_timeout 1200s;

    }
}
```
