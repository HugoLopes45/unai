# Commit message anti-patterns

LLMs write commit messages the way a manager summarizes a sprint: vague, passive, and verbose. Real commit messages are imperative, specific, and lowercase. These patterns show up constantly.

---

## 1. Vague improvement verbs

**LLM**
```
Refactor the codebase to enhance maintainability and improve code quality
```

**Human**
```
extract auth into middleware
```

Rule: name the thing you changed and what you did to it. "Improve", "enhance", "update", and "refactor" without a subject are content-free.

---

## 2. "Various" and "several"

**LLM**
```
Fix various bugs and address several issues across the application
```

**Human**
```
fix null deref in user loader
fix off-by-one in paginator
```

Rule: one commit per fix. "Various" is a signal you bundled changes that should be separate. Reviewers can't revert "various".

---

## 3. Passive voice

**LLM**
```
Code has been updated to reflect new requirements
The configuration was modified to support the new environment
```

**Human**
```
update schema for soft deletes
use env vars for db config
```

Rule: commits are imperative by convention. The subject is always "this commit". "Add", not "Added" or "Was added".

---

## 4. Capitalized subjects

**LLM**
```
Update The Configuration File To Use Environment Variables
Refactor User Authentication Module For Better Performance
```

**Human**
```
use env vars for db config
speed up token verification
```

Rule: lowercase everything except proper nouns and acronyms. Title case reads like a PowerPoint slide.

---

## 5. Past tense

**LLM**
```
Added user authentication
Fixed the null pointer bug
Updated the config file
```

**Human**
```
add user auth
fix null deref in session init
update config loader
```

Rule: git convention is imperative mood. Think "if applied, this commit will ___." Not "I did ___."

---

## 6. "In order to"

**LLM**
```
Refactor UserService in order to improve testability
Extract helper functions in order to reduce code duplication
```

**Human**
```
make UserService testable
deduplicate date formatting helpers
```

Rule: never write "in order to". It always collapses to "to", and "to" usually collapses to nothing.

---

## 7. Body that describes the diff, not the reason

**LLM**
```
feat: update user service

- Added email validation
- Updated the save method
- Changed the return type to Optional
- Modified error handling
```

**Human**
```
validate email before save

Previously we deferred validation to the DB layer, which made
errors hard to attribute. Validate at the service boundary so
callers get a clear ValueError with the offending value.
```

Rule: the diff already shows what changed. The body is for why. If the why is obvious, skip the body entirely.

---

## 8. Emoji prefixes used wrong

**LLM**
```
🐛 Refactor database connection pooling
✨ Fix typo in error message
🔧 Add new user registration feature
```

**Human**
```
fix: db connection pool exhaustion under load
feat: add user registration
```

Rule: if you use emoji or conventional commit types, use them correctly. A refactor is not a bug fix. A typo fix is not a feature. When in doubt, skip the emoji.

---

## 9. "WIP" commits that are complete

**LLM**
```
WIP: add payment processing
WIP: implement auth (complete)
WIP: fix bug (done)
```

**Human**
```
add Stripe checkout session creation
```

Rule: WIP means the work is not done. If it is done, name what it is. WIP commits should be squashed before merge.

---

## 10. "Initial implementation" on non-initial commits

**LLM**
```
initial implementation of user service
initial commit for payment module
add initial version of auth system
```

**Human**
```
add user service skeleton
scaffold payment module
add JWT-based auth
```

Rule: "initial" is redundant. Every commit adds something for the first time. Name what it is. Reserve "initial commit" for the literal first commit in a repo, and even then consider something more descriptive.

---

Human commit messages read like instructions to a future reader who has the diff open. They answer "what should I expect this change to do" in under 72 characters, and only explain more in the body when the why is non-obvious. The subject line is a command, not a caption.
