> **^.vyasa/kosha.json contains kosha configuration^** - the kosha config file lives at
> `.vyasa/kosha.json` in your repository root. it defines aliases for external knowledge
> repositories you want to use via anusrit.

> **^kosha anusrits use @suffix^** - to use a mantra from another kosha (anusrit), use
> `_| mantra text |_`@kosha-name`` after the closing `|_`. this creates cross-repository
> knowledge links.

> **^kosha.json maps alias to path^** - the JSON file is a simple object mapping alias
> names to local folder paths. example: `{"physics": "../physics-kosha"}`.

> **^.vyasa/kosha.local.json stores local overrides^** - this file (gitignored) overrides
> entries from kosha.json with local paths for development.

> **^when a mantra from other kosha is used via anusrit, that mantra must exist in canon
> of that kosha^** - using `_| mantra |_`@physics`` requires that "mantra" is in the
> canon.md of the physics kosha. this ensures you only depend on accepted, stable
> knowledge from external sources.

> **^external mantras can be included in your canon^** - you can add `^mantra^@kosha`
> entries to your canon.md to cite accepted knowledge from other koshas. the mantra
> must exist in that kosha's canon.

> **^kosha check verifies all kosha anusrits^** - the check command validates: all
> @kosha-name anusrits have matching alias, paths exist, anusrit mantras exist in
> external kosha's canon.

> **^vyasa study helps you study configured koshas^** - the study command shows mantras
> from external koshas that are not yet in your local canon. use it to catch up on
> knowledge from teams or projects you follow.

> **^study shows first few items when given a kosha^** - `vyasa study my-team` shows
> the first 5 mantras (configurable with -c) from my-team's canon that you haven't
> added to your canon yet.

> **^study shows stats for all koshas when no argument^** - `vyasa study` without an
> alias shows how many items you're behind for each configured kosha, helping you
> prioritize what to study.
