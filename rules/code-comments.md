# Code comments

LLMs write comments the way a junior dev writes documentation to prove they understand the code. The comments are technically accurate and completely useless. They narrate what is already visible, apologize for imperfect code, and add ceremony that makes a file feel finished while telling the reader nothing.

This document catalogs every pattern. Each one has a cause, a before/after, and a one-line fix rule.

---

## 1. Tautological comments

The comment restates exactly what the code says, at the same level of abstraction. LLMs do this because training data contains tutorial code where every line is explained for beginners. The model cannot distinguish "explaining for a student" from "documenting for a colleague."

LLM:
```python
# Initialize the counter variable to zero
counter = 0

# Increment the counter by one
counter += 1
```

Human:
```python
counter = 0
counter += 1
```

Fix: if you can read the comment and then look at the code and think "yes, I see", delete the comment.

---

## 2. Captain Obvious docstrings

The docstring restates the function signature in prose. LLMs are trained on codebases with style guides mandating docstrings on every function, so they fill them in mechanically, repeating the type annotations and parameter names already visible in the signature.

LLM:
```python
def get_user(user_id: int) -> User:
    """
    Retrieves a user by their user ID.

    Args:
        user_id: The ID of the user to retrieve.

    Returns:
        The user object corresponding to the provided user ID.
    """
```

Human:
```python
def get_user(user_id: int) -> User:
    """Fetch user by ID. Raises UserNotFound if missing."""
```

Fix: a docstring earns its place only if it adds information not in the signature -- exceptions raised, side effects, non-obvious behavior, or a usage example.

---

## 3. Section header comments

Decorative banners dividing a file or function into labeled regions. LLMs produce these because training data includes legacy enterprise codebases written before IDEs had symbol navigation. The pattern signals file structure in a way that was useful in 1998.

LLM:
```python
# ============================================================
# SECTION 1: INITIALIZATION
# ============================================================

# --- Helper Functions ---

# ~~~ Database Layer ~~~
```

Human:
```python
# (just write the code)
```

Fix: if a function needs section headers to be navigable, it needs to be split into smaller functions.

---

## 4. Narrating intent obvious from the code

The comment explains why a pattern was chosen when the pattern is self-evident. LLMs annotate stylistic choices because tutorial content typically explains not just what code does but why a particular construct was picked -- habits that leak into production comments.

LLM:
```python
# Use a list comprehension for efficiency
result = [x * 2 for x in items]

# Use a dictionary for fast lookups
index = {item.id: item for item in items}
```

Human:
```python
result = [x * 2 for x in items]
index = {item.id: item for item in items}
```

Fix: only explain a construct choice when a reader might reasonably ask "why not a simpler alternative" -- and have a real answer.

---

## 5. Hedge comments

Disclaimers, apologies, and vague future intentions. LLMs generate these as a kind of epistemic hedging -- the model is uncertain, so it signals uncertainty in the comment. In code, this transfers as indefinite TODOs and non-committal warnings that no human will ever act on.

LLM:
```python
# Note: This approach may not be optimal for all use cases.
# TODO: Consider refactoring this in the future for better performance.
# This is a simplified implementation and may need enhancement.
```

Human:
```python
# (if it needs fixing, fix it or open a ticket with specifics)
```

Fix: every hedge comment should either become a concrete TODO with a ticket reference or be deleted.

---

## 6. Explaining standard language features

The comment describes what a well-known language construct does, as if the reader has never seen it. This comes from the same tutorial-data bias as tautological comments, compounded by LLMs optimizing to be helpful to the least-experienced possible reader.

LLM:
```python
# Use try-except to handle potential errors gracefully
try:
    result = fetch(url)
except RequestError as e:
    # Log the error for debugging purposes
    logger.error(e)
    # Return None to indicate failure
    return None
```

Human:
```python
try:
    result = fetch(url)
except RequestError as e:
    logger.error(e)
    return None
```

Fix: assume the reader knows the language; never explain syntax or standard library behavior.

---

## 7. Inline comments that restate the variable name

An inline comment that expands the identifier into a sentence. LLMs produce these because they have been reinforced on code where inline comments are valued as a sign of thoroughness -- so they add them to every declaration, regardless of whether the name is already clear.

