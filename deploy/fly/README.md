# Static website on Fly.io

This deploys only the built public site and its practice demo. There is no game backend, database, volume, proof service, token-claim server, or private save in the image. Nginx serves HTTP on port 8080; Fly terminates HTTPS. The app uses one shared CPU, 256 MB RAM, and can stop when idle. The app `diggerchain-alpha` was created and deployed in the maintainer's personal organization on 2026-10-24: https://diggerchain-alpha.fly.dev/. The custom domain remains to be supplied.

## Prepare locally

```sh
python3 scripts/build-website.py --skip-wasm
python3 scripts/prepare-fly.py
```

The isolated build context is `target/fly-website`. Deploy from that folder, never from the entire monorepo. The installed CLI is `~/.fly/bin/flyctl`. Its installation did not modify shell startup files or Codex configuration.

The maintainer signs in locally:

```sh
~/.fly/bin/flyctl auth login
```

After login, select the real organization and app name. Creating an app, allocating IPs, and deploying create account resources and are separate from preparation. Before publication, review the staged site, download availability, and actual destination with the maintainer.

## Update the existing deployment

```sh
cd target/fly-website
~/.fly/bin/flyctl deploy --remote-only --ha=false
```

The app already exists; do not run `launch` again. Keep the supplied configuration. `--ha=false` avoids creating a second spare machine. Do not provision a database or volume. If the chosen app name changes, persist it in `deploy/fly/fly.toml` and rebuild the context before the next deployment.

The hostname and deployment outcome must be recorded from the actual CLI result, not inferred from the proposed name. Once deployed, the maintainer supplies the Dynadot domain; use `flyctl certs add DOMAIN` and copy the exact DNS records returned by Fly into that domain's DNS. Do not replace unrelated DNS records. Certificate issuance and domain routing are not complete until their real status is confirmed.

Reference: [Fly app configuration](https://docs.fly.io/reference/configuration), [Fly deployment CLI](https://docs.fly.io/flyctl/cmd/fly_deploy), and [custom-domain certificates](https://docs.fly.io/flyctl/cmd/fly_certs_add).
