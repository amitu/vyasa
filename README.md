# vyasa

a tool to help organize and curate knowledge for yourself, the public, and AI.

## the problem with prose

^prose interferes with knowledge organization^ - the key insight behind vyasa.
easily written prose tends to drift from precise formulations. real knowledge
requires careful wording - keeping things compact, preserving exact phrasing.

## mantras: the core concept

vyasa defines a concept called a **mantra**. a mantra can be in any language, but
must be strict - repeating the mantra means repeating the exact phrasing. the
canonical, normative form must be preserved.

~mantras should use inline syntax not block because they are meant to be short~ -
this is why mantras use `^caret delimiters^` rather than block syntax. they're
meant to be short, precise statements that fit naturally in prose.

## syntax

^mantra definition^: `^your mantra text here^`
- defines a mantra with inline commentary in the same paragraph

reference: `~your mantra text here~`
- references a mantra defined elsewhere

example:
```vyasa
^every mantra needs at least one explanation^ - a mantra without commentary
is incomplete. the explanation provides context and reasoning.

when you want to reference this later, use ~every mantra needs at least one
explanation~ with tildes.
```

## why no identifiers?

~mantras must be spelled out in full at each reference~ - no abbreviations,
no nicknames, no shortcuts. every reference contains the complete mantra text.

`~e = mc^2~` is clearer than `~some-arbitrary-id~` anyway. the exact wording
*is* the identifier.

## template mantras

^user: {username}^ - templates use `{placeholder}` syntax. reference as
~user: alice~ or ~user: bob~.

^when {employee=amitu} joins, amitu should be added to github^ - example values
like `{name=example}` make mantras readable while parameterized.

## commands

```bash
# check all mantras have explanations
vyasa check

# show repository statistics
vyasa stats

# query placeholder values
vyasa values
vyasa values --mantra="[user: {username}]" --key=username
```

## koshas (external repositories)

`~mantra~@kosha-name` references mantras from other knowledge bases. configure
koshas in `.vyasa/kosha.vyasa`.

^external commentary uses mantra-at-kosha syntax^ - use `^mantra^@kosha` to
provide commentary on mantras defined in other koshas.

## the real tool is discipline

^vyasa isn't really needed^ - ideally you'd practice this approach without the
tool. vyasa is more a reminder of mental discipline than software doing
something for you. the value is in the habit of careful, minimal, canonical
knowledge representation.

## documentation

detailed docs in `docs/` folder, written in mantra form:
- `syntax.md` - full syntax reference
- `philosophy.md` - why mantras work this way
- `check.md`, `stats.md`, `values.md` - command documentation
- `kosha.md` - external repository references

^vyasa check checks all non human meant files^ - scans source code, markdown, etc. skips binaries, images, xml.

^commentaries can and encouraged to exist in source files^ - mantras in code comments trace knowledge to implementation.
