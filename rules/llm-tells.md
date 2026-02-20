# LLM tells

Patterns that almost never appear in human-written code. Each one alone is suggestive. Several together is diagnostic.

---

## 1. Commenting out alternatives "in case you need them"

LLMs are uncertain which approach the user wants, so they commit all candidates.

LLM:
```python
def calculate_discount(price, rate):
    # Option 1: Percentage-based
    return price * (1 - rate)
    # Option 2: Fixed amount
    # return price - rate
    # Option 3: Tiered
    # if price > 100:
    #     return price * 0.8
```

Human: picks one and deletes the rest.

Tell: dead-code comment blocks with numbered options mean the author was hedging.

---

## 2. The "you can also..." comment

LLMs replicate tutorial behavior, offering unrequested alternatives inline.

LLM:
```python
def connect_to_db(url):
    # You can also use connection pooling by replacing Pool() with SingletonPool()
    # Alternatively, pass echo=True to enable SQL logging
    return create_engine(url)
```

Human: comments explain why, not what else you could do.

Tell: inline comments offering unrequested alternatives are a tutorial artifact.

---

## 3. Placeholder examples that are too perfect

LLMs draw from training data, where canonical test values appear millions of times.

LLM:
```python
# Example usage:
# user = create_user("john.doe@example.com", "SecurePassword123!")
# payment = process_payment(order, card_number="4111111111111111")
```

Human: uses realistic, messy, project-specific examples ("jsmith", a real test account, an actual card from the staging environment).

Tell: `SecurePassword123!` or `4111111111111111` (the Stripe test card) in example comments.

---

## 4. Perfectly symmetrical start/stop functions

LLMs optimize for visual elegance. Real teardown handles partial failures and has a different shape than startup.

LLM:
```python
def start():
    initialize_database()
    start_cache()
    start_server()
    register_shutdown_hook()

def stop():
    deregister_shutdown_hook()
    stop_server()
    stop_cache()
    cleanup_database()
```

Human: teardown is messier, has try/except blocks, skips steps that may not have started, and rarely mirrors startup exactly.

Tell: a `stop()` that is a perfect reverse-image of `start()` was generated.

---

## 5. `pass` with a TODO comment in production code

LLMs scaffold method signatures to complete a class structure and defer the body.

LLM:
```python
def on_user_created(user: User) -> None:
    # TODO: Implement this method
    pass

def on_payment_failed(payment: Payment) -> None:
    # To be implemented
    pass
```

Human: uses `raise NotImplementedError()` or implements it. `pass` is for empty blocks that are intentionally empty.

Tell: `pass` paired with a comment about future implementation is always AI-generated.

---

## 6. `__all__` with everything listed

LLMs treat `__all__` as documentation rather than as an API surface control mechanism.

LLM:
```python
__all__ = [
    'UserService', 'UserRepository', 'UserNotFoundError',
    'UserAlreadyExistsError', 'UserValidationError',
    'CreateUserRequest', 'UpdateUserRequest', 'UserResponse',
]
```

Human: sets `__all__` to explicitly hide internals from `import *`, not to enumerate everything.

Tell: `__all__` that includes every class in the file is a coverage-maximizing reflex.

---

## 7. SQL column comments that mirror column names

LLMs generate comments to look thorough. They have no knowledge of the actual business context.

LLM:
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,                    -- Unique identifier for the user
    email VARCHAR(255) NOT NULL UNIQUE,     -- User's email address
    created_at TIMESTAMP NOT NULL,          -- Timestamp when the user was created
);
```

Human: comments in SQL explain constraints, business rules, or migration history. "-- was added in migration 47 to support SSO" not "-- the user's email".

Tell: SQL comments that restate the column name in prose are noise generated for appearance.

---

## 8. `#[derive]` everything in Rust

LLMs apply the maximal derive set to avoid compile errors they cannot see coming.

LLM:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default, Builder)]
struct User { ... }
```

Human: derives only what the code actually needs. `Hash` appears when the struct is used as a map key. `Builder` appears when construction is genuinely complex.

Tell: a derive list longer than four items that was not driven by a compiler error.

---

## 9. Step-numbered comments inside functions

LLMs narrate their own output, describing procedure steps as if writing a tutorial.

LLM:
```python
def process_order(order):
    # Step 1: Validate the order
    validate_order(order)
    # Step 2: Calculate the total
    total = calculate_total(order)
    # Step 3: Apply discounts
    total = apply_discounts(total, order)
