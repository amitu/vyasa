# vyasa

a tool to help organize and curate knowledge for yourself, the public, and AI.

## the problem with prose

the key idea behind vyasa is that easily written prose tends to get in the way of
organizing knowledge rather than helping it. real knowledge requires careful
wording - keeping things compact, preserving exact formulations for canonical
ideas, while commentaries can be added separately for explanations.

## mantras: the core concept

vyasa defines a concept called a **mantra**. a mantra can be in any language, but
it must be strict - repeating the mantra means repeating the exact phrasing. the
canonical, normative form must be preserved.

this forces all "knowledge" to be expressed minimally. the tool "resists" drift
because any change in wording, no matter how small, becomes a new mantra. if a
new mantra is to be accepted, it should either update all previous references,
or both versions exist in the system - pointing to an ambiguity that needs
resolution.

## repositories and koshas

vyasa operates on a repository. this can be:

- a single folder
- a git repository
- a fastn-kosha (available over p2p, being experimented with in fastn-stac/spatial)

the tool analyzes text files in the repo to extract mantras. these files follow
a mantra syntax that's still being experimented with in this repo.

## mantra syntax (draft proposal)

here's what a vyasa file might look like:

```vyasa
--
e = mc^2
--

this is a vyasa file containing a mantra, delimited by -- above.
```

### defining mantras

```vyasa
--
each mantra starts and ends with lines containing --
--
```

here we've created a mantra. to reference it elsewhere, we spell it out exactly:
[each mantra starts and ends with lines containing --].

### why no identifiers?

after some thought, mantras won't have ids. every reference must fully spell out
the mantra. this might seem tedious, but it's intentional:

```vyasa
--
mantras must be spelled out in full at each reference
--
```

[e = mc^2] is probably clearer than [some-arbitrary-name-for-einstein-formula]
anyway. the exact wording *is* the identifier.

### mantra explanations

```vyasa
--
every mantra needs at least one explanation
--
```

an explanation is simply prose that follows a mantra definition. a mantra can be
explained multiple times in different places - that's fine. what's not allowed is
using a mantra that's never been explained anywhere.

importantly, explanations are non-normative. they must not encode knowledge
themselves - just explain it. humans and AI should refer to the original mantras
for canonical knowledge, not the explanations.

### mantra redefinition

```vyasa
--
mantras should not change once defined
--
```

but sometimes you need to update a mantra's wording. when this happens, all past
references should be updated to the new definition. if there are commentaries
about old definitions worth preserving, you have options:

- give the old meaning a new mantra with different wording
- update all references to the old meaning
- then introduce the new mantra with its new meaning

this is intentionally a deliberate, somewhat complicated process. that's kind of
the point.

### what vyasa actually does

vyasa assists this process, though only a little. it can't do much beyond:
- tracking all mantra definitions (storing hashes in a `.vyasa` folder)
- verifying mantras are spelled out exactly when referenced
- printing stats about how many mantras exist
- showing how mantras have been added over time (git and kosha aware)
- tracing mantra additions and changes in a limited way

## commands

```vyasa
--
vyasa check verifies all mantras have explanations
--

the check command scans your repository and reports any mantras that lack
explanations. [every mantra needs at least one explanation], and this command
enforces that rule.

--
vyasa stats shows repository statistics
--

the stats command provides an overview of your knowledge repository: how many
mantras exist, how many have explanations, reference counts, and a histogram
showing which mantras are referenced most frequently.

--
vyasa values cli can query placeholder in file/directory, and filter mantras or even keys
--

the values command extracts instantiated placeholder values from template
references. filter by template with -t or by key with -k.
```

### usage examples

```bash
# check the current directory
vyasa check

# check a specific path
vyasa check ./docs

# show stats with default histogram (max 10 buckets)
vyasa stats

# show stats with custom bucket count
vyasa stats --buckets 5

# show individual reference counts (no bucketing)
vyasa stats --buckets 0

# show all placeholder values
vyasa values

# filter by mantra reference
vyasa values --mantra="[user: {username}]"

# filter by placeholder key
vyasa values --key=username

# combine mantra and key filter on specific file
vyasa values --mantra="[config: {key} = {value}]" --key=key ./docs
```

### the real tool is discipline

```vyasa
--
vyasa isn't really needed
--
```

ideally you'd practice this approach without the tool at all. vyasa is more a
reminder of a mental discipline than software doing something for you. the value
is in the habit of careful, minimal, canonical knowledge representation - not in
the tool that checks your work.

## documentation

detailed documentation is available in the `docs/` folder, written in mantra form.
