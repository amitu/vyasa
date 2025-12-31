# Objections Round 3

After reviewing updates to shastra/shastra.md, shastra-amitu/objections.md, and shastra-amitu/mantra-bhashya.md.

---

## RESOLVED (dropping these)

| # | Objection | Resolution |
|---|-----------|------------|
| 1 | "Personal" vs "repository" contradiction | New bhasya (line 61): "shastra are personal" - explicit now |
| 2 | "Authoritative" according to whom? | Removed from shastra bhasya entirely |
| 3 | Tutorials vs bhasya unclear | Lines 55-59 now explain supporting material vs bhasya |
| 7 | Understand person vs knowledge | "personal accepted knowledge" framing clarifies this |

---

## SYNTAX ERRORS (fix these)

### S1. shastra.md line 71-73: Malformed mantra syntax

```
**^ vyasa helps track personal accepted knowledge **^
```

The closing should be `^**` not `**^`. Currently: `knowledge **^` should be `knowledge ^**`

### S2. objections.md line 24-25: Inconsistent spacing

```
**^ mantra^**
```

Should be `**^ mantra ^**` (space before closing caret).

### S3. mantra-bhashya.md: Old syntax throughout

Still uses `**^mantra^**` instead of `**^ mantra ^**`. Needs consistency pass.

---

## STRUCTURAL OBJECTIONS

### O1. Circular definition in shastra bhasya (line 41-43)

> **^ shastra ^** - a repository of knowledge, organized as **^ mantra ^**,
> **^ bhasya ^** and supporting material...

You're defining shastra in terms of mantra and bhasya, but those aren't defined yet in this document. A reader encountering this for the first time learns nothing. They see three undefined terms used to define each other.

**Challenge:** Can you define shastra without reference to mantra/bhasya? Or must these three be defined together as a unit?

### O2. Bhasya overload (lines 61-65)

This single bhasya introduces THREE mula mantras:
- `**^ shastra are personal ^**`
- `**^ vyasa ^**`
- `**^ shastra ^**` (again)
- `**^ bhasya ^**` (again)

That's too dense. The reader can't focus. Each mantra deserves its own bhasya with focused commentary.

**Challenge:** Split this into separate bhasyas, one concept each.

### O3. "observed shastra" introduced but undefined (line 69)

You write `_| observed shastra |_` but never define it. What makes a shastra "observed"? Is it:
- Explicitly declared in a config file?
- Any shastra you've read?
- Shastras you've copied bhasyas from?

### O4. Copy-paste semantics unclear (lines 64-65, 68)

"copy-paste the specific bhasya one has just learnt"

Questions this raises:
- If I copy a bhasya containing `**^ X ^**`, do I now "own" mantra X?
- What's the difference between copying and endorsing?
- What about attribution? Must I cite the source shastra?
- Can I modify the copied bhasya? Then is it still "that" bhasya?

---

## PHILOSOPHICAL OBJECTIONS (from original list, still open)

### O5. What does "accepted" mean? (original #4)

Line 3: "collection of knowledge accepted by a person"

Accepted how? Possibilities:
- Believed to be true
- Found useful in practice
- Endorsed for others
- Merely bookmarked for reference

These are very different. A shastra of "things I find useful" differs from "things I believe are true" differs from "things I recommend to my students."

### O6. Guru-shishya framing is culturally loaded (original #5)

Lines 5-6: "pupils, students, who declare this person their personal guru"

This assumes:
- Hierarchical knowledge transmission
- Explicit declaration of allegiance
- Single-guru model

What about:
- Peer learning (study groups, collaborators)?
- Multi-source learning (I learn from 20 people, none is "my guru")?
- Autodidacts (I maintain a shastra for myself, no students)?

The framing excludes valid patterns of knowledge work.

### O7. "Not a way to make money" is arbitrary (original #6)

Line 11: "shastra is not a way to make money"

Why not? If I write a shastra, publish it as a book, and sell it - does it cease to be shastra? The Vedas were transmitted for millennia. Now they're sold in bookstores. Are they no longer shastra?

This feels like your personal preference smuggled into the definition.

### O8. Admitting rambling undermines the system (original #8)

Lines 20-24: You demonstrate the very problem you're trying to solve, in the document proposing the solution. "you can see evidence of rambling this very paragraph."

If the author of vyasa can't maintain discipline in the foundational document, why would anyone trust the system? This is like a diet book author being photographed eating cake on the cover.

**Counter-argument you might make:** "The rambling is in supporting material, not in bhasyas. Bhasyas are disciplined."

**My response:** Then the rambling material shouldn't be in the canonical shastra at all. Edit it down or move it to a "development notes" document.

### O9. No criterion for "worth preserving" (original #9)

You repeatedly say we need to "call out" things worth reading/remembering/propagating. But what makes something worth it?

- Truth? (But you allow shastras to redefine terms, so "truth" is relative)
- Usefulness? (Useful to whom? For what?)
- Beauty? (Subjective)
- Memorability? (Circular - memorable because we marked it memorable)

Without a principle, the system reduces to "I mark what I feel like marking."

### O10. Mountain of text not reduced (original #10)

The problem (line 30-31): "faced with a mountain of text... not going to have time to wade through"

Your solution: Add bhasya markers to some paragraphs.

But this doesn't reduce the mountain. It just puts flags on some peaks. The reader still faces 10,000 words. Now they know 500 words are "important" - but they still have to wade through 10,000 to find them.

**Real solutions would be:**
- Extract bhasyas into a separate index (vyasa could generate this)
- Limit supporting material ruthlessly
- Provide multiple reading paths (bhasya-only vs full document)

### O11. Why quote blocks specifically? (original #11)

You never justify the `>` syntax. Why not:
- Bold paragraphs?
- Special headers (`## Bhasya: mantra text`)?
- XML-style tags (`<bhasya>...</bhasya>`)?
- Indentation?

Quote blocks have specific semantics in writing - they usually mean "someone else said this." Using them for "I'm saying this importantly" is a semantic mismatch.

---

## NEW OBJECTIONS FROM objections.md

### O12. "Do NOT bother reading their words" is hostile (line 20)

This alienates:
- Curious skeptics who might convert
- People doing comparative study
- Researchers

It also contradicts the learning mindset. You learn from disagreement too. Dismissing all non-devotees as "fools" is intellectually lazy and rhetorically weak.

### O13. Astika/nastika binary is too crude (lines 47-72)

Real positions exist on a spectrum:
- "I agree with 90%, quibble with 10%" - astika or nastika?
- "I'm neutral, just studying" - which bucket?
- "I agree with your method but not your conclusions" - ?

The binary creates an us-vs-them mentality that discourages nuanced engagement.

### O14. "authoritative / normative / important / agreed upon" (mantra-bhashya.md line 34)

This is still a list of four different things:
- **Authoritative** = comes from recognized source
- **Normative** = prescribes how things should be
- **Important** = matters (to whom?)
- **Agreed upon** = consensus exists

These are NOT synonyms. A bhasya can be authoritative but not agreed upon (controversial expert opinion). It can be agreed upon but not normative (descriptive consensus).

Pick ONE. Or define the relationship between them.

---

## SUMMARY: Top 5 to address next

1. **S1-S3**: Fix syntax errors (quick win)
2. **O1**: Resolve circular definition - either define shastra independently or explicitly acknowledge the three concepts must be introduced together
3. **O5**: Define "accepted" - this is foundational
4. **O8**: Either edit down the rambling or justify why it belongs
5. **O14**: Pick one word instead of "authoritative / normative / important / agreed upon"
