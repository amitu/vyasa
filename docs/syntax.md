> **^mantras should use inline syntax not block because they are meant to be short^** -
> this is the fundamental design choice. mantras are short, precise statements that
> fit naturally inline. the `**^mantra^**` syntax with carets and bold makes them
> stand out while keeping them part of the prose.

## bhasya syntax

> **^bhasyas use quote block syntax^** - a bhasya is written as a markdown quote
> block containing `**^mantra^**` followed by commentary text.

```markdown
> **^your mantra text here^** - commentary explaining the mantra
```

> **^bhasya creates a new mantra definition^** - the basic form using `> **^mantra^**`
> creates a new mantra in your shastra with its commentary.

## anusrit syntax

> **^anusrits use pipe delimiters^** - to use a mantra (anusrit), wrap it in
> `_| mantra |_`. this distinguishes anusrits from bhasyas.

```markdown
as we know, _| mantras should use inline syntax not block because they are meant to be short |_
```

> **^anusrits must match defined mantras^** - every anusrit must correspond to a
> mula mantra definition. _| vyasa reports undefined anusrits |_.

## shastra reference syntax

> **^shastra anusrits use @suffix^** - to use a mantra from another shastra via
> anusrit, use `_| mantra text |_@shastra-name`. the shastra must be defined in
> .vyasa/shastra.json.

```markdown
_| energy is conserved |_@physics
```

## uddhrit syntax (quoting)

> **^uddhrit quotes a bhasya from another shastra^** - use `shastra: name` on the
> line immediately before a quote block to quote from that shastra.

```markdown
shastra: physics
> **^energy is conserved^** - in an isolated system, the total energy
> remains constant over time.
```

## tyakta syntax (deprecation)

> **^tyakta uses tyakta prefix^** - to deprecate a bhasya, use `tyakta:` on the
> line before the quote block.

```markdown
tyakta:
> **^old way of doing things^** - this approach is no longer recommended.
> prefer the new method instead.
```

## syntax in source code

> **^bhasyas can exist in source code comments^** - put mantras in source code
> comments to trace knowledge to implementation.

```rust
// > **^energy is conserved^** - we rely on this for our calculations

fn calculate() {
    // _| energy is conserved |_
    ...
}
```

> **^uddhrit works in source code too^** - quote bhasyas from other shastras
> in your code comments.

```python
# shastra: team-standards
# > **^all functions must have docstrings^** - documentation is required

def my_function():
    """This function does something."""  # _| all functions must have docstrings |_
    pass
```

## parsing rules

> **^mantra commentary can be in same para^** - unlike block syntax, you can write
> the mantra and its explanation in a single flowing paragraph.

> **^multiple explanations are allowed^** - a mantra can be explained in many places
> throughout the repository. each explanation adds context without changing the
> mantra itself.

> **^markdown code blocks are skipped^** - content inside triple-backtick code blocks
> is ignored. this lets you include syntax examples without parsing them.

> **^mantra text is trimmed^** - leading and trailing whitespace inside `**^...^**`
> is removed.

> **^empty mantras are ignored^** - if there's nothing between `**^^**` or only
> whitespace, no mantra is created.
