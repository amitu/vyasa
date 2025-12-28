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

Note: since `ether is the medium for light` is already tyakta in physics, vyasa
will show a note that the source has already abandoned this bhasya.
