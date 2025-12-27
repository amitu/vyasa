> **^vyasa list shows all mantras with canon status^** - the list command displays
> every mantra in alphabetical order with a status marker showing its relationship
> to canon.

> **^list uses status markers^** - each mantra is prefixed with: [ok] for accepted,
> [!!] for new, [**] for changed, [??] for orphaned (canon only).

> **^list --pending shows only unaccepted mantras^** - use the -p or --pending flag
> to filter the list to mantras not yet in canon.

> **^list accepts a filter argument^** - pass text to filter mantras: `vyasa list kosha`
> shows only mantras containing "kosha" in their text.

> **^list shows definition count^** - mantras with multiple definitions show the count
> in parentheses. mantras only in canon show "(canon only)".

> **^list accepts a path argument^** - use --path to specify the repository location:
> `vyasa list --path=./docs`.
