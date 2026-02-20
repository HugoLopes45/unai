# Error handling anti-patterns

LLMs treat error handling as a ritual: wrap everything in try/except, log something reassuring, return None, move on. This produces code that swallows failures silently and makes debugging harder than no error handling at all. Good error handling is specific about what failed, why, and what the caller should do next.

---

## 1. Catch-all exception handlers that swallow errors

LLMs default to `except Exception` because it is safe — it always compiles, it never re-raises, and it looks like robust error handling. It is none of those things. It catches programming errors, `KeyboardInterrupt` relatives, and every failure you did not anticipate, then hides them behind a `return None`.

**LLM**
```python
try:
    result = risky_operation()
except Exception as e:
    print(f"An error occurred: {e}")
    return None
```

**Human**
```python
try:
    result = risky_operation()
except NetworkTimeout as e:
    raise RetryableError("network timeout in risky_operation") from e
except PermissionDenied:
    raise  # caller handles auth errors
```

Rule: catch the specific exception you know how to handle; let everything else propagate — returning `None` on unknown failure lies to the caller.

---

## 2. Generic error messages without context

LLMs write error messages that describe the situation in abstract terms: "Invalid input", "Processing failed", "Something went wrong." These are useless at 2am. The offending value is never in the message.

**LLM**
```python
raise ValueError("Invalid input provided")
raise RuntimeError("An error occurred during processing")
raise Exception("Something went wrong")
```

**Human**
```python
raise ValueError(f"age must be >= 0, got {age!r}")
raise RuntimeError(
    f"ffmpeg exited with code {proc.returncode}: {proc.stderr.decode()[:200]}"
)
raise ParseError(line=12, col=4, msg="unexpected token ']'")
```

Rule: include the offending value, the constraint violated, and the location — the f-string costs nothing; the debugging time costs hours.

---

## 3. Logging and then raising

LLMs log before re-raising because both actions feel responsible. The result is double-logged errors in every monitoring tool, with duplicates that make it impossible to trace where an error originated.

**LLM**
```python
except Exception as e:
    logger.error(f"Error processing request: {e}")
    raise
```

**Human**
```python
except DatabaseConnectionError as e:
    raise ServiceUnavailableError("payment service unreachable") from e
```

Rule: log at the boundary that handles the error, not at every intermediate layer — if you raise, the handler will log; doing both produces noise.

---

## 4. "Error:" prefix in error messages

LLMs prefix exception messages with "Error:" because it feels explicit. The exception class already communicates that this is an error. The prefix is redundant. It also breaks the convention of reading exception messages as plain sentences.

**LLM**
```python
raise ValueError("Error: Invalid email address")
raise RuntimeError("Error: failed to connect to database")
raise KeyError("Error: key not found in config")
```

**Human**
```python
raise ValueError(f"invalid email: {email!r}")
raise RuntimeError(f"database unreachable at {host}:{port}")
raise KeyError(f"required config key missing: {key!r}")
```

Rule: write exception messages as plain sentences describing what went wrong — the exception class supplies the "error" framing.

---

## 5. Exception classes that add no information

LLMs create exception hierarchies with bare subclasses. The class exists for `isinstance` checks but carries no data that would help a handler respond correctly.

**LLM**
```python
class ApplicationError(Exception): pass
class DataProcessingError(Exception): pass
class UserServiceException(Exception): pass
class InvalidOperationError(Exception): pass
```

**Human**
```python
class ParseError(ValueError):
    def __init__(self, line: int, col: int, msg: str) -> None:
        self.line = line
        self.col = col
        super().__init__(f"line {line}, col {col}: {msg}")

class QuotaExceeded(Exception):
    def __init__(self, limit: int, used: int) -> None:
        self.limit = limit
        self.used = used
        super().__init__(f"quota exceeded: {used}/{limit}")
```

Rule: add the fields a handler needs to respond correctly — if the handler must know the line number to display a useful error, the exception must carry it.

