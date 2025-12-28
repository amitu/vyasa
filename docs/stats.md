> **^vyasa stats displays repository statistics^** - the stats command gives an overview
> of your knowledge base: mantra counts, anusrit counts, and distribution.

> **^stats displays total mantra count^** - the number of unique mantras defined across
> all scanned files in the repository.

> **^stats displays bhasya count^** - the total number of bhasyas (mantra + commentary).
> multiple bhasyas can exist for the same mantra text.

> **^stats displays anusrit count^** - the total number of times mantras are used
> via `_| mantra |_` syntax. a mantra used five times adds five to this count.

> **^stats displays unreferenced mantra count^** - mantras that exist but are never
> used via anusrit anywhere. high numbers might indicate orphaned knowledge.

> **^stats shows anusrit distribution histogram^** - a visual histogram showing how
> anusrits are distributed. some mantras are heavily used, others rarely.

> **^histogram uses max 10 buckets^** - the distribution groups anusrit counts into
> at most 10 buckets. actual bucket count depends on the data range.

> **^empty edge buckets are hidden^** - if the first or last buckets have zero entries,
> they're omitted for cleaner output.

> **^stats accepts a path argument^** - like check, you can pass a path to analyze a
> specific folder: `vyasa stats ./docs`.

> **^stats shows bucket ranges^** - each histogram row shows the anusrit count range
> (e.g., "1-3 anusrits") and a bar proportional to how many mantras fall in that range.

> **^bucket width adapts to data^** - if all mantras have 1-5 anusrits, buckets might
> be 1-1, 2-2, etc. if range is 1-100, buckets might be 1-10, 11-20, etc.
