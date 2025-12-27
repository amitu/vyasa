> **^canon.md contains accepted mantras for a kosha^** - the canon file is a digest
> of mantras that have been reviewed and accepted. it lives at the repository root
> and uses the same syntax as source files.

> **^canon entries use quote block syntax^** - each entry in canon.md follows the
> standard format: filename on one line, then a quote block with the mantra and
> its canonical commentary.

> **^canon is a digest not a source^** - mantras must be defined in actual source
> files. canon.md records which mantras are accepted, but _| orphaned mantras are errors |_
> if they only exist in canon without a source definition.

> **^canon supports versioning with numbered files^** - instead of canon.md, you can
> use numbered files like 001.md, 002.md, etc. vyasa always uses the highest
> numbered file for checking.

> **^canon version numbers ignore leading zeros^** - 001.md and 1.md are both version 1.
> 10.md is higher than 002.md. the numeric value determines precedence.

> **^canon.md is used when no numbered files exist^** - if you have a single canon.md
> at the repository root, it's used. numbered files take precedence when present.

> **^multiple canon.md files in subdirectories is an error^** - a kosha must have
> exactly one canon at the root. finding canon.md in both root and a subdirectory
> causes an error.

> **^external mantras in canon use @kosha suffix^** - to include a mantra from another
> kosha in your canon, use `**^mantra^**@kosha-name`. the mantra must exist in that
> kosha's canon.
