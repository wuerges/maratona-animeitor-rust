# Max open files

Animeitor server relies heavily on websockets.
Thus we need to keep a very large number of open files.
Otherwise clients will get dropped.

Animeitor needs 3 websocket connections for each client.
Therefore, if we have 50000 users, we'll need 150000 open files.

We need to set this for both the `simples` server and NGINX.

For NGINX, we can add this entry to `/etc/nginx/nginx.conf`:

```
worker_rlimit_nofile 999999;
```

```
events {
        worker_connections 999999;
}
```

For the `simples` animeitor server, we can add an entry in `/etc/security/limits.conf`:

```
#<domain> <type> <item> <value>
animeitor-user soft nofile 999999
```

# NGINX https forwarding

There are several ways to host animeitor.
Since most of the content is static, we can use NGINX to do the regular hosting
and only use the `simples` server for the api.

Another way is to host http using the `simples` server, and https using NGINX as a reverse proxy.

The `nginx-examples/actix-http-nginx-https.conf` is an example of how to setup nginx as a https reverse proxy.

The `nginx-examples/api-only-server.conf` is an example of how to setup nginx to host the static
data and forward the api requests to the simples server handling only the api.
