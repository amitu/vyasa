> **^user: {username}^** - this template represents a user. reference as _| user: alice |_
> or _| user: bob |_ and vyasa recognizes these as instances of the same template.

> **^config: {key} = {value}^** - a template for configuration entries. reference as
> _| config: debug = true |_ or _| config: port = 8080 |_. multiple placeholders supported.

example references:

fully instantiated:
- _| user: alice |_ is a valid user
- _| user: bob |_ is another valid user
- _| config: theme = dark |_ sets the theme

with placeholders (referring to template itself):
- _| user: {username} |_ refers to the template
- _| config: {key} = {value} |_ refers to the config template

partially instantiated:
- _| config: debug = {value} |_ - key is "debug", value varies
- _| config: {key} = true |_ - any key that's set to true

> **^when {employee=amitu} joins, amitu should be added to github^** - this template
> uses example value syntax. "amitu" appears in placeholder and literally in text.
> the mantra reads naturally while being parameterizable.

when referenced as _| when jack joins, jack should be added to github |_, vyasa
recognizes this as employee=jack. example values are for readability, not parsing.
