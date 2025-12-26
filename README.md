# vyasa

a tool to help organize / curate knowledge for self / public / AI.

they key idea of this tool is easily written prose tends to come in the way
of organising knowledge and not assist it, that knowledge requires careful
wording, trying to keep things compact, preserving the exact formulations for
canonical knowledge, and commentaries can be added for general explanations 
etc.

the tool defines a concept of a mantra, it can be in any language, but it has
to be strict, repeating the mantra must repeat the exact phrasing, and 
canonical / normative form must be preserved etc.

this forces all "knowledge" to be expressed in minimal way, and the tool
"resists" drift, as any tiniest change in wording is now a new mantra, and
new mantra, if they are to be accepted should go back update all previous
invocations of the new mantra, or the two exists in the system, pointing to
a ambiguity.

## a repo or kosha

vyasa operates on a repository, it can be a single folder, a git repository, or
the fastn-kosha (available over p2p etc being experimented in fastn-stac/spatial).

the knowledge in the repo are analysed by vyasa tool to extract the mantras out
of them, so the text files in the repo follow a mantra syntax. the mantra
syntax is not yet fixed, it will be experimented in this repo.

lets start with one draft proposal:

```vyasa
--
e = mc^2 
--

this is a vyasa file which contains a mantra, delimited by the -- above. 

-- dash-dash-mantra
each mantra starts and ends with lines containing --, and can have optional
name along with body.
--

here we have created a mantra about vyasa, [dash-dash-mantra], and we have 
a mantra [e = mc^2]. 

-- mantra-reference
every time we want to refer to a mantra, we can prefix it with 
(optional-repo:)[identifier]..
--

we have defined a third mantra:mantra-reference, as you can see there are few
ways to refer to a mantra, by their ids, ids are optional, as [e = mc^2]
is probably better than [whatever-you-will-want-to-name-this-formula]

-- mentra-redifinition
a mantra once defined must not be changed, unless we want to redifine it, in 
which case, we must use the refinitions syntax.
--

-- mantra-redicinition-proposal-example
\-- some-mantra
the original exact wording of this mantra
--- redinition-identifer
the new wording of this mantra
--

this is a call to redifine any mantra, here we have proposed a new definition to
the [mantra-redicinition-proposal-example], and this redifinition should
be referrered as [mantra-redicinition-proposal-example][redinition-identifer].

vyasa tool is aware of all the original definitions, it creates the hashes of
each mantra in .vyasa folder, and does basic sanity testing like if the original
mantra was not exactly spelled out.

--
mantras wont have ids
--

after some thought i am removing the mantra id, and forcing use of fully spelling
out each mantra at each reference site.

--
no redifinition identifier needed
--

with that simplification we can say there is no need for
[mantra-redicinition-proposal-example][redinition-identifer] syntax.

vyasa will verify that all [mentra] have been explained at least once, meaning

--
mantra explanations can repeat
--

[mantra explanation] is a -- block for the mantra, and it can happen any number
of the times, so it is illegal to use a mantra that has never been explained,
but a mantra can be explained multiple times. the explanations are non-normative,
they must not encode knowledge, just explanation of knowledge and humans / ai
must only refer to original unique mantras for knowedge.

-- 
mantra deprecation
--

once a new wording of a mantra has been identified / accepted, all past references
of the mantra should be updated to new definition, or if there are commentaries
about old definitions worth preserving, either old mantra mentions in those
commentaries should be given a new name for old mantra, say you prefer the
wording of the original mantra, but redifine its meaning, you first give a new
wording to old meaning of mantra, update all references to it, and then add the
new mantra etc.

this is intentionally a very deliberate / compicated sounding process, and thats
kind of the key idea of the vyasa tool, to assist (a little bit, it can not
really do much, will print some stats about how many mantras are there, how
have they have been added over time etc, as vyasa will be git and kosha aware
so it can see the history of the repo, and trace the mantra additions / renames
etc in a limited way).

--
vyasa isnt really needed
--

ideally you should be using this tool without having the tool at all, this tool
is a sort of reminder of a mental descipline more than a tool doing something
for you.
```
