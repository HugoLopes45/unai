# API design anti-patterns

LLMs design APIs for the code review, not for the caller. They optimize for symmetry, comprehensiveness, and apparent flexibility — producing interfaces that are harder to use correctly than a simpler alternative would be. Good API design is about reducing the surface area a caller must understand to accomplish one thing.

---

## 1. Boolean flag parameters

LLMs add boolean flags when they need slightly different behavior from an existing function. Each flag multiplies internal branching and forces callers to read the signature to understand what `True` does at the call site.

**LLM**
```python
def get_user(
    user_id: str,
    include_deleted: bool = False,
    with_permissions: bool = True,
    as_dict: bool = False,
) -> User | dict | None:
```

**Human**
```python
def get_user(user_id: str) -> User: ...
def get_deleted_user(user_id: str) -> User: ...
def get_user_permissions(user_id: str) -> list[Permission]: ...
```

Rule: when a boolean parameter changes what a function does rather than how it does it, split into separate functions with clear names.

---

## 2. "Data" and "Info" suffixes on return types

LLMs name functions and types with generic suffixes that add no information: `UserData`, `ConfigInfo`, `OrderDetails`, `ResponseObject`. These names exist because the LLM ran out of vocabulary after the noun.

**LLM**
```python
def get_user_data() -> dict:
def fetch_config_info() -> dict:
def retrieve_order_details() -> OrderDetails:
```

**Human**
```python
def get_user() -> User:
def load_config() -> Config:
def get_order() -> Order:
```

Rule: name the thing, not the fact that it is data — if you need a projection, use `UserSummary` or `UserProfile`, not `UserData`.

---

## 3. `**kwargs` catch-alls for "extensibility"

LLMs add `**kwargs` with a comment about future extensibility. The parameter absorbs unknown arguments silently, makes type checking impossible, and hides the real interface behind a comment.

**LLM**
```python
def process(data: dict, **kwargs) -> Result:
    """Process data. kwargs for extensibility."""
    timeout = kwargs.get("timeout", 30)
    retries = kwargs.get("retries", 3)
```

**Human**
```python
def process(data: dict, timeout: int = 30, retries: int = 3) -> Result:
```

Rule: use explicit typed parameters — `**kwargs` makes the type signature a lie and shifts the documentation burden onto the reader.

---

## 4. Over-returning — more than the caller asked for

LLMs return rich response objects from simple operations. A function named `create_user` returns a dict containing the user, a status code, a message, and the ID separately. The caller asked for a user.

**LLM**
```python
def create_user(email: str, name: str) -> dict:
    user = User(email=email, name=name)
    db.save(user)
    return {
        "user": user,
        "id": user.id,
        "status": "created",
        "message": "User created successfully",
        "created_at": user.created_at.isoformat(),
    }
```

**Human**
```python
def create_user(email: str, name: str) -> User:
    user = User(email=email, name=name)
    db.save(user)
    return user
```

Rule: return the thing the function created or found — callers who need status or timestamps can read them from the object.

---

## 5. Optional everywhere

LLMs mark parameters as optional as a defensive habit. Every parameter might be absent; every return might be null. The result is a function where nothing is required and nothing is guaranteed.

**LLM**
```typescript
function getUser(
    id?: number,
    name?: string,
    email?: string,
): User | null | undefined
```

**Human**
```typescript
function getUser(id: number): User  // throws UserNotFound if absent
```

Rule: require what is truly required; make the return type non-nullable unless absence is a legitimate, expected outcome for the caller to handle.

---

## 6. God functions that do everything

LLMs combine validation, transformation, persistence, and notification in one function because all those things "relate to" the operation. The function cannot be tested without standing up the entire dependency graph.

**LLM**
```python
def register_user(email: str, password: str, send_welcome: bool = True) -> dict:
    validate_email(email)
    validate_password_strength(password)
    if db.users.find_by_email(email):
        raise DuplicateEmail(email)
    hashed = hash_password(password)
    user = db.users.create(email=email, password_hash=hashed)
    if send_welcome:
        email_client.send_welcome(user)
    analytics.track("user_registered", user_id=user.id)
    audit_log.write("registration", user_id=user.id)
    return {"user": user, "status": "ok"}
```

**Human**
```python
def register_user(email: str, password: str) -> User:
    validate_email(email)
    ensure_not_duplicate(email)
    return users.create(email, hash_password(password))

# Caller or application layer handles notification and analytics
```

Rule: a function should do one thing; side effects like notifications and analytics belong in the caller or a separate application layer.

