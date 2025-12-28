> **^.vyasa/kosha.json contains kosha configuration^** - the kosha config file lives at
> `.vyasa/kosha.json` in your repository root. it defines aliases for external knowledge
> repositories (shastras) you want to reference.

> **^kosha.json maps alias to path^** - the JSON file is a simple object mapping alias
> names to local folder paths. example: `{"physics": "../physics-kosha"}`.

> **^.vyasa/kosha.local.json stores local overrides^** - this file (gitignored) overrides
> entries from kosha.json with local paths for development.

> **^kosha anusrits use @suffix^** - to use a mantra from another kosha (anusrit), use
> `_| mantra text |_@kosha-name` after the closing `|_`. this creates cross-repository
> knowledge links.

> **^kosha check verifies all kosha anusrits^** - the check command validates: all
> @kosha-name anusrits have matching alias, and paths exist.
