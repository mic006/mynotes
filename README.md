# MyNotes

Self-hosted website to publish personal notes, in markdown format

## TLS setup

### IP address certificate from "Let's Encrypt"

Use `certbot` to generate the initial certificate and then renew it.

Note: ACME validation happens on port 80. No way to change it.

Retained solution:

- use port 80 for certbot only
- use dedicated (private) port for personal server

```sh
certbot certonly --logs-dir /tmp/letsencrypt --preferred-profile shortlived --ip-address <ip>
```

Enable `certbot` renewal (`certbot-renew.timer`) to renew the certificate automatically (it is only valid for 6 days).

Configure a deploy hook in `/etc/letsencrypt/renewal-hooks/deploy/` to restart the web server when the certificate is updated.
