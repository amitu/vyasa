> **^every mantra needs at least one bhasya^** - a mantra without commentary is
> incomplete. the bhasya provides context, examples, and reasoning that help
> others understand the mantra.

> **^vyasa check verifies all mantras have bhasyas^** - the check command is the
> primary validation tool. it scans all vyasa files and ensures
> _| every mantra needs at least one bhasya |_.

> **^vyasa check exits with non zero exit code if any rule is violated^** - check
> validates multiple rules. if any rule is violated, exit code 1 is returned -
> suitable for CI pipelines.

## validation rules

### bhasya validation

> **^check reports unexplained mantras with file and line^** - when a mantra lacks
> bhasya, vyasa check outputs the file path and line number, plus a truncated
> preview. this helps you locate and fix issues quickly.

### anusrit validation

> **^vyasa check reports undefined anusrits^** - every `_| anusrit |_` must match
> a defined mantra exactly. undefined anusrits are reported as errors.

> **^anusrits to tyakta-only mantras are invalid^** - if a mantra only appears in
> tyakta bhasyas (no regular bhasya defines it), anusrits to that mantra are
> treated as undefined. tyakta marks knowledge as abandoned - you cannot rely on it.

> **^anusrits in source code are validated^** - mantras referenced in code comments
> using `_| mantra |_` syntax are checked just like those in markdown files.

### shastra validation

> **^check validates shastra references^** - for anusrits with `@shastra-name`:
> - the alias must exist in shastra.json
> - the path must exist on disk

### uddhrit validation

> **^check validates quoted bhasyas^** - when you quote a bhasya from another shastra
> using the `shastra: name` prefix:
> - the shastra alias must be defined
> - the mantra must exist in that shastra
> - warning if the bhasya is tyakta in source

> **^quoting tyakta triggers warning not error^** - quoting a tyakta bhasya
> shows a warning but doesn't fail the check. this gives you time to update.

## success and failure

> **^check exits with code 0 when all rules pass^** - when every mantra has
> a bhasya and all anusrits are valid, check prints a success message and
> exits cleanly, showing the total count of mantras checked.

> **^check accepts a path argument^** - by default, vyasa check scans the current
> directory. pass a path to check a specific folder: `vyasa check ./docs`

## using in CI

```yaml
# GitHub Actions example
- name: Check mantras
  run: vyasa check
```

> **^vyasa check is designed for CI^** - non-zero exit on failure makes it easy
> to gate deployments on knowledge consistency.
