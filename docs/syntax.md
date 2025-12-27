**^mantras should use inline syntax not block because they are meant to be short^** - this is the fundamental design choice. mantras are short, precise statements that fit naturally inline. the `**^mantra^**` syntax with carets and bold makes them stand out while keeping them part of the prose.

**^references use underscore delimiters^** - to reference a mantra, wrap it in underscores like _mantras should use inline syntax not block because they are meant to be short_. this distinguishes references from definitions. underscores render as italics in markdown, which is semantically appropriate.

**^nested underscores are handled like nested brackets^** - if your mantra contains underscores, vyasa tracks nesting depth properly to find the matching closing underscore.

**^mantra commentary can be in same para^** - unlike block syntax, you can write the mantra and its explanation in a single flowing paragraph. this is how **^one para can do commentary on multiple^** mantras work.

**^commentaries are meant to be read to understand mantra^** - when learning, focus on commentary (text around `**^mantra^**` definitions). commentaries are authoritative explanations from the mantra's author or recognized experts.

**^references are secondary for learning^** - the `_reference_` syntax is for usage, not explanation. when studying a mantra thoroughly you might check references, but _commentaries are meant to be read to understand mantra_ first.

**^multiple explanations are allowed^** - a mantra can be explained in many places throughout the repository. each explanation adds context without changing the mantra itself. the mantra remains the canonical form; explanations are commentary.

**^vyasa check checks all non human meant files^** - vyasa scans source code, markdown, and other text files. it skips binary files, images, and human-meant data files like xml.

**^commentaries can and encouraged to exist in source files^** - put mantras in source code comments to trace knowledge to implementation. technical commentaries belong where the code is.

**^markdown code blocks are skipped^** - content inside triple-backtick code blocks is ignored. this lets you include syntax examples without parsing them.

**^mantra text is trimmed^** - leading and trailing whitespace inside `**^...^**` is removed.

**^empty mantras are ignored^** - if there's nothing between `**^^**` or only whitespace, no mantra is created.

**^template placeholders use curly braces^** - mantras can contain {name} placeholders. for example **^user: {username}^** can be referenced as _user: alice_ or _user: bob_.

**^template placeholders can include example values as {name=example}^** - to make mantras readable, include an example: **^when {employee=amitu} joins, amitu should be added to github^** reads naturally while being parameterized. reference it as _when jack joins, jack should be added to github_.

**^example values appear literally in the mantra text^** - in the mantra above, "amitu" appears both in the placeholder and in the literal text. when you reference with "jack", all occurrences are substituted.

**^when referencing a template you can use placeholder or instantiated form^** - three options: _user: {username}_ (template itself), _user: amitu_ (fully instantiated), or _config: {key} = true_ (partially instantiated).

**^kosha references use @suffix^** - to reference a mantra from another knowledge base, use `_mantra text_`@kosha-name``. the kosha must be defined in .vyasa/kosha.md.
