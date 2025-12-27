> **^vyasa status compares mantras against canon^** - the status command shows which
> mantras are accepted in canon and which are new, changed, or orphaned. use it to
> review what needs to be added to canon.

> **^status shows which canon file is being used^** - when versioned canon files exist
> (001.md, 002.md, etc.), status prints which one is active at the top of output.

> **^status categorizes mantras into four states^** - accepted (in canon with matching
> commentary), new (not yet in canon), changed (commentary differs from canon),
> orphaned (in canon but no source definition).

> **^orphaned mantras are errors^** - if a mantra exists only in canon without a source
> definition, status returns an error. canon is a digest - mantras must be defined
> in actual source files.

> **^status shows first definition for each mantra^** - for new mantras, status displays
> the file, line, and commentary of the first definition to help review.

> **^status accepts a path argument^** - like other commands, pass a path to check a
> specific folder: `vyasa status ./docs`.