LLM:
```js
const maxRetries = 3;        // Maximum number of retries
const isValid = true;        // Whether the input is valid
const userId = req.user.id;  // The ID of the current user
```

Human:
```js
const maxRetries = 3;
const isValid = true;
const userId = req.user.id;
```

Fix: if the comment is just the variable name with spaces added, delete it.

---

## 8. Philosophical and motivational comments

Prose that describes the importance or spirit of the code rather than anything about how it works. LLMs occasionally absorb this from architecture docs, ADRs, or README text that ended up interleaved with code in their training data.

LLM:
```python
# This function serves as the cornerstone of our data processing pipeline.
# It embodies our commitment to clean, maintainable code and robust design.
def process(records: list[Record]) -> list[Result]:
    ...
```

Human:
```python
def process(records: list[Record]) -> list[Result]:
    ...
```

Fix: if the comment could appear verbatim in a slide deck, remove it from the source file.

---

## 9. "We" language

Using "we" or "our" to describe what the code does. LLMs use collaborative pronouns because they are trained to be helpful and inclusive, and they mimic the editorial "we" found in textbooks and documentation. It sounds performatively collegial and signals that the comment was generated, not written.

LLM:
```python
# We first validate the input before processing.
# Here we apply our transformation logic.
# We return early to avoid unnecessary computation.
```

Human:
```python
# validate before processing
# or: (no comment)
```

Fix: rewrite in imperative or declarative form, or delete; never "we".

---

## 10. Changelog comments

Explaining what the code used to do, or why something was changed. LLMs produce these when asked to modify existing code -- they document the change as if there is no version control. In a repository, this information belongs in the commit message.

LLM:
```python
# Changed from list to dict for O(1) lookups
# Previously used a for loop; now uses map() for clarity
# Removed the caching layer that was causing race conditions
```

Human:
```python
# (that is what git blame and commit messages are for)
```

Fix: any comment that begins with "was", "previously", "changed", or "removed" belongs in the commit message, not the source.

---

## 11. Multi-line comments for one-liners

Expanding a simple remark into a block comment for no reason. LLMs are implicitly rewarded for appearing thorough, and a longer comment can look like a more documented codebase, even when a single short line would do.

LLM:
```python
# This line strips leading and trailing whitespace from
# the user-provided input string to ensure that the value
# is clean before we proceed with further validation.
name = name.strip()
```

Human:
```python
name = name.strip()
```

Fix: if the comment would fit on one short line, write it that way or not at all.

---

## 12. "IMPORTANT" and "NOTE" emphasis markers

Capitalised keywords that assert a comment is important without encoding that importance in the code itself. LLMs use these because documentation style guides often define NOTE and WARNING admonitions. Applied to source comments, they indicate a constraint the model knows exists but did not enforce structurally.

LLM:
```python
# IMPORTANT: This must be called before initialize().
# NOTE: Do not modify this value directly.
# WARNING: This function is not thread-safe.
```

Human:
```python
# call before initialize()
```
Or better:
```python
# enforce the ordering in code -- raise if called out of sequence
```

Fix: if the constraint matters, enforce it in code; if you cannot, write a plain comment without the all-caps theatre.

---

## 13. Vague TODO format

TODOs that describe a direction rather than a task. LLMs write aspirational TODOs because they are optimistic about future improvements but have no skin in the game. The result is a comment that sounds reasonable and will never be acted on.

LLM:
```python
# TODO: Consider implementing caching to enhance performance.
# TODO: Add comprehensive error handling for edge cases.
# TODO: Refactor this method to improve readability and maintainability.
```

Human:
```python
# TODO(hugo): cache this -- hits DB on every request (#1234)
# TODO: handle empty list -- currently returns None, breaks caller in routes/users.py
```

Fix: a valid TODO has an owner or ticket, a specific problem, and ideally the location where the problem surfaces.

---

## 14. Redundant type comments

Annotating types in comments when the language already has type annotations. LLMs trained on pre-annotation Python internalized the old pattern of describing parameter types in prose, then apply it in codebases that already use type hints -- doubling up the information.

LLM:
```python
def process(data: dict) -> list:
    # data: dict - the input data dictionary
    # returns: list - the processed results
    ...

# users is a list of User objects
users: list[User] = repo.all()
```

