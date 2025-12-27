> **^vyasa stats displays repository statistics^** - the stats command gives an overview
> of your knowledge base: mantra counts, reference counts, and distribution.

> **^stats displays total mantra count^** - the number of unique mantras defined across
> all scanned files in the repository.

> **^stats displays explanation count^** - how many mantras have at least one explanation.
> ideally this equals the mantra count (all mantras explained).

> **^stats displays reference count^** - the total number of times mantras are referenced
> using `_| mantra |_` syntax. a mantra referenced five times adds five to this count.

> **^stats displays unreferenced mantra count^** - mantras that exist but are never
> referenced anywhere. high numbers might indicate orphaned knowledge.

> **^stats shows reference distribution histogram^** - a visual histogram showing how
> references are distributed. some mantras are heavily referenced, others rarely.

> **^histogram uses max 10 buckets^** - the distribution groups reference counts into
> at most 10 buckets. actual bucket count depends on the data range.

> **^empty edge buckets are hidden^** - if the first or last buckets have zero entries,
> they're omitted for cleaner output.

> **^stats accepts a path argument^** - like check, you can pass a path to analyze a
> specific folder: `vyasa stats ./docs`.

> **^stats shows bucket ranges^** - each histogram row shows the reference count range
> (e.g., "1-3 refs") and a bar proportional to how many mantras fall in that range.

> **^bucket width adapts to data^** - if all mantras have 1-5 references, buckets might
> be 1-1, 2-2, etc. if range is 1-100, buckets might be 1-10, 11-20, etc.