---

## 7. Mirrored get/set for everything

LLMs generate full getter/setter pairs for every attribute, even in Python where direct attribute access or `@property` is idiomatic. The pattern imports Java conventions into languages that do not need them.

**LLM**
```python
class User:
    def get_name(self) -> str:
        return self._name

    def set_name(self, name: str) -> None:
        self._name = name

    def get_email(self) -> str:
        return self._email

    def set_email(self, email: str) -> None:
        self._email = email
```

**Human**
```python
@dataclass
class User:
    name: str
    email: str

    @property
    def email(self) -> str:
        return self._email

    @email.setter
    def email(self, value: str) -> None:
        self._email = validate_email(value)
```

Rule: use direct attribute access in Python; use `@property` only when the accessor needs validation or computation — never generate get/set pairs by default.

---

## 8. Async everything

LLMs mark functions `async` by default when writing modern Python or TypeScript. Functions that do pure computation, string manipulation, or in-memory work become coroutines for no reason, requiring `await` at every call site and hiding which operations are actually concurrent.

**LLM**
```python
async def format_currency(amount: Decimal, currency: str) -> str:
    return f"{currency} {amount:.2f}"

async def calculate_tax(subtotal: Decimal, rate: Decimal) -> Decimal:
    return subtotal * rate
```

**Human**
```python
def format_currency(amount: Decimal, currency: str) -> str:
    return f"{currency} {amount:.2f}"

def calculate_tax(subtotal: Decimal, rate: Decimal) -> Decimal:
    return subtotal * rate
```

Rule: mark a function `async` only when it performs I/O or awaits another coroutine — unnecessary `async` pollutes the call graph and misleads readers about what is concurrent.

---

## 9. Config objects that are just dicts with no schema

LLMs pass configuration as `dict` or `Any` because it is flexible. The keys are undocumented, the values are untyped, and typos in key names fail silently at runtime rather than at construction time.

**LLM**
```python
def create_client(config: dict) -> Client:
    host = config.get("host", "localhost")
    port = config.get("port", 5432)
    timeout = config.get("timeout", 30)
```

**Human**
```python
@dataclass(frozen=True)
class ClientConfig:
    host: str = "localhost"
    port: int = 5432
    timeout: int = 30

def create_client(config: ClientConfig) -> Client:
```

Rule: config is a domain object — give it a type with defaults, validation, and documentation at the field level.

---

## 10. The options parameter with undocumented keys

LLMs add an `options` or `config` parameter that accepts arbitrary keys "for flexibility." Callers cannot know what keys exist without reading the implementation. Unused keys are silently ignored.

**LLM**
```python
def send_email(to: str, subject: str, body: str, options: dict = {}) -> None:
    retry = options.get("retry", 3)
    timeout = options.get("timeout", 10)
    template = options.get("template", "default")
    cc = options.get("cc", [])
    bcc = options.get("bcc", [])
```

**Human**
```python
def send_email(
    to: str,
    subject: str,
    body: str,
    *,
    cc: list[str] = (),
    bcc: list[str] = (),
    template: str = "default",
    retry: int = 3,
    timeout: int = 10,
) -> None:
```

Rule: every option a function supports must be an explicit, typed, documented parameter — an untyped `options` dict is an undocumented API.

---

## 11. Premature interface abstraction

LLMs generate repository interfaces with two or three concrete implementations before a second real implementation exists. The abstraction costs readability and indirection without enabling anything. The `InMemoryRepository` for tests is the only "second implementation" ever written.

**LLM**
```typescript
interface UserRepository {
    findById(id: string): Promise<User>;
    findByEmail(email: string): Promise<User | null>;
    save(user: User): Promise<void>;
    delete(id: string): Promise<void>;
}

class PostgresUserRepository implements UserRepository { ... }
class MongoUserRepository implements UserRepository { ... }  // never used
class InMemoryUserRepository implements UserRepository { ... }  // tests only
```

**Human**
```typescript
// Direct functions until a second real storage backend exists
async function findUser(id: string): Promise<User> { ... }
async function saveUser(user: User): Promise<void> { ... }
```

Rule: introduce an interface when a second real implementation exists, not in anticipation of one — functions compose more easily than classes and require no ceremony.

---

## 12. `{ success: boolean, data: T, error: string }` return unions

LLMs avoid exceptions by returning success objects. Every caller must check `result.success` before using `result.data`. The pattern spreads defensive null-checking throughout the codebase and makes it impossible to compose operations without unwrapping at each step.