Human:
```python
def process(data: dict) -> list:
    ...

users: list[User] = repo.all()
```

Fix: if the type is in the annotation, it is not in the comment.

---

## 15. Boilerplate license headers on ephemeral files

Prepending a full Apache 2.0 or MIT block to a ten-line utility script or a generated file. LLMs associate professional code with license headers because the open-source repositories in their training data have them on every file. They cannot judge when a file warrants the ceremony.

LLM:
```python
# Copyright 2024 Acme Corp.
# Licensed under the Apache License, Version 2.0.
# See LICENSE file in the repository root for details.
# SPDX-License-Identifier: Apache-2.0
#
# THIS SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND.
```

Human:
```python
# SPDX-License-Identifier: Apache-2.0
```

Fix: license headers belong in the repo root or a NOTICE file; if per-file headers are required by policy, use a one-line SPDX identifier.

---

## 16. Transitional narration

Comments like "now we can proceed to..." or "at this point we have..." that read as a running commentary on execution flow. LLMs produce these when generating longer functions because they are narrating their own chain-of-thought, which bleeds into the output.

LLM:
```python
# At this point, the user has been authenticated.
# Now we can proceed to load their preferences.
# Finally, we return the fully constructed response.
```

Human:
```python
# (just write the next line of code)
```

Fix: delete any comment that describes what the next line of code is about to do rather than why it does it.

---

## 17. Symmetrical closing comments

Marking the end of a block with a comment repeating what opened it. These appeared in pre-IDE C and shell scripts, where matching braces across hundreds of lines was hard. LLMs reproduce the pattern regardless of language or block length.

LLM:
```python
if user.is_admin:
    ...
# end if user.is_admin

for item in cart:
    ...
# end for
```

Human:
```python
if user.is_admin:
    ...

for item in cart:
    ...
```

Fix: if a closing comment feels necessary, the block is too long and should be extracted into a named function.

---

## 18. Complimentary self-commentary

Describing the code's own quality: "clean", "efficient", "robust", "elegant". LLMs absorb marketing language from documentation and READMEs and occasionally surface it inside source files. It is not a comment. It is an adjective in search of a sentence.

LLM:
```python
# Clean, efficient implementation of the merge algorithm
def merge(a: list, b: list) -> list:
    ...

# Robust error handling ensures reliability
try:
    ...
```

Human:
```python
def merge(a: list, b: list) -> list:
    ...

try:
    ...
```

Fix: delete any comment that asserts the quality of the code rather than explaining it.

---

## What humans actually comment on

After removing everything above, what remains is what comments are for.

Non-obvious business logic. The rule that cannot be inferred from the code: a regulatory constraint, a domain quirk, a product decision someone will definitely question.

```python
# EU accounts use gross pricing; US accounts use net. Do not unify.
price = gross_price if account.region == "EU" else net_price
```

Known workarounds. Code that is ugly for a real reason. The comment names the reason so the next person does not "fix" it and reintroduce the bug.

```python
# Stripe webhooks can arrive out of order. Re-fetching the charge here
# gets canonical state instead of trusting the event payload.
charge = stripe.Charge.retrieve(event.data.object.id)
```

Performance traps. A choice that looks naive but is faster in practice, or a warning that a seemingly simple call is expensive.

```python
# .count() hits the DB. Use len(results) if you already fetched the list.
total = queryset.count()
```

Links to external context. Issue trackers, RFCs, Stack Overflow answers, or vendor documentation that explains a non-obvious decision.

```python
# ECMA-262 sec 20.4.1.11 -- Date UTC conversion behavior changed in ES2020.
# https://github.com/org/repo/issues/881
offset = utc_offset_minutes(tz)
```

Intentional omissions. Code that deliberately does not handle a case, so a future reader does not add handling thinking it was forgotten.

```python
# Not handling SIGTERM -- process manager sends SIGKILL after 10s anyway.
signal.signal(signal.SIGINT, shutdown)
```

Unsafe invariants. Assumptions the code relies on that are not enforced by the type system and that would cause silent corruption if violated.

```python
# Caller must hold the write lock. This function does not acquire it.
def _unsafe_write(self, key: str, value: bytes) -> None:
    ...
```

The test: a good comment tells you something the code cannot. If you deleted the comment and the code still communicated everything the comment said, the comment was noise.