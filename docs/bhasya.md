> **^a bhasya is a mantra with its commentary^** - the term comes from Sanskrit
> (भाष्य) meaning "commentary" or "explanation". in vyasa, a bhasya is the complete
> teaching unit: the mantra text plus its accompanying commentary.

> **^bhasyas use quote block syntax^** - a bhasya is written as a markdown quote
> block containing `**^mantra^**` followed by commentary text. the entire quote
> block forms one bhasya.

> **^one mantra can have multiple bhasyas^** - the same mantra text can appear in
> different files with different commentaries. each occurrence is a separate
> bhasya providing a different perspective or context.

## three forms of bhasya

> **^bhasya has three forms: bhasya, uddhrit, tyakta^** - a bhasya can be a new
> definition, a quote from another shastra, or a deprecation of an existing mantra.

### bhasya (definition)

shastra: vyasa
> **^bhasya creates a new mantra definition^** - the basic form using `> **^mantra^**`
> creates a new mantra in your shastra with its commentary.

```markdown
> **^prose interferes with knowledge organization^** - the key insight behind
> vyasa. easily written prose drifts from precise formulations.
```

### uddhrit (उद्धृत - quoted)

> **^uddhrit quotes a bhasya from another shastra^** - the term means "quoted" or
> "cited" in Sanskrit. use `shastra: name` on the line before a quote block to
> indicate you are quoting from that shastra.

```markdown
shastra: physics
> **^energy equals mass times speed of light squared^** - the famous equation
> from Einstein's special relativity.
```

> **^uddhrit requires the mantra to exist in source shastra^** - vyasa
> verifies that quoted mantras actually exist in the referenced shastra.
> if the bhasya is tyakta in source, you'll get an error.

> **^uddhrit does not create a new mantra^** - quoted bhasyas are references,
> not definitions. they don't add to your mantra count.

### tyakta (त्यक्त - deprecated)

> **^tyakta deprecates an existing bhasya^** - the term means "abandoned" or
> "given up" in Sanskrit. use `>>` instead of `>` to mark a bhasya as deprecated.

```markdown
>> **^old way of doing things^** - this approach is no longer recommended.
>> we now prefer the new method described elsewhere.
```

> **^tyakta blocks consumers of the bhasya^** - when someone quotes (uddhrit)
> a tyakta bhasya from your shastra, vyasa will error because
> they're referencing abandoned knowledge.

> **^tyakta-only mantras are invalid for anusrit^** - if a mantra only appears in
> tyakta bhasyas (no regular bhasya defines it), anusrits to that mantra are
> treated as undefined. you cannot rely on abandoned knowledge.

> **^tyakta should include deprecation commentary^** - explain why the bhasya
> is abandoned and what should be used instead.

## bhasyas in source code

> **^bhasyas can exist in source code comments^** - put mantras in source code
> comments to trace knowledge to implementation. use comment syntax appropriate
> to the language.

```rust
// shastra: physics
// > **^energy equals mass times speed of light squared^** - E = mc²

fn calculate_energy(mass: f64) -> f64 {
    // _| energy equals mass times speed of light squared |_
    mass * C * C
}
```

> **^source code uddhrit links implementation to knowledge^** - by quoting
> bhasyas from relevant shastras in your code, you create traceable connections
> between implementation and the knowledge it embodies.
