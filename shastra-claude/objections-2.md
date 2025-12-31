## OBJECTIONS STATUS

This is the second set of objections after https://github.com/amitu/vyasa/commit/f0db97f24338243ab01c083476299d0d681cd208. 

# ADDRESSED (can drop):

| #   | Objection              | How Addressed                                                       |
  |-----|------------------------|---------------------------------------------------------------------|
| 12  | Can shastra be wrong?  | objections.md: "shastra is a person's view", relativism section     |
| 15  | Shastras conflict?     | tyakta (abandon own), khandita (reject others), observer resolution |
| 16  | Versioning?            | tyakta provides deprecation mechanism                               |
| 18  | Shastra relationships? | khandita, observed shastras, astika/nastika                         |

PARTIALLY ADDRESSED:

| #   | Objection                                | Status                                                                                              |
  |-----|------------------------------------------|-----------------------------------------------------------------------------------------------------|
| 1   | "Personal" vs "repository" contradiction | objections.md clarifies intent, but shastra.md bhasya still says "repository" without qualification |
| 13  | Shastra vs blog?                         | Mantra/bhasya discipline distinguishes it, but not stated explicitly                                |
| 17  | Quality control?                         | astika/nastika filters objectors, but doesn't address quality of content itself                     |

STILL UNADDRESSED:

| #   | Objection                                         |
  |-----|---------------------------------------------------|
| 2   | "Authoritative" according to whom?                |
| 3   | Tutorials vs bhasya relationship unclear          |
| 4   | What does "accepted" mean?                        |
| 5   | Guru-shishya framing culturally limiting          |
| 6   | "Not a way to make money" - arbitrary restriction |
| 7   | Understand person vs understand knowledge         |
| 8   | Admitting rambling undermines credibility         |
| 9   | No criterion for what's "worth" preserving        |
| 10  | Mountain of text not actually reduced             |
| 11  | Why quote blocks specifically?                    |

  ---
NEW ISSUES IN YOUR UPDATES

objections.md:

1. Line 12: Nested quote > > - What does this syntax mean? Is this a bhasya within a bhasya? Quoting someone else? Undefined.
2. Line 24-25: Inconsistent spacing - **^ mantra^** has space before mantra but not after. Should be **^ mantra ^**.
3. Line 87: Incomplete - "When one prepares a astika" cuts off mid-thought.
4. Dismissive framing risk - "shastra does not concern itself with people who don't agree" could read as "we don't engage with criticism." That's intellectually weak. Real strength is engaging with the best opposing arguments, not ignoring them.
5. Line 20: "Do NOT bother reading their words" - Hostile. Alienates curious skeptics who might convert. Also contradicts the learning mindset.
6. Astika/nastika as binary - What about people who agree with 70%? Are they astika or nastika? The binary is too crude.

mantra.md:

7. No bhasya yet - You explain mantra in prose but haven't committed to a mula mantra definition.
8. Gödel reference unexplained - You invoke incompleteness theorems but don't explain how they relate to mantra vagueness. This looks like name-dropping.
9. "Try to bring some sort of precision" - This is the opposite of precision. What KIND of precision? Logical? Semantic? Referential?

shastra.md bhasya (lines 41-43):

10. Still says "repository" - Generic institutional word. Doesn't capture the personal nature you emphasize in prose.
11. "Authoritative definitions" - Still unexplained. Authoritative to whom? By what standard?

  ---
STRUCTURAL CONCERN

You now have content in three places:
- shastra/shastra.md - accepted
- shastra-amitu/mantra.md - draft
- shastra-amitu/mantra-bhashya.md - draft
- shastra-amitu/objections.md - draft

The objections.md introduces mantras (shastra is a person's view..., shastra can and are meant re-define words...) that should probably be in the main shastra, not in a meta-document about objections.

Question: Is objections.md meant to become part of the shastra, or is it working notes?

  ---
TOP 5 REMAINING CHALLENGES (prioritized):

1. Define "authoritative" - This word appears in your core bhasya but has no grounding. Either define it or remove it.
2. Mantra needs a bhasya - You have prose explaining mantra but no committed definition. The verbatim principle demands you commit.
3. Astika/nastika is too binary - Add nuance or you'll be accused of cultish thinking.
4. "Repository" vs "personal" - Resolve this tension in the bhasya itself, not just in supporting prose.
5. What makes something worth preserving? - Without a principle, the system is arbitrary. "Whatever I feel like" isn't compelling.