**LLM**
```typescript
function createUser(email: string): { success: boolean; data?: User; error?: string } {
    try {
        const user = db.users.create(email);
        return { success: true, data: user };
    } catch (e) {
        return { success: false, error: String(e) };
    }
}
```

**Human**
```typescript
function createUser(email: string): User {
    return db.users.create(email);  // throws on failure
}

// Or, if the codebase uses Result consistently:
function createUser(email: string): Result<User, DuplicateEmail | InvalidEmail> {
```

Rule: throw on failure in languages that support exceptions; use a typed `Result` if the codebase has one — never return ad hoc `{ success, data, error }` objects.

---

## 13. Every function returns a detailed response object

LLMs return metadata from operations that should return nothing or just the created entity. A function that deletes a record returns a confirmation object. The caller must inspect a response to know if a side effect occurred.

**LLM**
```python
def delete_user(user_id: str) -> dict:
    user = db.users.find(user_id)
    db.users.delete(user_id)
    return {
        "success": True,
        "message": "User deleted successfully",
        "deleted_at": datetime.utcnow().isoformat(),
        "user_id": user_id,
    }
```

**Human**
```python
def delete_user(user_id: str) -> None:
    db.users.delete(user_id)  # raises UserNotFound if absent
```

Rule: return `None` from mutation functions that succeed; raise on failure — callers who need the deletion timestamp can record it themselves.

---

## 14. Callback parameters that should be events or promises

LLMs wire callbacks into function signatures when the operation is asynchronous or has side effects. Callbacks invert control without providing the composability of promises or the decoupling of events.

**LLM**
```python
def process_file(
    path: str,
    on_success: Callable[[Result], None],
    on_error: Callable[[Exception], None],
    on_progress: Callable[[float], None],
) -> None:
```

**Human**
```python
async def process_file(path: str) -> Result:
    ...  # raises on failure; caller awaits and handles errors
```

Rule: return a promise or raise an exception; pass a callback only when the operation has multiple intermediate events that cannot be represented by a single return value.

---

## 15. The unnecessary wrapper class

LLMs create utility classes as containers for static methods: `StringUtils`, `MathUtils`, `DateHelper`, `ValidationUtils`. These are namespaced functions pretending to be objects. They cannot be instantiated meaningfully and add a class hierarchy that serves no purpose.

**LLM**
```python
class StringUtils:
    @staticmethod
    def slugify(text: str) -> str: ...

    @staticmethod
    def truncate(text: str, max_len: int) -> str: ...

class MathUtils:
    @staticmethod
    def clamp(value: float, min_val: float, max_val: float) -> float: ...
```

**Human**
```python
# strings.py
def slugify(text: str) -> str: ...
def truncate(text: str, max_len: int) -> str: ...

# math.py
def clamp(value: float, min_val: float, max_val: float) -> float: ...
```

Rule: use module-level functions — modules are namespaces; classes are for objects with state and behavior.

---

## 16. APIs designed for the code review, not the caller

LLMs produce APIs that look thorough: symmetrical methods, comprehensive parameters, consistent naming, exhaustive `__all__` exports. The surface area is large and appears professional. Callers must read the entire interface to accomplish one task.

**LLM**
```python
__all__ = [
    "create_user", "read_user", "update_user", "delete_user",
    "create_session", "read_session", "delete_session",
    "create_token", "validate_token", "revoke_token",
    "UserService", "SessionService", "TokenService",
    "UserRepository", "SessionRepository",
]
```

**Human**
```python
__all__ = ["register_user", "login", "logout"]
# Everything else is an implementation detail
```

Rule: export only what callers need — a large `__all__` is a sign that the module boundary is in the wrong place.

---

## 17. Parameter names that encode the type

LLMs name parameters with their type embedded: `userObject`, `configDict`, `itemsList`, `callbackFunction`. This is redundant in typed languages and a readability problem in all languages. It encodes the implementation, not the role.

**LLM**
```python
def process(userObject: User, configDict: dict, itemsList: list[Item]) -> ResultObject:
```

**Human**
```python
def process(user: User, config: Config, items: list[Item]) -> Result:
```

Rule: name parameters after their role or domain concept, not their type — the type annotation already carries that information.

---

Human APIs are asymmetric: they start with the caller's most common need and build outward only when forced by a second real use case. When an LLM designs an API, it starts with a complete surface; when a human designs one, they start with one working call path and let real use cases drive the rest.
