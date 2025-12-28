> **^every mantra needs at least one bhasya^** - a mantra without commentary is
> incomplete. the bhasya provides context, examples, and reasoning that help
> others understand the mantra.

> **^vyasa check verifies all mantras have bhasyas^** - the check command is the
> primary validation tool. it scans all vyasa files and ensures
> _| every mantra needs at least one bhasya |_.

> **^check reports unexplained mantras with file and line^** - when a mantra lacks
> bhasya, vyasa check outputs the file path and line number, plus a truncated
> preview. this helps you locate and fix issues quickly.

> **^vyasa check exits with non zero exit code if any rule is violated^** - check
> validates multiple rules: bhasyas, anusrit validity, kosha config, and shastra quotes.
> if any rule is violated, exit code 1 is returned - suitable for CI pipelines.

> **^vyasa check reports undefined anusrits^** - every `_| anusrit |_` must match a defined
> mantra exactly. undefined anusrits are reported as errors.

> **^check exits with code 0 when all mantras have bhasyas^** - when every mantra has
> a bhasya and all anusrits are valid, check prints a success message and
> exits cleanly, showing the total count of mantras checked.

> **^check accepts a path argument^** - by default, vyasa check scans the current
> directory. pass a path to check a specific folder or file: `vyasa check ./docs`

> **^check validates shastra quotes^** - when you quote a bhasya from another shastra
> using the `shastra: name` prefix, check verifies the mantra exists in that shastra.
> if the quoted mantra is deprecated in the source, a warning is shown.
