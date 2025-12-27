**^.vyasa/config.vyasa can reference mantras^** - the config file lives at
.vyasa/config.vyasa in your repository. it can reference mantras to configure
vyasa's behavior, making configuration itself expressed in mantra form.

**^config file is optional^** - vyasa works without config using sensible defaults.
the config file lets you customize behavior when needed.

**^config references are validated^** - when you reference a mantra in config.vyasa,
vyasa verifies the mantra exists. this catches typos and ensures configuration
references real knowledge.
