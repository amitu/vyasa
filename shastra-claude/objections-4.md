# Objections Round 4

Full review of shastra/ and shastra-amitu/ folders.

---

## RESOLVED (dropping permanently)

| Objection | Resolution |
|-----------|------------|
| "Personal" vs "repository" | Now "repository/collection" + explicit "shastra are personal" bhasya |
| "Authoritative" undefined | Removed from shastra bhasya |
| What does "accepted" mean | Clarified by personal framing - you put whatever you want |
| Worth preserving criterion | Same - no external standard, your shastra, your choice |
| Tutorials vs bhasya | Explained in lines 66-70 of shastra.md |
| "Not a way to make money" | Removed entirely from new version |
| Guru-shishya limiting | Now includes "teacher / guide / adviser" alternatives (line 15) |

---

## SYNTAX ERRORS

### S1. shastra.md line 16: Missing space
```
maintaining the_| shastra |_
```
Should be: `maintaining the _| shastra |_`

### S2. objections.md line 24-25: Inconsistent spacing
```
**^ mantra^**
```
Should be: `**^ mantra ^**`

### S3. mantra-bhashya.md: Old syntax throughout
Uses `**^mantra^**` instead of `**^ mantra ^**` in lines 27, 34-36.

---

## OPEN OBJECTIONS

### O1. Circular definition persists (shastra.md lines 52-54)

> **^ shastra ^** - a repository/collection of knowledge, organized as **^ mantra ^**, **^ bhasya ^** and supporting material...

Reader encounters shastra, mantra, bhasya as undefined terms defining each other. The shastra.md document doesn't define mantra or bhasya before using them in the core definition.

**Options:**
- Define all three together explicitly ("these three concepts are co-dependent")
- Restructure so mantra and bhasya are defined before the shastra bhasya
- Accept circularity as intentional (concepts that can only be understood together)

### O2. Bhasya overload (shastra.md lines 20-24)

One bhasya introduces: `shastra are personal`, `vyasa`, `shastra`, `bhasya`. Four mula mantras. Too dense for a reader to absorb.

### O3. mantra.md has no bhasya

9 lines of prose, no committed definition. The verbatim principle demands commitment. Either:
- Add a bhasya defining mantra
- Merge content into mantra-bhashya.md

### O4. "authoritative / normative / important / agreed upon" (mantra-bhashya.md line 34)

Four different words with different meanings:
- Authoritative = from recognized source
- Normative = prescriptive
- Important = valued (by whom?)
- Agreed upon = consensus

Pick one, or define their relationship. Currently reads as "I couldn't decide so I listed synonyms."

### O5. "every mantra has at least one bhasya" (mantra-bhashya.md line 35-36)

Is this true? The vyasa tool explicitly reports "unexplained mantras" - mantras without bhasyas. So they exist. This mantra seems empirically false.

**Options:**
- Rephrase: "every mantra SHOULD have at least one bhasya"
- Rephrase: "a mantra without bhasya is incomplete"
- Remove if not defensible

### O6. "observed shastra" still undefined

Used in shastra.md line 5 as `_| observed shastra |_` but no bhasya defines it. What makes observation formal? Config file? Mental commitment? Having copied a bhasya?

### O7. Rambling admission (shastra.md line 35)

"you can see evidence of rambling this very paragraph"

Still undermines credibility. The foundational document demonstrating the problem it claims to solve.

### O8. "Do NOT bother reading their words" (objections.md line 20)

Hostile, alienating. Contradicts learning mindset. Consider softening or removing.

### O9. Astika/nastika framing (objections.md lines 47-81)

More nuanced now with examples (lines 77-81), but still presents as binary. Real positions are spectrum. Someone 70% aligned is... what?

Also line 53 typo: "The object does not become" should be "The objection does not become"

---

## DOCUMENTS STATUS

| Document | Status | Action Needed |
|----------|--------|---------------|
| shastra/shastra.md | Good structure, 3 bhasyas | Fix S1, address O1-O2 |
| shastra-amitu/mantra.md | Incomplete, no bhasya | Add bhasya or merge |
| shastra-amitu/mantra-bhashya.md | Has bhasyas, old syntax | Fix S3, address O4-O5 |
| shastra-amitu/objections.md | Rich content, some issues | Fix S2, typo, address O8-O9 |

---

## PRIORITY ORDER

1. **S1-S3**: Syntax fixes (5 minutes)
2. **O3**: Commit to a mantra bhasya - this is foundational
3. **O4**: Pick one word for bhasya definition
4. **O5**: Fix "every mantra has at least one bhasya"
5. **O1**: Acknowledge circular definition explicitly or restructure
6. **O6**: Define "observed shastra"
7. **O7-O9**: Polish (lower priority)

---

## WHAT'S WORKING WELL

- "shastra are personal" comes early and strong
- vyasa purpose bhasya is clear (line 8-10)
- Supporting material vs bhasya distinction explained
- Tyakta/khandita mechanism for living system
- Sanskrit terms objection handled gracefully
- Relativism section preempts absolutist objections
