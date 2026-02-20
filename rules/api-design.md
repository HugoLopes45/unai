# API design anti-patterns

LLMs design APIs by anticipating every possible caller need and exposing it through a single function with enough parameters to cover all of them. Humans design APIs around the most common use case and make uncommon cases explicit. The difference is felt by every caller, in every refactor, forever.

---

## 1. Boolean flag parameters

**LLM**
```python
def get_user(
    user_id: int,
    include_deleted: bool = False,
    with_permissions: bool = True,
    as_dict: bool = False,
    eager_load: bool = True,
) -> User | dict:
```

**Human**
```python
def get_user(user_id: int) -> User: ...
def get_deleted_user(user_id: int) -> User: ...
def get_user_permissions(user_id: int) -> list[Permission]: ...
```

Rule: boolean parameters are branching logic disguised as an argument. Each `True/False` doubles the number of code paths and forces callers to know which combination of flags is valid. Separate the behaviors into separate functions. Names are cheaper than flags.

---

## 2. "Data" and "Info" suffix types

**LLM**
```python
def get_user_data(user_id: int) -> dict:
def fetch_config_info() -> dict:
def load_session_data() -> dict:
def retrieve_product_info(sku: str) -> dict:
```

**Human**
```python
def get_user(user_id: int) -> User:
def load_config() -> Config:
def get_session() -> Session:
def get_product(sku: str) -> Product:
```

Rule: "data" and "info" are placeholders for a real type name. If you cannot name the return type, that is a signal to define one. `dict` return types push the burden of knowing the shape onto every caller. Named types are self-documenting, type-checkable, and refactorable.

---

## 3. **kwargs catch-alls for "extensibility"

**LLM**
```python
def process(data: bytes, **kwargs) -> Result:
    """Process data. kwargs are passed through for extensibility."""

def send_request(url: str, **options) -> Response:
    """Send request. options supports future parameters."""
```

**Human**
```python
def process(data: bytes, timeout: int = 30, retries: int = 3) -> Result:

def send_request(url: str, timeout: int = 30, headers: dict[str, str] | None = None) -> Response:
```

Rule: `**kwargs` destroys the contract. Callers cannot discover valid parameters from the signature. Type checkers cannot validate them. IDE autocomplete breaks. "Extensibility" is not a reason to hide parameters; it is a reason to design them carefully now.

---

## 4. Over-returning

**LLM**
```python
def create_user(name: str, email: str) -> dict:
    # returns:
    # {
    #   "user": {...},
    #   "status": "created",
    #   "message": "User created successfully",
    #   "id": 42,
    #   "timestamp": "2024-01-01T00:00:00Z"
    # }
```

**Human**
```python
def create_user(name: str, email: str) -> User:
    ...
```

Rule: return the thing the function creates. If the caller needs the ID, it is on the `User`. If they need the timestamp, it is on the `User`. Wrapping in a status envelope forces every caller to unwrap it and makes the actual return value harder to access. REST APIs sometimes need envelopes; internal functions do not.

---

## 5. Optional everywhere

**LLM**
```typescript
function getUser(
    id?: number,
    name?: string,
    email?: string
): User | null | undefined {
```

**Human**
```typescript
function getUserById(id: number): User        // throws UserNotFound
function findUserByEmail(email: string): User | null  // null = not found, never undefined
```

Rule: `Optional` parameters signal that the function does not know what it needs. Each optional parameter is a question the caller has to answer. Make required things required. When absence is meaningful, use `null` with a clear documented meaning, not `undefined | null | T`.

---

## 6. God functions

**LLM**
```python
def handle_registration(
    name: str,
    email: str,
    password: str,
    plan: str,
    referral_code: str | None = None,
) -> dict:
    # validate inputs
    # check email uniqueness
    # hash password
    # create user in database
    # create subscription
    # apply referral discount
    # send welcome email
    # send admin notification
    # log analytics event
    # return user + token + subscription details
```

