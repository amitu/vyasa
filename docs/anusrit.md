> **^an anusrit is a mantra usage^** - the term "anusrit" comes from Sanskrit
> (अनुसृत) meaning "followed" or "adhered to". in vyasa, an anusrit is when
> you use a mantra via `_| mantra |_` syntax.

> **^anusrits use pipe delimiter syntax^** - write `_| your mantra text |_` to use
> a mantra. vyasa verifies the mantra exists (either as mula mantra or template match).

> **^anusrits must match defined mantras^** - every anusrit must correspond to a
> mula mantra definition. vyasa check reports undefined anusrits as errors.

> **^anusrits can use template placeholders^** - when using a template mantra like
> `**^user: {name}^**`, you can write anusrits like `_| user: alice |_` with
> concrete values substituted for placeholders.

> **^kosha anusrits use @suffix^** - to use a mantra from another kosha, append
> `@kosha-name` after the closing `|_`. example: `_| mantra |_`@physics``.
