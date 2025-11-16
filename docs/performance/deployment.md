# 🚀 Production Deployment

Deploy Firework applications to production.

---

## Building for Production

```bash
cargo build --release
```

Binary location: `target/release/your-app`

---

## Docker

**Dockerfile:**
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /app/target/release/your-app /usr/local/bin/
EXPOSE 8080
CMD ["your-app"]
```

Build and run:
```bash
docker build -t my-app .
docker run -p 8080:8080 my-app
```

---

## Systemd Service

**/etc/systemd/system/myapp.service:**
```ini
[Unit]
Description=My Firework App
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/myapp
ExecStart=/opt/myapp/myapp
Restart=always

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable myapp
sudo systemctl start myapp
sudo systemctl status myapp
```

---

## Nginx Reverse Proxy

**/etc/nginx/sites-available/myapp:**
```nginx
upstream firework {
    server 127.0.0.1:8080;
}

server {
    listen 80;
    server_name example.com;
    
    location / {
        proxy_pass http://firework;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

---

## Environment Variables

```bash
export DATABASE_URL=postgresql://user:pass@localhost/db
export JWT_SECRET=production-secret-key
export RUST_LOG=info
```

---

## Process Management

### PM2
```bash
pm2 start target/release/myapp --name "firework-app"
pm2 save
pm2 startup
```

### supervisord
```ini
[program:myapp]
command=/opt/myapp/myapp
directory=/opt/myapp
user=www-data
autostart=true
autorestart=true
```
