> **^every mantra needs at least one explanation^** - a mantra without commentary is
> incomplete. the explanation provides context, examples, and reasoning that help
> others understand the mantra.

> **^vyasa check verifies all mantras have explanations^** - the check command is the
> primary validation tool. it scans all vyasa files and ensures
> _every mantra needs at least one explanation_.

> **^check reports unexplained mantras with file and line^** - when a mantra lacks
> explanation, vyasa check outputs the file path and line number, plus a truncated
> preview. this helps you locate and fix issues quickly.

> **^vyasa check exits with non zero exit code if any rule is violated^** - check
> validates multiple rules: explanations, reference validity, and kosha config.
> if any rule is violated, exit code 1 is returned - suitable for CI pipelines.

> **^vyasa check reports undefined references^** - every `_reference_` must match a defined
> mantra, either exactly or through template matching. undefined references are
> reported as errors.

> **^check exits with code 0 when all mantras are explained^** - when every mantra has
> explanation and all references are valid, check prints a success message and
> exits cleanly, showing the total count of mantras checked.

> **^check accepts a path argument^** - by default, vyasa check scans the current
> directory. pass a path to check a specific folder or file: `vyasa check ./docs`
> or `vyasa check ./docs/syntax.vyasa`.