```

Human: trusts function names. Steps belong in a docstring or design doc, not between every call.

Tell: `# Step N:` comments inside a function body.

---

## 10. The no-personality professional tone throughout

LLMs have no frustration, no humor, and no opinions. Real codebases accumulate human voice.

LLM: every comment is neutral, complete, and formal.

Human:
```python
# this is a disaster but it's the vendor's fault
# don't touch this
# lol
```

Tell: a codebase where every comment reads like a specification has no human author.

---

## 11. No external references

LLMs cannot link to bugs, tickets, or people because they have no access to the organizational context that produced the code.

LLM: no links, no names, no ticket numbers anywhere.

Human:
```python
# Chrome bug: https://bugs.chromium.org/p/chromium/issues/detail?id=123456
# See JIRA-4821
# Hat tip: https://stackoverflow.com/a/12345678
# @alice suggested this at the retro
```

Tell: zero external references across an entire file, especially in workaround code.

---

## 12. TODOs without owners or dates

LLMs generate TODOs as structural placeholders, not as actionable items with accountability.

LLM:
```python
# TODO: add error handling
# TODO: consider refactoring this
```

Human:
```python
# TODO(hugo, 2026-02-20): remove after migration completes
# HACK: temporary until Redis 8 ships -- see issue #892
```

Tell: a TODO with no owner, no date, and no ticket is a placeholder that was never meant to be acted on.

---

## 13. `result` as a catch-all variable name, used repeatedly

LLMs reuse generic names because they process one statement at a time without holding the full naming context.

LLM:
```python
result = fetch_user(id)
result = calculate_total(items)
result = validate_input(data)
```

Human: names the thing: `user`, `total`, `ok, err`.

Tell: `result` appearing more than once in a function body for different types of values.

---

## 14. Defensive checks inside typed functions

LLMs layer runtime guards over types that already provide those guarantees, because they hedge against failure modes they cannot see.

LLM:
```typescript
function processUser(user: User): void {
  if (!user) throw new Error('User is required');
  if (!user.id) throw new Error('User ID is required');
  // TypeScript already guarantees user: User here
}
```

Human: validates at system boundaries (parse, deserialize, API entry) and trusts the type system inside the domain.

Tell: null-checks on typed parameters in a typed language, especially with generic error messages.

---

## 15. Never admitting known bugs or approximations

LLMs cannot observe the running system, so they have no bugs to document.

LLM: no notes on known inaccuracies, edge-case failures, or performance caveats.

Human:
```python
# NOTE: overcounts by ~2% due to eventual consistency lag -- JIRA-3891
# WARN: this breaks if the locale is not UTF-8, see issue #204
```

Tell: complex numeric or I/O logic with no caveats, approximations, or known-issue notes.

---

## 16. Language-specific tells

### Python

- `from typing import List, Dict, Optional` in Python 3.10+ code. Use `list`, `dict`, `X | None`.
- Manual `__init__` with only attribute assignments instead of `@dataclass`.
- `for item in list_of_items:` instead of `for item in items:`. LLMs over-qualify the collection name.

Tell: `from typing import List` in a file that uses `match` statements or walrus operators.

### TypeScript

- `interface` for everything including simple data shapes (use `type` for plain objects).
- `as X` casting instead of proper type narrowing.
- Explicit `Promise<void>` on every async function return, even when inferred.

Tell: `interface` declarations for objects that are never extended or implemented.

### Rust

- `.clone()` where a borrow would work. LLMs resolve lifetime friction with clone.
- `.unwrap()` with a comment that says "should never fail". If it should never fail, use `expect` with a message.
- Wrapping safe code in `unsafe` blocks with a comment about performance.

Tell: `.clone()` in a hot path with no performance comment, or `unsafe` with a vague justification.

### Go

- `fmt.Errorf("error in stepN: %w", err)` at every internal call site instead of wrapping at boundaries.
- Struct tag comments that restate the field name (`// UserID is the user ID`).

Tell: error wrapping at every layer, creating stack traces in error strings.

---

## The root cause

LLMs optimize for the code review, not for the running system. They generate code that demonstrates correctness to a reader, not code written by someone who has to maintain it. An LLM has never been paged at 3am, never hit a race condition in production, never argued with a vendor about a bug. That experience is what produces the personality, the scars, and the specificity that distinguishes human code. LLMs produce the median of what correct code looks like in training data. Humans produce the residue of what actually happened.
