# Writing rules

Apply these rules to agent instructions, explanations, comments, documentation, and PR text.
Use the user's language for replies.
Keep each existing document's language unless the task requires a translation.

## English text

Use this strict project profile based on ASD-STE100 Issue 9.

1. Give one action in each instruction sentence.
2. Start instructions with an imperative verb.
3. If a condition controls an action, put the condition first.
4. Limit instruction sentences to 20 words.
5. Limit descriptive sentences to 25 words.
6. Use active voice in instructions.
7. Use active voice in descriptions when the actor is known.
8. Give each paragraph one topic, with no more than six sentences.
9. Use one term for each concept.
10. Keep each term's meaning and part of speech consistent.
11. Use complete sentences with the necessary articles.
12. Separate explanatory notes from instructions.

Use the approved dictionary meaning for general words when you can verify it.
Use project technical terms for concepts that general words cannot describe accurately.
If a dictionary entry is uncertain, record that uncertainty during a compliance review.
Do not claim full ASD-STE100 compliance without a review against the official rules and dictionary.

## Project terminology

Keep API names, code identifiers, paths, flags, commands, and quoted errors exact.
Define unfamiliar abbreviations at first use in each standalone document.
Use the terms from the relevant contract or source file.

| Term | Meaning in this repository |
| --- | --- |
| Room | A collaboration space with members and messages. |
| Tenant | An isolation scope; it is not another name for a room. |
| Agent run | One invocation of the room assistant. |
| Tool call | One tool operation within an agent run. |
| Acceptance evidence | Recorded results for a named check and revision. |

Do not change technical meaning to meet a sentence limit.
Split a long explanation into separate sentences.
For Chinese text, use short sentences, one action per instruction, and consistent terms.
The English word limits do not define Chinese sentence length.

## Explain results

- Give the result first.
- State the cause when the evidence supports it.
- Separate observed behavior, proposed behavior, and unknown results.
- For a failed check, name the command and the failure.
- State the effect of any missing verification.
- Remove filler, metaphors, vague praise, and repeated summaries.
- Use a table for comparisons or a diagram for relationships when it improves understanding.
- Use interactive output only when interaction helps the reader make a decision.

Example:

> The gateway rejects a blank production signing key. The startup test passes. Restart recovery remains unverified.

## Review instructions

1. Check that each action has a clear object.
2. Check sentence lengths and terminology.
3. Check conditions, exceptions, and technical meaning against the source.
4. Check that each result has supporting evidence.
5. Remove duplicate rules from the root file.

For a local length check, count whitespace-separated words after you remove Markdown markup.
Count each path or code identifier as one token.
Exclude command blocks from prose limits.
This local check cannot validate the complete STE dictionary or official word-count rules.

## Sources

- [ASD-STE100 Issue 9](https://www.asd-ste100.org/assets/files/ASD-STE100_ISSUE9.pdf): official writing rules and dictionary.
- [ASD overview](https://www.asd-ste100.org/about_STE.html): controlled vocabulary and project technical terms.
- [ASD FAQ](https://www.asd-ste100.org/STE_faq.html): scope and use outside maintenance documentation.
- [Karpathy's post](https://x.com/karpathy/status/2105819303471976479): motivation for clear explanations and useful output formats.

The original post was inaccessible during preparation.
Its text was available through a [public mirror](https://x.twstalker.com/karpathy/status/2105819303471976479).
The project profile is an adaptation, not a replacement for the official standard.
