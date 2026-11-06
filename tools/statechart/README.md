# Statechart runtime

Vendored runtime from `cursor/plugins`, `pstack/skills/statechart`, revision
`4dd1d05818d19792e7cd6e818f1667f95f7e1dbe`, as distributed with the installed
statechart skill on November 6, 2026. MIT license: see `LICENSE.txt`.

Only the worker and its runtime imports are included; no npm install is needed.
Local adaptations in `run.ts` normalize CRLF output and use Windows process-tree
cleanup and hidden child windows. The repository runner supplies the pinned TLC
jar explicitly, so the runtime's mise fallback is unused.

Use `statecharts/run.ps1`; treat this directory as third-party code, rather than
application logic. On upgrade, review the upstream diff and rerun both known
counterexamples and passing models before replacing this snapshot.
