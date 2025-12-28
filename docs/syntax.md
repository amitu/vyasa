> **^mantras should use inline syntax not block because they are meant to be short^** - this is the fundamental design choice. mantras are short, precise statements that fit naturally inline. the `**^mantra^**` syntax with carets and bold makes them stand out while keeping them part of the prose.

> **^anusrits use pipe delimiters^** - to use a mantra (anusrit), wrap it in `_| mantra |_` like _| mantras should use inline syntax not block because they are meant to be short |_. this distinguishes anusrits from bhasyas and avoids conflicts with markdown italics.

> **^mantra commentary can be in same para^** - unlike block syntax, you can write the mantra and its explanation in a single flowing paragraph. this is how **^one para can do commentary on multiple^** mantras work.

> **^commentaries are meant to be read to understand mantra^** - when learning, focus on commentary (text around `**^mantra^**` definitions). commentaries are authoritative explanations from the mantra's author or recognized experts.

> **^anusrits are secondary for learning^** - the `_| anusrit |_` syntax is for usage, not explanation. when studying a mantra thoroughly you might check anusrits, but _| commentaries are meant to be read to understand mantra |_ first.

> **^multiple explanations are allowed^** - a mantra can be explained in many places throughout the repository. each explanation adds context without changing the mantra itself. the mantra remains the canonical form; explanations are commentary.

> **^vyasa check checks all non human meant files^** - vyasa scans source code, markdown, and other text files. it skips binary files, images, and human-meant data files like xml.

> **^commentaries can and encouraged to exist in source files^** - put mantras in source code comments to trace knowledge to implementation. technical commentaries belong where the code is.

> **^markdown code blocks are skipped^** - content inside triple-backtick code blocks is ignored. this lets you include syntax examples without parsing them.

> **^mantra text is trimmed^** - leading and trailing whitespace inside `**^...^**` is removed.

> **^empty mantras are ignored^** - if there's nothing between `**^^**` or only whitespace, no mantra is created.

> **^kosha anusrits use @suffix^** - to use a mantra from another knowledge base via anusrit, use `_| mantra text |_`@kosha-name``. the kosha must be defined in .vyasa/kosha.md.
