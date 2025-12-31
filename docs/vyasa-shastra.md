# Vyasa Shastra

This document defines vyasa's core concepts using vyasa's own format.

---

## Core Concepts

> **^shastra^** - a repository of knowledge containing mantras and bhasyas.

> **^mantra^** - a concise, referenceable unit of knowledge owned by exactly
> one _| shastra |_.

> **^bhasya^** - a quote block containing one or more _| mantra |_ references
> with commentary.

> **^anusrit^** - a non-authoritative reference to a _| mantra |_ using
> `_| mantra text |_` syntax.

> **^mula mantra^** - an authoritative reference to a _| mantra |_ in a
> _| bhasya |_ using `**^mantra text^**` syntax. When studying a _| mantra |_,
> _| bhasya |_ containing _| mula mantra |_ references should be studied first.

---

## Shastra Relationships

> **^observed shastra^** - a _| shastra |_ listed in `.vyasa/shastra.json` that
> the current _| shastra |_ can reference. The current _| shastra |_ plus all
> _| observed shastra |_ form the _| resolution scope |_.

> **^resolution scope^** - the set of _| shastra |_ (current + observed) used
> to resolve unqualified _| mantra |_ references.

---

## Mantra Ownership

> **^mantra ownership^** - every _| mantra |_ is owned by exactly one
> _| shastra |_. The same mantra text can exist in multiple _| shastra |_ as
> independent _| mantra |_.

> **^mantra ambiguity^** - when the same mantra text exists in multiple
> _| shastra |_ within the _| resolution scope |_, references to it are
> ambiguous and require _| shastra qualifier |_.

> **^shastra qualifier^** - the `@shastra-name` suffix that explicitly
> specifies which _| shastra |_ a _| mantra |_ belongs to. Bypasses
> _| mantra resolution |_.

---

## Resolution

> **^mantra resolution^** - the process of determining which _| shastra |_ an
> unqualified _| mantra |_ reference belongs to. Searches the
> _| resolution scope |_ for a unique match.

> **^resolution algorithm^** - for unqualified _| mantra |_ references:
> 1. Collect all _| shastra |_ in _| resolution scope |_
> 2. Find which define the mantra text
> 3. If exactly one: resolved
> 4. If zero: error (undefined)
> 5. If multiple: error (_| mantra ambiguity |_, use _| shastra qualifier |_)

---

## Bhasya Types

> **^mula bhasya^** - a _| bhasya |_ with original commentary containing at
> least one _| mula mantra |_. The mantra may be native to this _| shastra |_
> or from an _| observed shastra |_.

> **^uddhrit bhasya^** - a _| bhasya |_ prefixed with `shastra: name` that
> quotes another _| shastra |_'s _| bhasya |_ (agreement/citation).

> **^khandita bhasya^** - a _| bhasya |_ prefixed with `khandita: name` that
> refutes another _| shastra |_'s _| bhasya |_.

> **^tyakta bhasya^** - a _| bhasya |_ prefixed with `tyakta:` that deprecates
> a _| mantra |_ in this _| shastra |_.

---

## Bhasya Prefix Independence

> **^bhasya prefix independence^** - _| bhasya |_ prefixes (`shastra:`,
> `khandita:`, `tyakta:`) affect _| bhasya |_ type but do NOT affect
> _| mantra resolution |_. Each _| mantra |_ inside is resolved independently
> using the same _| resolution algorithm |_.

---

## Syntax Reference

> **^mula syntax^** - `**^mantra text^**` for _| mula mantra |_ in this
> _| shastra |_, or `**^mantra text^**@shastra` for explicit _| shastra |_.

> **^anusrit syntax^** - `_| mantra text |_` for _| anusrit |_ resolved via
> _| resolution algorithm |_, or `_| mantra text |_@shastra` for explicit.

---

## Data Model

> **^mantra key^** - _| mantra |_ are stored with key `(mantra_text, shastra_name)`.
> After _| mantra resolution |_, the _| shastra |_ is always explicit.

> **^mantra info^** - metadata for a _| mantra |_: file, line, has_explanation,
> list of _| bhasya |_ indices where it appears as _| mula mantra |_ or
> _| anusrit |_.

---

## Scoping Principle

> **^local scoping^** - each _| shastra |_ is self-consistent within its own
> _| resolution scope |_. An observer who follows multiple _| shastra |_ may
> see _| mantra ambiguity |_ that individual _| shastra |_ authors don't see.
> Each author only needs to resolve ambiguity within their own scope.
