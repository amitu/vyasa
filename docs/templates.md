^user: {username}^ - this template represents a user. reference as ~user: alice~
or ~user: bob~ and vyasa recognizes these as instances of the same template.

^config: {key} = {value}^ - a template for configuration entries. reference as
~config: debug = true~ or ~config: port = 8080~. multiple placeholders supported.

example references:

fully instantiated:
- ~user: alice~ is a valid user
- ~user: bob~ is another valid user
- ~config: theme = dark~ sets the theme

with placeholders (referring to template itself):
- ~user: {username}~ refers to the template
- ~config: {key} = {value}~ refers to the config template

partially instantiated:
- ~config: debug = {value}~ - key is "debug", value varies
- ~config: {key} = true~ - any key that's set to true

^when {employee=amitu} joins, amitu should be added to github^ - this template
uses example value syntax. "amitu" appears in placeholder and literally in text.
the mantra reads naturally while being parameterizable.

when referenced as ~when jack joins, jack should be added to github~, vyasa
recognizes this as employee=jack. example values are for readability, not parsing.