**Human**
```python
def register_user(name: str, email: str, password: str) -> User:
    ...  # validate, create, persist

def start_subscription(user: User, plan: Plan) -> Subscription:
    ...

def apply_referral(subscription: Subscription, code: str) -> Subscription:
    ...
```

Rule: a function that validates, persists, emails, logs, and returns a status dict cannot be tested, reused, or reasoned about in isolation. Each step should be a function. The orchestration belongs in the caller or in a use-case layer, not baked into a single function.

---

## 7. Mirrored getters and setters for everything

**LLM**
```python
class User:
    def get_name(self) -> str: return self._name
    def set_name(self, name: str) -> None: self._name = name
    def get_email(self) -> str: return self._email
    def set_email(self, email: str) -> None: self._email = email
    def get_age(self) -> int: return self._age
    def set_age(self, age: int) -> None: self._age = age
```

**Human**
```python
@dataclass
class User:
    name: str
    email: str
    age: int

    def rename(self, new_name: str) -> "User":
        if not new_name.strip():
            raise ValueError("name cannot be blank")
        return replace(self, name=new_name.strip())
```

Rule: Java-style getters and setters in Python are almost never correct. Use `@property` when you need computed access or validation. Use dataclasses or attrs for plain data. Expose domain methods (`rename`, `suspend`, `promote`) rather than raw field access.

---

## 8. Async everything

**LLM**
```python
async def get_username(user: User) -> str:
    return user.name

async def calculate_total(items: list[Item]) -> Decimal:
    return sum(item.price for item in items)

async def is_admin(user: User) -> bool:
    return user.role == "admin"
```

**Human**
```python
def get_username(user: User) -> str:
    return user.name

def calculate_total(items: list[Item]) -> Decimal:
    return sum(item.price for item in items)

async def fetch_permissions(user_id: int) -> list[str]:
    return await db.query(...)  # actually async
```

Rule: async is a contagious annotation that propagates to every caller. Functions that do no I/O should not be async. Marking pure computation async forces callers into async context for no reason and makes the function harder to use in synchronous code.

---

## 9. Configuration objects with no schema

**LLM**
```python
def create_server(config: dict) -> Server:
    host = config.get("host", "localhost")
    port = config.get("port", 8080)
    debug = config.get("debug", False)
    # 20 more config.get() calls
```

**Human**
```python
@dataclass(frozen=True)
class ServerConfig:
    host: str = "localhost"
    port: int = 8080
    debug: bool = False
    workers: int = 4

def create_server(config: ServerConfig) -> Server:
    ...
```

Rule: `dict` configs are stringly-typed. Typos in keys fail silently at runtime, not at the call site. A typed config class gives IDE completion, type checking, and a single place to document each option. It is not over-engineering; it is the minimum viable contract.

---

## 10. The options object with fifteen keys

**LLM**
```typescript
function fetchReport(id: string, options: {
    format?: "pdf" | "csv" | "json",
    includeSummary?: boolean,
    includeCharts?: boolean,
    dateRange?: [Date, Date],
    groupBy?: string,
    filters?: Record<string, unknown>,
    maxRows?: number,
    timeout?: number,
    cache?: boolean,
    cacheKey?: string,
    retry?: boolean,
    retryCount?: number,
    onProgress?: (pct: number) => void,
    signal?: AbortSignal,
    locale?: string,
} = {}): Promise<Report>
```

**Human**
```typescript
function fetchReport(id: string, format: ReportFormat): Promise<Report>
function fetchReportWithFilters(id: string, query: ReportQuery): Promise<Report>
```

Rule: an options bag with fifteen optional keys is not one function, it is fifteen functions that share an implementation. Each key adds a code path. Design for the primary use case. If secondary use cases exist, expose them through additional functions with honest signatures.

---

Human API design follows the principle of least surprise: calling code should read naturally, the common case should be the easy case, and errors should be impossible to ignore. When an API requires reading the implementation to use it correctly, the API has failed. Names, types, and signatures are the documentation.
