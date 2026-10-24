# Preserved visual prototype

The development machine locally preserves the accepted visual playground at commit `7acf8d6`, tag `prototype-v0.1.0`, and branch `prototype/gloaming-threshold`.

In the current main workspace, normal launch opens the human expedition. Open the original goblin scene and its original JSON protocol explicitly:

```sh
cargo run --release --locked -- --prototype
cargo run --release --locked -- --prototype --headless
```

The baseline worktree below retains its original commands and source unchanged.

Its separate working directory is:

```text
../DCG-PROTOTYPE
```

Run the original game from that directory:

```sh
cargo run --release --locked
```

Run its original headless interface:

```sh
cargo run --release --locked -- --headless
```

This snapshot includes the original scene, top-down default view, optional sprites, enabled fog, disabled reveal, and CEO banner drafts. It does not include later expedition experiments. Its source matches the prototype before the roadmap implementation work began; no new test run has been performed during preservation.

The initial local commit uses `Codex <codex@localhost>` as the author because no Git author identity was configured. The identity was supplied for that command only. Global Git configuration and remotes were not changed.

On a development checkout containing the private baseline tag, create an equivalent worktree with:

```sh
git worktree add ../DCG-PROTOTYPE prototype-v0.1.0
```

The baseline tag remains fixed locally. The public alpha repository starts from a clean source snapshot and does not publish CEO draft history or the local baseline tag. Public clones can run the preserved scene with `--prototype`; they do not need the separate worktree. Continue development in the main workspace.
