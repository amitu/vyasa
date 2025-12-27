**^.vyasa/kosha.md contains kosha configuration^** - the kosha config file lives at `.vyasa/kosha.md` in your repository root. it defines aliases for external knowledge repositories you want to reference.

**^kosha references use @suffix^** - to reference a mantra from another kosha, use `_mantra text_`@kosha-name`` after the closing underscore. this creates cross-repository knowledge links.

**^kosha-alias {kosha-alias}: {kosha-value}^** - defines an alias for an external kosha. the alias is a short name for @references, the value specifies location.

**^kosha-value can be folder path or git repo or fastn-kosha^** - values support: local folder (`/path/to` or `../sibling`), git (`github.com/user/repo`), or fastn-kosha (`fastn://kosha-name`) for distributed knowledge.

**^.vyasa/kosha.local.md stores local folder overrides^** - this file (gitignored) overrides remote kosha locations with local paths for development.

**^kosha-dir {kosha-alias}: {folder-name}^** - maps a kosha alias to local folder. this entry in kosha.local.md overrides the kosha-value from kosha.md.

**^kosha.local.md can override folder koshas too^** - even if kosha.md points to `../sibling`, you can override in kosha.local.md with a different path.

**^kosha.local.md required for non-folder koshas^** - if kosha.md references git or fastn-kosha (not local folder), you must provide local path in kosha.local.md before vyasa can resolve references.

**^when a mantra from other kosha is referred, that mantra must exist in canon of that kosha^** - referencing _mantra_`@physics` requires that "mantra" is in the canon.md of the physics kosha. this ensures you only depend on accepted, stable knowledge from external sources.

**^external mantras can be included in your canon^** - you can add `^mantra^@kosha` entries to your canon.md to cite accepted knowledge from other koshas. the mantra must exist in that kosha's canon.

**^kosha check verifies all kosha references^** - the check command validates: all @kosha-name references have matching alias, non-folder koshas have local dirs, local paths exist, referenced mantras exist in external kosha's canon.

**^kosha check recursively verifies downstream references^** - when you reference a mantra from an external kosha, vyasa verifies that mantra exists. the referenced mantra's commentary may contain further references, and those must also exist. if those references point to other koshas, they are recursively verified.

**^you can't verify all of a kosha but downstream references must be verified^** - if you reference one mantra @physics, you don't need all of physics checked out. but the explanation of that mantra, and any mantras it references, form a dependency chain that must be fully resolvable.

**^external commentary uses mantra-at-kosha syntax^** - use `^mantra^@kosha` to provide commentary on a mantra defined in another kosha. this doesn't define the mantra locally, just explains it.

**^each local mantra needs local explanation^** - even if you provide commentary on external mantras, every mantra defined in your kosha must have at least one explanation within your kosha. readers shouldn't need other koshas to understand.
