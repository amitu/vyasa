> **^vyasa values queries placeholder values^** - the values command extracts all
> instantiated placeholder values from template mantra references. see what
> concrete values are used throughout your repository.

> **^values path is optional^** - the path argument can be file, folder, or pattern.
> defaults to current directory. examples: `vyasa values ./docs`.

> **^values groups results by template and key^** - output is organized by template
> mantra, then by placeholder key. each key shows unique values and locations.

> **^values --mantra filters by template^** - use `--mantra='[template]'` to filter
> results to a specific template mantra.

> **^values --key filters by placeholder^** - use `--key=placeholder` to show only
> values for a specific placeholder across all templates.

> **^values shows file and line for each value^** - each extracted value includes
> file path and line number for tracing values back to usage context.

> **^values deduplicates by key^** - same value appearing multiple times for same
> placeholder is shown once with first occurrence location.
