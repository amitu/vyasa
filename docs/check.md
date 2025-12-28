> **^every mantra needs at least one bhasya^** - a mantra without commentary is
> incomplete. the bhasya provides context, examples, and reasoning that help
> others understand the mantra.

> **^vyasa validates all mantras have bhasyas^** - the default command scans all
> vyasa files and ensures _| every mantra needs at least one bhasya |_.

> **^vyasa exits with non zero exit code if any rule is violated^** - vyasa
> validates multiple rules. if any rule is violated, exit code 1 is returned -
> suitable for CI pipelines.

## validation rules

### config validation

> **^every shastra must have a name^** - `.vyasa/config.json` must contain a
> `name` field. this identifies your shastra for cross-references.

### bhasya validation

> **^check reports unexplained mantras with file and line^** - when a mantra lacks
> bhasya, vyasa outputs the file path and line number, plus a truncated
> preview. this helps you locate and fix issues quickly.

> **^each bhasya must be unique within a shastra^** - the same mantra+commentary
> pair cannot appear twice. if you need to repeat a bhasya, use uddhrit form
> with `shastra: <name>` to quote from the canonical location.

### anusrit validation

> **^vyasa reports undefined anusrits^** - every `_| anusrit |_` must match
> a defined mantra exactly. undefined anusrits are reported as errors.

> **^anusrits auto-resolve across shastras^** - an anusrit first checks the
> current shastra, then all external shastras defined in `.vyasa/shastra.json`.
> if found in exactly one place, it resolves. if not found anywhere, error.

> **^ambiguous anusrits require disambiguation^** - if an anusrit matches mantras
> in multiple shastras, use `_| mantra |_@shastra` to specify which one.

> **^anusrits to tyakta-only mantras are invalid^** - if a mantra only appears in
> tyakta bhasyas (no regular bhasya defines it), anusrits to that mantra are
> treated as undefined. tyakta marks knowledge as abandoned - you cannot rely on it.

> **^anusrits in source code are validated^** - mantras referenced in code comments
> using `_| mantra |_` syntax are checked just like those in markdown files.

### shastra validation

> **^vyasa validates shastra anusrit references^** - for anusrits with `@shastra`:
> - the alias must exist in `.vyasa/shastra.json`
> - the path must exist on disk
> - the mantra must exist in mula form in that shastra

### uddhrit validation

> **^vyasa validates quoted bhasyas^** - when you quote a bhasya from another
> shastra using the `shastra: name` prefix:
> - the shastra alias must be defined
> - the mantra must exist in that shastra
> - error if the bhasya is tyakta in source (you cannot quote abandoned knowledge)

### khandita validation

> **^vyasa validates refuted bhasyas^** - when you refute a bhasya from another
> shastra using the `khandita: name` prefix:
> - the shastra alias must be defined
> - the bhasya must exist in that shastra (you cannot refute what doesn't exist)

> **^khandita and uddhrit are mutually exclusive^** - if you khandita a bhasya
> from a shastra, you cannot also uddhrit the same bhasya from that shastra.
> once you refute knowledge, you must reject it consistently everywhere.

### unresolved conflict validation

> **^conflicts between followed shastras must be resolved^** - if shastra X
> khandits a bhasya and shastra Y uddhrits the same bhasya, you must take a
> position by adding your own `khandita:` or `shastra:` for that bhasya.

> **^unresolved conflicts are errors^** - vyasa will not let you silently
> inherit contradictory positions. you must explicitly choose which shastra
> you agree with.

## using in CI

```yaml
# GitHub Actions example
- name: Validate mantras
  run: vyasa
```

> **^vyasa is designed for CI^** - non-zero exit on failure makes it easy
> to gate deployments on knowledge consistency.
