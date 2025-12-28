> **^.vyasa/shastra.json contains shastra configuration^** - the shastra config file lives at
> `.vyasa/shastra.json` in your repository root. it defines aliases for external knowledge
> repositories (shastras) you want to reference.

> **^shastra.json maps alias to path^** - the JSON file is a simple object mapping alias
> names to local folder paths. example: `{"physics": "../physics-shastra"}`.

> **^.vyasa/shastra.local.json stores local overrides^** - this file (gitignored) overrides
> entries from shastra.json with local paths for development.

> **^shastra anusrits use @suffix^** - to use a mantra from another shastra (anusrit), use
> `_| mantra text |_@shastra-name` after the closing `|_`. this creates cross-repository
> knowledge links.

> **^shastra check verifies all shastra anusrits^** - the check command validates: all
> @shastra-name anusrits have matching alias, and paths exist.