---

## 6. Identical try/except wrappers around every async call

LLMs wrap each database or network call in its own try/except block with `console.log` and `return null`. Each block is structurally identical. None of them add context. All of them hide the origin of the failure.

**LLM**
```typescript
async function getUser(id: string) {
    try {
        const user = await db.users.findById(id);
        return user;
    } catch (error) {
        console.error("Failed to get user:", error);
        return null;
    }
}

async function getOrg(id: string) {
    try {
        const org = await db.orgs.findById(id);
        return org;
    } catch (error) {
        console.error("Failed to get org:", error);
        return null;
    }
}
```

**Human**
```typescript
async function getUser(id: string): Promise<User> {
    return db.users.findById(id);  // throws if not found; handled at boundary
}

async function getOrg(id: string): Promise<Org> {
    return db.orgs.findById(id);
}
```

Rule: let errors propagate from individual calls; handle them at the boundary where you have enough context to respond correctly.

---

## 7. Catching the `Exception` base class everywhere

LLMs catch `Exception` (or `Error` in TypeScript) as a default. This treats every possible failure — including bugs, type errors, and assertion failures — as something the current function knows how to handle. It does not.

**LLM**
```python
except Exception:
    return default_value
```

**Human**
```python
except FileNotFoundError:
    return default_value
```

Rule: catch specific exceptions; let unexpected ones propagate — unexpected exceptions are bugs, and bugs should crash loudly.

---

## 8. Finally blocks that duplicate context managers

LLMs write `try/finally` with `resource.close()` when a context manager already handles cleanup. The manual version is harder to get right: it does not suppress exceptions raised during cleanup, and it does not compose correctly.

**LLM**
```python
conn = get_connection()
try:
    result = conn.execute(query)
    return result
except Exception as e:
    logger.error(e)
    raise
finally:
    conn.close()
```

**Human**
```python
with get_connection() as conn:
    return conn.execute(query)
```

Rule: use context managers for resources that support them; reserve `finally` for cleanup that cannot be expressed as a context manager.

---

## 9. try/except wrapping the entire function body

LLMs wrap the entire function in a single try/except to "handle any error." Each step inside can fail for a different reason requiring a different response. Treating them all identically makes failures indistinguishable.

**LLM**
```python
def process_payment(order_id: str, amount: Decimal) -> Receipt:
    try:
        order = load_order(order_id)
        customer = load_customer(order.customer_id)
        validate_payment_method(customer)
        charge = stripe.charge(customer.payment_method, amount)
        receipt = save_receipt(charge)
        send_confirmation_email(customer, receipt)
        return receipt
    except Exception as e:
        logger.error(f"Payment processing failed: {e}")
        return None
```

**Human**
```python
def process_payment(order_id: str, amount: Decimal) -> Receipt:
    order = load_order(order_id)          # raises OrderNotFound
    customer = load_customer(order.customer_id)
    validate_payment_method(customer)     # raises InvalidPaymentMethod
    charge = stripe.charge(              # raises stripe.StripeError
        customer.payment_method, amount
    )
    receipt = save_receipt(charge)
    send_confirmation_email(customer, receipt)
    return receipt
```

Rule: handle exceptions where you can do something useful with them — at the step that failed, not at the outer scope of every function.

---

## 10. Returning error codes and throwing exceptions

LLMs mix error signaling strategies within the same codebase or even the same module: some functions return `(result, error)` tuples, others raise, others return `None`. Callers must handle both mechanisms everywhere.

**LLM**
```python
def find_user(user_id: int) -> tuple[User | None, str | None]:
    try:
        return db.query(user_id), None
    except NotFoundError:
        return None, "user not found"
    except Exception as e:
        return None, str(e)
```

**Human**
```python
def find_user(user_id: int) -> User:
    """Raises UserNotFound if no user with this ID exists."""
    return db.query(user_id)
```

