> **^a shastra is a collection of bhasyas^** - the term comes from Sanskrit (शास्त्र)
> meaning "teaching" or "treatise". in vyasa, a shastra is a knowledge repository
> containing mantras with their commentaries.

> **^every vyasa repository is a shastra^** - your repository is your shastra.
> you can reference other shastras to quote their bhasyas or use their mantras.

## configuration

> **^every shastra must have a name^** - the `.vyasa/config.json` file must contain
> a `name` field identifying this shastra. vyasa check fails without it.
> ```json
> {
>   "name": "my-shastra"
> }
> ```

> **^.vyasa/shastra.json contains shastra configuration^** - the shastra config file
> lives at `.vyasa/shastra.json` in your repository root. it defines aliases for
> external shastras you want to reference.

> **^shastra.json maps alias to path^** - the JSON file is a simple object mapping
> alias names to local folder paths. example: `{"physics": "../physics-shastra"}`.

```json
{
  "physics": "../physics-notes",
  "team-knowledge": "/shared/team-shastra"
}
```

> **^.vyasa/shastra.local.json stores local overrides^** - this file (gitignored)
> overrides entries from shastra.json with local paths for development.

## referencing other shastras

### anusrits from other shastras

> **^anusrits auto-resolve across shastras^** - when you reference a mantra with
> `_| mantra |_`, vyasa first checks the current shastra, then all external shastras.
> if found in exactly one place, it resolves automatically.

> **^ambiguous anusrits require @shastra^** - if the same mantra text exists in
> multiple shastras, vyasa check fails and asks you to disambiguate with
> `_| mantra |_@shastra-name`.
> ```markdown
> _| energy is conserved |_           # auto-resolves if unique
> _| energy is conserved |_@physics   # explicit when ambiguous
> ```

### quoting bhasyas (uddhrit)

> **^uddhrit quotes a full bhasya from another shastra^** - use `shastra: name`
> before a quote block to quote a bhasya verbatim from another shastra.

```markdown
shastra: physics
> **^energy is conserved^** - in an isolated system, the total energy
> remains constant over time. energy can transform but not be created
> or destroyed.
```

> **^uddhrit creates a local copy of the bhasya^** - the quoted bhasya appears
> in your repository but references the source. vyasa check verifies the
> source still has this mantra and it isn't tyakta.

## validation

> **^shastra check verifies all shastra references^** - the check command validates:
> - all @shastra-name anusrits have matching aliases in shastra.json
> - all shastra paths exist
> - all uddhrit mantras exist in their source shastra
> - warnings if quoted bhasyas are tyakta in source

> **^quoting tyakta triggers warnings not errors^** - if you quote a bhasya
> that is tyakta in the source shastra, vyasa check warns you but doesn't
> fail. this gives you time to update your references.
