# Examples of Cross-Shastra References

This file demonstrates uddhrit (quoting from other shastras).

## Valid uddhrit

shastra: physics
> **^energy is conserved^** - quoting this fundamental law from the physics shastra

## Using quoted mantra via anusrit

When implementing calculations, we rely on _| energy is conserved |_ (no @physics needed
since it's unambiguous - only defined in physics shastra).

## Quoting a tyakta bhasya (error)

Quoting a tyakta bhasya results in an error:

```markdown
shastra: physics
> **^ether is the medium for light^** - this bhasya is tyakta in physics
```

This fails because `ether is the medium for light` is tyakta in the physics shastra.

## Refuting a bhasya (khandita)

khandita: physics
> **^ether is the medium for light^** - we refute this: the Michelson-Morley
> experiment disproved the existence of luminiferous ether.

Note: even though `ether is the medium for light` is tyakta in physics, this
khandita remains valid - our refutation may have contributed to the tyakta.

## Conflict: can't both khandita and uddhrit same bhasya (error)

If you khandita a bhasya, you cannot also uddhrit it - that would be contradictory:

```markdown
khandita: physics
> **^ether is the medium for light^** - we refute this

shastra: physics
> **^ether is the medium for light^** - but also quoting it? ERROR!
```

Once you refute a bhasya, you must reject it consistently everywhere.