Rule: pick one error signaling mechanism per layer and use it consistently — mixing tuples and exceptions forces every caller to defend against both.

---

## 11. "Gracefully" in error handling comments

LLMs use "gracefully" as a filler word that signals intent without describing behavior. It is a comfort word. The reader needs to know what actually happens, not that it happens in a pleasant way.

**LLM**
```python
# Handle the error gracefully
# Fail gracefully if the service is unavailable
# Gracefully degrade when the cache is down
```

**Human**
```python
# Return cached value if Redis is unreachable; stale for up to 60s
# Skip email notification if SMTP fails; payment still processes
# Serve the last-known config if the config service is down
```

Rule: replace "gracefully" with the specific behavior — what is returned, what is skipped, what the user experiences.

---

## 12. TODO: add proper error handling

LLMs leave TODO comments in error handling positions when they cannot decide what to do. These are broken windows. The comment signals that the code is incomplete but ships anyway.

**LLM**
```python
def load_config(path: str) -> dict:
    with open(path) as f:
        return json.load(f)
    # TODO: add proper error handling

def connect(host: str, port: int):
    # TODO: handle connection errors gracefully
    return socket.create_connection((host, port))
```

**Human**
```python
def load_config(path: str) -> Config:
    try:
        with open(path) as f:
            return Config(**json.load(f))
    except FileNotFoundError:
        raise ConfigNotFound(path) from None
    except json.JSONDecodeError as e:
        raise ConfigInvalid(path, reason=str(e)) from e
```

Rule: either handle the error now or let the exception propagate naturally — a TODO in an error position that survives one commit will survive forever.

---

## 13. The `console.error` + return null pattern

LLMs avoid propagating errors in TypeScript by catching, logging, and returning `null`. The caller receives `null` with no explanation. The error disappears from the call stack. Null checks proliferate throughout the codebase.

**LLM**
```typescript
catch (error) {
    console.error("Failed:", error);
    return null;
}
```

**Human**
```typescript
catch (error) {
    throw new PaymentError(
        `charge failed for order ${orderId}`,
        { cause: error }
    );
}
```

Rule: propagate or transform with context — `return null` on catch forces every caller to defensively check for null without knowing why it happened.

---

## 14. Symmetric error handling that treats all exceptions identically

LLMs produce error handlers that have the same shape for every exception type: same log level, same return value, same message template. Real systems need to distinguish retryable failures from permanent ones, user errors from infrastructure errors, and expected conditions from bugs.

**LLM**
```python
except ValueError as e:
    logger.error(f"Error: {e}")
    return None
except TypeError as e:
    logger.error(f"Error: {e}")
    return None
except RuntimeError as e:
    logger.error(f"Error: {e}")
    return None
```

**Human**
```python
except RateLimitExceeded as e:
    raise RetryAfter(seconds=e.retry_after) from e
except InvalidRequest as e:
    raise  # 4xx; do not retry
except InfrastructureError as e:
    metrics.increment("infra.error", tags={"service": "stripe"})
    raise
```

Rule: each exception type should have a handler appropriate to its nature — retryable vs permanent, user error vs bug, expected vs unexpected.

---

## 15. Error variables always named `e` or `error` with no specificity

LLMs use `e` universally. In nested error handlers or long except blocks, `e` becomes meaningless. The variable name is a free opportunity to communicate what kind of failure occurred.

**LLM**
```python
except Exception as e:
    logger.error(f"Failed: {e}")
```

**Human**
```python
except stripe.StripeError as payment_error:
    logger.warning("stripe charge declined", extra={"error": str(payment_error)})
    raise PaymentDeclined(order_id) from payment_error
```

Rule: name exception variables after the failure they represent — `payment_error`, `parse_failure`, `timeout` — not the generic `e`.

---

Error handling quality is measured by how fast a stranger can diagnose a failure at 2am from just the logs and the exception. If the message omits the offending value and the traceback collapses into a `return None`, you have failed that stranger. Often that stranger is you.
