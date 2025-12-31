# Mantra-Shastra Data Model

## Core Concepts

### Shastra

A shastra is a repository of knowledge - a collection of mantras and bhasyas. Each shastra:
- Has a name (defined in `.vyasa/config.json`)
- Can observe other shastras (defined in `.vyasa/shastra.json`)
- Is self-contained but can reference external mantras

### Mantra

A mantra is a concise, referenceable unit of knowledge. A mantra is always owned by
exactly one shastra.

Mantra with same text can belong to multiple shastra, in which case when they are
referred in a shastra, which observes multiple shahstras (inclusing self and one other shastra) defining their own mantra with same text, they must use @<shastra-alias> to disambiguate.

### Bhasya

A bhasya is a quote block containing one or more mantras with commentary.

Types of bhasya:

- `Mula` - defines original commentary with at least one mula mantra (which may 
  be native to this shashtra or be in one of the observed shashtra).
- `Uddhrit` - quotes bhasya from another shastra (agreement/usage)
- `Khandita` - refutes bhasya from another shastra
- `Tyakta` - deprecates a bhasya in this shastra

A bhasya can refer to mantras either via mula mantra syntax, **^mula mantra^**,
meaning it is making an authoritative definition / clarification on that mula
mantra. Bhasya can also use _| anusrit mantra |_, syntax, where it is not trying
to make a clarification on which this anusrit mantra, it assumes you know what
this is, or you refer to other bhasyas for what this anusrit mantra is, it is
just referring to it. 

When studying a mantra, mula bhasyas, meaning all bhasyas that used this mantra
using mula syntax should be studied first to understand what this mantra stands
for. There can be a lot of anusrits references to this mantra, and they are
not authoritative/normative about this mantra. 

### Anusrit

An anusrit is a reference to a mantra (inline usage via `_| mantra |_` syntax).

---

## Syntax

### Mula Mantra (Unqualified)

```markdown
> **^mantra text^** - commentary explaining the mantra
```
Resolved by looking at current shastra + all observed shastras.
Must be unique in scope, otherwise use @shastra to disambiguate.

### Mula Mantra (Qualified)
```markdown
> **^mantra text^**@other-shastra - commentary about their mantra
```
Explicitly references "mantra text" from "other-shastra". Bypasses resolution.

### Anusrit (Inline Reference)
```markdown
As we discussed in _| some concept |_, this applies to...
```
References "some concept". Resolved within local scope.

### Anusrit with Explicit Shastra
```markdown
According to _| some concept |_@other-shastra, we should...
```
Explicitly references "some concept" from "other-shastra".

---

## Scoping Rules

### The @shastra Qualifier

**When is @shastra required?**
- Only when there's ambiguity among shastras YOU observe
- If mantra text is unique across your observed shastras, @shastra is optional

**Scoping is LOCAL to each shastra:**
- Each shastra has its own `.vyasa/shastra.json` defining observations
- When parsing my shastra, only MY shastra.json matters
- Unqualified references resolve within MY observed shastras

### Observer's Perspective

If someone observes my shastra + another shastra:
- My unqualified references remain valid (they resolve in MY scope)
- The observer, when writing THEIR shastra, must qualify if THEY see conflicts
- Each shastra is self-consistent within its own observation set

### Example

```
My shastra observes: A, B
Shastra A has: "foo", "bar"
Shastra B has: "baz"

In my shastra:
  _| foo |_      → resolves to A (only place it exists)
  _| baz |_      → resolves to B (only place it exists)

If B also had "foo":
  _| foo |_      → ERROR: ambiguous
  _| foo |_@A    → OK: explicit
  _| foo |_@B    → OK: explicit
```

---

## Data Model

### Mantra Key

Mantras are keyed by `(mantra_text, shastra_name)`:

```rust
pub mantras: HashMap<(String, String), MantraInfo>,
```

- Always explicit shastra name after resolution
- Unqualified mantras resolved: find unique match in (current + observed shastras)
- Qualified mantras (`@shastra`): shastra name taken directly

### MantraInfo

```rust
pub struct MantraInfo {
    /// First definition location (for display)
    pub file: String,
    pub line: usize,
    /// Whether this mantra has commentary
    pub has_explanation: bool,
    /// Bhasya indices where this is a mula definition
    pub mula_bhasyas: Vec<usize>,
    /// Bhasya indices where this is referenced inside a bhasya
    pub anusrit_bhasyas: Vec<usize>,
}
```

### Anusrit

```rust
pub struct Anusrit {
    pub mantra_text: String,
    pub shastra: Option<String>,  // None = unqualified, needs resolution
    pub file: String,
    pub line: usize,
    pub bhasya_index: Option<usize>,  // Some if inside a bhasya
}
```

- `shastra: None` → unqualified, resolve using resolution algorithm
- `shastra: Some(x)` → qualified with @x, validate against shastra x

---

## Parsing and Resolution

When parsing, we track context for resolution:

```rust
impl Repository {
    pub fn parse(path: &Path) -> Result<Self, String> {
        // Get THIS shastra's name and observed shastras
        let current_shastra = config.name.unwrap_or_else(|| derive_from_path(path));
        let observed_shastras = load_shastra_json(path);

        // Parse files - resolution happens here or deferred to validation
        for file in files {
            parse_file(file, &current_shastra, &observed_shastras, &mut repo);
        }

        Ok(repo)
    }
}
```

**Resolution algorithm (for unqualified mantras):**
1. Collect all shastras in scope: current + observed
2. Find which shastras define this mantra text
3. If exactly one → resolved to that shastra
4. If zero → error: undefined mantra
5. If multiple → error: ambiguous, use @shastra

**Qualified mantras (`@shastra`):**
- No resolution needed, shastra is explicit
- Validation: check mantra exists in that shastra

---

## Validation

Both mula mantras and anusrits follow the same resolution/validation rules:

### Unqualified (`**^mantra^**` or `_| mantra |_`)

Uses the resolution algorithm:
1. Collect scope: current shastra + observed shastras
2. Find which define this mantra text
3. Exactly one → valid, resolved to that shastra
4. Zero → error: undefined mantra
5. Multiple → error: ambiguous, use @shastra

### Qualified (`**^mantra^**@X` or `_| mantra |_@X`)

1. Resolve X via shastra.json aliases
2. Parse shastra X (or use cached)
3. Check if mantra exists in X
4. If not found → error: mantra not in shastra X

---

## Bhasya Prefixes

The bhasya prefix (`shastra:`, `khandita:`, `tyakta:`) affects the bhasya's kind,
but mantras inside follow the same resolution rules:

```markdown
shastra: X
> **^foo^** - "foo" is resolved: if unique in scope, OK; if ambiguous, ERROR
> **^foo^**@X - explicitly X's foo (bypasses resolution)
> **^foo^**@Y - explicitly Y's foo (can reference different shastras in same bhasya)
```

The prefix indicates the BHASYA's relationship to another shastra (quoting, refuting).
Mantra resolution is always the same algorithm - find unique match in
(current shastra + all observed shastras). Use @shastra to be explicit or
when there's ambiguity.

---

## File Structure

```
my-shastra/
├── .vyasa/
│   ├── config.json      # { "name": "my-shastra" }
│   └── shastra.json     # { "aliases": { "other": "../other-shastra" } }
├── docs/
│   └── concepts.md      # Contains mantras and bhasyas
└── ...
```
