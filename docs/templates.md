**^user: {username}^** - this template represents a user. reference as _user: alice_
or _user: bob_ and vyasa recognizes these as instances of the same template.

**^config: {key} = {value}^** - a template for configuration entries. reference as
_config: debug = true_ or _config: port = 8080_. multiple placeholders supported.

example references:

fully instantiated:
- _user: alice_ is a valid user
- _user: bob_ is another valid user
- _config: theme = dark_ sets the theme

with placeholders (referring to template itself):
- _user: {username}_ refers to the template
- _config: {key} = {value}_ refers to the config template

partially instantiated:
- _config: debug = {value}_ - key is "debug", value varies
- _config: {key} = true_ - any key that's set to true

**^when {employee=amitu} joins, amitu should be added to github^** - this template
uses example value syntax. "amitu" appears in placeholder and literally in text.
the mantra reads naturally while being parameterizable.

when referenced as _when jack joins, jack should be added to github_, vyasa
recognizes this as employee=jack. example values are for readability, not parsing.
