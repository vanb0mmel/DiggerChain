# Static website on Fly.io

This deploys only the built public site and its practice demo. There is no game backend, database, volume, proof service, token-claim server, or private save in the image. Nginx serves HTTP on port 8080; Fly terminates HTTPS. The app uses one shared CPU, 256 MB RAM, and can stop when idle. The app `diggerchain-alpha` was created and deployed in the maintainer's personal organization on 2026-10-24: https://diggerchain-alpha.fly.dev/. The site also serves https://diggerchain.xyz/ and https://www.diggerchain.xyz/, with active HTTPS certificates.

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

The hostname and deployment outcome must be recorded from the actual CLI result, not inferred from the proposed name. For any future domain, the maintainer supplies its name; use `flyctl certs add DOMAIN` and copy the exact DNS records returned by Fly into that domain's DNS. Do not replace unrelated DNS records. Certificate issuance and domain routing are not complete until their real status is confirmed.

Reference: [Fly app configuration](https://docs.fly.io/reference/configuration), [Fly deployment CLI](https://docs.fly.io/flyctl/cmd/fly_deploy), and [custom-domain certificates](https://docs.fly.io/flyctl/cmd/fly_certs_add).

## Fast token publication

Once the maintainer supplies all three official values, run from the monorepo:

```sh
python3 scripts/publish_token.py --ca OFFICIAL_CA --x 'OFFICIAL_HTTPS_X_URL' --pons 'OFFICIAL_HTTPS_PONS_URL'
```

The command validates the supplied values, updates the single website configuration and README token block, and rebuilds static pages using the existing WASM module and native archives. GitHub push and Fly deployment run in parallel. A final HTTPS read confirms the live configuration; logs are retained in `target/token-publication`. The command rejects existing edits to the two source files, commits only those files, and reports partial publication failures explicitly. It does not start game builds, tests, proof generation, or wallet operations. The 2–3 minute launch window is a target, not a guarantee of external service availability.
