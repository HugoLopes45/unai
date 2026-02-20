# Docstring anti-patterns

LLMs treat docstrings as mandatory forms to fill out. Humans treat them as communication to the next reader. The difference shows up immediately: LLM docstrings say what the code says; human docstrings say what the code cannot.

---

## 1. Tautological summary line

**LLM**
```python
def calculate_tax(amount):
    """Calculate the tax for the given amount."""
```

**Human**
```python
def calculate_tax(amount):
    """Returns tax at the current rate. Raises ValueError if amount < 0."""
```

Rule: if reading the function name tells you what the docstring says, delete the docstring or replace it with something the name cannot express. The summary line should add information, not echo the signature.

---

## 2. Documenting what the type already says

**LLM**
```python
def create_user(user_id: int, name: str, active: bool) -> User:
    """
    Args:
        user_id (int): The ID of the user.
        name (str): The name of the user.
        active (bool): Whether the user is active.
    """
```

**Human**
```python
def create_user(user_id: int, name: str, active: bool) -> User:
    """
    Args:
        user_id: must be positive; use 0 for anonymous sessions
        name: must be non-empty; stripped of leading/trailing whitespace
    """
```

Rule: document constraints, not types. If you have type annotations, the type is already documented. Document what the type cannot express: ranges, invariants, side effects, special sentinel values.

---

## 3. "This function/method/class..." opener

**LLM**
```python
"""This function serves as the primary handler for incoming HTTP requests."""
"""This class is responsible for managing the lifecycle of database connections."""
"""This method provides functionality to parse the configuration file."""
```

**Human**
```python
"""Dispatches incoming HTTP requests to the appropriate route handler."""
"""Manages database connection lifecycle including pooling and retry."""
"""Parses the config file. Raises FileNotFoundError if path does not exist."""
```

Rule: start with a verb. The reader already knows they are reading a docstring for a function. "This function" is three wasted words.

---

## 4. JSDoc @param that restates the type

**LLM**
```typescript
/**
 * Creates a user profile.
 * @param {string} name - The name string parameter
 * @param {number} age - The age number parameter
 * @returns {User} - Returns a User object
 */
function createUserProfile(name: string, age: number): User
```

**Human**
```typescript
/**
 * Creates a user profile. Throws if name is empty or age is negative.
 */
function createUserProfile(name: string, age: number): User
```

Rule: in TypeScript, the types are in the signature. @param tags that just repeat `{string} name - The name` are noise. Document the behavior: what throws, what returns on edge input, what mutates.

---

## 5. Rustdoc that explains Rust to the reader

**LLM**
```rust
/// This function returns an `Option<T>` which represents either a `Some` value
/// containing the result, or `None` if the value is not present in the collection.
pub fn find_user(id: UserId) -> Option<User>
```

**Human**
```rust
/// Returns `None` if no user with this ID has been inserted.
pub fn find_user(id: UserId) -> Option<User>
```

Rule: rustdoc readers know what `Option` means. Document the domain condition that produces `None` or `Err`, not the mechanics of the type.

---

## 6. "Please note" and "It is important to"

**LLM**
```python
"""
Parse the date string into a datetime object.

Please note that this function is not thread-safe.
It is important to ensure the input string is in ISO 8601 format.
"""
```

**Human**
```python
"""
Parse an ISO 8601 date string into a datetime object.

Not thread-safe. Raises ValueError on malformed input.
"""
```

Rule: "please note" and "it is important to" are filler. State the constraint directly. If something is important, lead with it.

---

## 7. Trivial examples in the Examples section

**LLM**
```python
def add(a: int, b: int) -> int:
    """
    Add two numbers.

    Examples:
        >>> add(1, 2)
        3
        >>> add(0, 0)
        0
    """
```

**Human**
```python
def parse_date(value: str, tz: str = "UTC") -> datetime:
    """
    Parse a date string, returning a timezone-aware datetime.

    Examples:
        >>> parse_date("2024-01-01")          # naive string -> UTC-aware
        datetime(2024, 1, 1, tzinfo=UTC)
        >>> parse_date("2024-01-01", tz="US/Eastern")
        datetime(2024, 1, 1, tzinfo=US/Eastern)
    """
```

Rule: examples earn their space by demonstrating non-obvious behavior. `add(1, 2) == 3` teaches nothing. Show the edge case, the gotcha, or the thing a reader would get wrong on first use.

---

## 8. The "comprehensive" docstring that misses the important edge case

**LLM**
```python
def transfer_funds(from_account, to_account, amount):
    """
    Transfer funds between two accounts.

    This function transfers the specified amount from the source account
    to the destination account. It validates the input parameters,
    checks account existence, verifies sufficient funds, performs the
    transfer atomically, and logs the transaction.

    Args:
        from_account (Account): The source account object.
        to_account (Account): The destination account object.
        amount (Decimal): The amount to transfer.

    Returns:
        Transaction: The completed transaction record.
    """
```

**Human**
```python
def transfer_funds(from_account, to_account, amount):
    """
    Transfer funds atomically. Raises InsufficientFunds if balance is low.

    Does NOT send notifications — call notify_transfer() separately.
    amount must be positive and rounded to 2 decimal places.
    """
```

Rule: long docstrings that describe the happy path in detail while burying or omitting the constraints are worse than short ones. Readers skim. Put the surprises first.

---

Good docstrings are asymmetric: they document what is surprising, constrained, or dangerous, and skip what is obvious. If you have to choose between documenting a parameter and documenting a side effect, document the side effect. The reader can see the parameter; they cannot see what happens three layers down.
