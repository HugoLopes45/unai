# Naming

LLMs name things the way documentation writers do: for someone who has never seen the codebase, has no context, and needs every concept spelled out. This produces names that are accurate, long, and wrong for production code. They narrate the type, the lifecycle phase, the container kind. Humans name for the reader who already knows what they're working on.

This document catalogs every pattern. Each one has a cause, a before/after, and a fix rule.

---

## 1. Verbose compound names that describe the type, not the concept

LLMs append the type or container kind to the name. The training corpus is full of tutorial code and Stack Overflow answers where these annotations help beginners who do not know what a dict or a list is. In production code it is noise: the type annotation, context, or usage already communicates the kind.

LLM:
```python
userDataObject = get_user(id)
configurationSettings = load_config()
errorMessageString = "not found"
listOfUsers = db.query(User)
dictionaryOfMappings = {}
```

Human:
```python
user = get_user(id)
config = load_config()
error = "not found"
users = db.query(User)
mapping = {}
```

Fix: name the concept, not the container. If the variable holds a user, name it user. The type annotation or the call site carries everything else.

---

## 2. "Manager", "Handler", "Helper", "Util", "Service" suffix abuse

These suffixes signal that the author did not know what the thing actually does. LLMs reach for them because documentation and enterprise codebases use them constantly — they sound professional and are semantically empty, which makes them safe choices when you are uncertain. The result is a class whose name says nothing except "this does stuff related to X."

LLM:
```python
class DatabaseConnectionManager: ...
class UserAuthenticationHandler: ...
class StringFormattingHelper: ...
class DataProcessingUtility: ...
class ApiRequestService: ...
```

Human:
```python
class DB: ...
class Auth: ...
def format_string(...): ...
def process(...): ...
class API: ...
```

Fix: name the role precisely. If you cannot, the abstraction is wrong. A class that manages database connections is a pool, or a registry, or a session — not a manager.

---

## 3. "Comprehensive", "Enhanced", "Advanced", "Improved" prefixes

LLMs add these adjectives when generating a replacement or upgrade. They come from documentation phrasing: "our enhanced error handling", "a more comprehensive solution". In code they mean nothing. There is no corresponding ComprehensiveDataProcessor and BasicDataProcessor — just a class that the author decorated with flattery.

LLM:
```python
class ComprehensiveDataProcessor: ...
class EnhancedUserValidator: ...
class AdvancedSearchEngine: ...
def improvedCalculation(): ...
```

Human:
```python
class DataProcessor: ...
class UserValidator: ...
class Search: ...
def calculate(): ...
```

Fix: delete the adjective. The name should describe what the thing is, not how good it is. If the word "improved" is load-bearing, the old version still exists somewhere and needs to be deleted or the names need to reflect an actual distinction (e.g., ExactSearch vs FuzzySearch).

---

## 4. initialize vs init vs setup

LLMs default to the full word. Stack Overflow answers spell things out because they are written for audiences who may not know the abbreviation. Humans who work in a codebase every day use init. The same applies to setup vs setUp vs configure: context determines the right choice, but length alone is not a virtue.

LLM:
```python
def initializeDatabase(): ...
def initializeConnectionPool(): ...
def performInitialization(): ...
```

Human:
```python
def init_db(): ...
def init_pool(): ...
def setup(): ...
```

Fix: prefer the short form your language community uses. Python uses init. Rust uses new or init. Go uses New as a constructor convention and init for package-level setup.

---

## 5. Boolean variables that do not read as questions

LLMs prefix every boolean with is, has, or should. This comes from Java naming conventions and from style guides that were written to enforce a rule, not to produce readable code. The result is longer than necessary and reads awkwardly in conditionals.

LLM:
```python
isUserAuthenticated = True
hasCompletedInitialization = False
shouldPerformValidation = True
```

Human:
```python
authenticated = True
initialized = False
validate = True
```

Fix: use a past participle (initialized, authenticated) or a plain adjective (valid, ready, done). Reserve the is/has prefix for when the sentence would be ambiguous without it: is_file is clearer than file when the variable sits next to path.

---

## 6. "process", "handle", "perform", "execute", "facilitate" as verb prefixes

These verbs are chosen because they are safe and universal: anything can be processed, handled, or executed. LLMs use them when they cannot identify what the function actually does. The function name ends up describing an activity (processing) rather than an outcome (the thing produced or changed).

LLM:
```python
def processUserRequest(): ...
def handleDataTransformation(): ...
def performValidationCheck(): ...
def executeBusinessLogic(): ...
def facilitateUserOnboarding(): ...
```

Human:
```python
def serve(): ...
def transform(): ...
def validate(): ...
def run(): ...
def onboard(): ...
```

Fix: name the domain action, not the meta-action. serve, transform, validate, parse, emit, flush — these say what happens. If you cannot name the domain action, the function does too many things.

---

## 7. Acronym avoidance

LLMs spell out what working developers abbreviate. This is the most consistent signal across languages. It comes from documentation style (spell things out on first use) applied indiscriminately to every identifier in every file.

LLM:
```python
identifier = generate_identifier()
configuration = load_configuration()
maximum_value = get_maximum()
minimum_value = get_minimum()
temporary_file = create_temporary()
application_instance = create_application()
database_connection = connect_to_database()
authentication_token = get_authentication_token()
```

Human:
```python
id = generate_id()
cfg = load_config()
max_val = get_max()
min_val = get_min()
tmp = create_tmp()
app = create_app()
db = connect()
token = get_auth_token()
```

Fix: use the abbreviation your language community has standardized. id, cfg, db, auth, tmp, max, min, app are universally understood. Spell out only when the abbreviation is genuinely ambiguous in context.

---

## 8. Mirrored redundancy (name mirrors the type annotation)

When a variable's type is already explicit in the annotation, LLMs add the type name into the identifier anyway. The reader now sees the same information twice. This pattern appears most often in typed languages where the model has learned that annotations and names coexist, but has not learned that they serve different purposes.

LLM:
```typescript
const usersList: User[] = []
const configObject: Config = {}
const errorMessage: string = ""
```

Human:
```typescript
const users: User[] = []
const config: Config = {}
const err: string = ""
```

Fix: let the annotation carry the type; let the name carry the meaning. If the name and the annotation say the same thing, delete from the name.

---

## 9. Over-specific loop variables

Loop variables are ephemeral. Their scope is tiny. LLMs treat them like module-level declarations, giving them names that include both the item kind and the collection kind. The name outlives its usefulness by one character.

LLM:
```python
for currentItem in itemCollection:
    ...
for userRecord in userRecordsList:
    ...
for dataEntry in dataEntries:
    ...
```

Human:
```python
for item in items:
    ...
for user in users:
    ...
for entry in entries:
    ...
```

Fix: singular of the collection name. items -> item, users -> user, entries -> entry. Nothing else is needed.

---

## 10. "result" as a catch-all variable

result is the LLM's equivalent of the placeholder comment // TODO. It signals that the author knows a value is returned but has not named what that value is. Used once in a function, it is tolerable. Used as the default name for every return value across a codebase, it destroys the reader's ability to track data flow.

LLM:
```python
result = fetch_user(id)
result = calculate_total(items)
result = validate_input(data)
```

Human:
```python
user = fetch_user(id)
total = calculate_total(items)
ok, err = validate_input(data)
```

Fix: name what was returned, not the act of returning. The function name already says an action happened; the variable should say what you now have.

---

## 11. Class names that include "Class" or abstract nouns

AbstractBase is a Java pattern that leaked into every other language via tutorials written by Java developers. The words Abstract, Base, Entity, Object, Container, and Class add zero information to a type name in a language where every class is abstract until instantiated, every object is an object, and every class is a class.

LLM:
```python
class AbstractBaseProcessor: ...
class UserEntityClass: ...
class DataContainerObject: ...
```

Human:
```python
class Processor: ...
class User: ...
class Container: ...
```

Fix: delete the decorative nouns. If the class is abstract in the Python ABC sense, name it for the protocol it defines: Renderable, Serializable, Store. Not AbstractBaseRenderable.

---

## 12. Constants that describe themselves instead of the domain

LLMs produce constant names that include units, the word "default", the word "value", and the word "count" — all derivable from the type or context. The constant reads like a sentence explaining itself, which defeats the point of a named constant.

LLM:
```python
MAX_RETRY_ATTEMPT_COUNT = 3
DEFAULT_TIMEOUT_VALUE_IN_SECONDS = 30
MINIMUM_PASSWORD_LENGTH_REQUIREMENT = 8
```

Human:
```python
MAX_RETRIES = 3
TIMEOUT = 30
MIN_PASSWORD_LEN = 8
```

Fix: name the constraint, not its implementation details. Units belong in a comment when they are not obvious from usage. DEFAULT belongs only when there is a non-default variant that coexists.

---

## 13. Callback and handler naming

LLMs add three layers to event handler names: the verb (handle, process), the target (Button, Form), and the event type (ClickEvent, SubmitCallback). This is documentation written into an identifier. In the context of a UI component, onClick already tells you everything.

LLM:
```typescript
const handleButtonClickEvent = () => {}
const onUserSubmitFormCallback = () => {}
const processMouseMoveEventHandler = () => {}
```

Human:
```typescript
const onClick = () => {}
const onSubmit = () => {}
const onMouseMove = () => {}
```

Fix: on + event name. The component already provides the noun. If the same component has two click handlers, name them by what they do: onAddItem, onRemoveItem — not onAddItemButtonClick.

---

## 14. File and module names that are too generic

utils.py is a drawer that everything gets thrown into. LLMs reach for utils, helpers, common, shared, and base because these names are always technically correct: a utility file contains utilities, after all. The problem is that the name does not tell anyone what to look for or where to add new code.

LLM:
```
utils.py
helpers.ts
common.rs
shared.py
base.ts
```

Human:
```
format.py
retry.ts
pool.rs
codec.py
router.ts
```

Fix: name the module for the thing it does, not for the category of things it might contain. If a module has grown into an actual utility drawer, split it by responsibility. format.py, retry.ts, pool.rs — each of these tells a new contributor exactly what is inside.

---

## The human naming heuristics LLMs miss

Beyond avoiding the patterns above, experienced developers use naming to encode information that a type system cannot express. LLMs almost never do this.

### Names that encode constraints

Prefixes and suffixes can carry invariants without comments. max_connections tells you there is a ceiling. min_latency tells you there is a floor. A Rust function named parse_or_panic signals to every caller that this is not a recoverable error path. These conventions are informal but they are real contracts.

```python
MAX_CONNECTIONS = 100
MIN_INTERVAL_MS = 50
DEADLINE_SECS = 5
```

```rust
fn parse_or_panic(s: &str) -> Config { ... }
fn checked_add(a: u32, b: u32) -> Option<u32> { ... }
```

### Names that signal mutability conventions

Some communities use naming to distinguish mutable from immutable values. JavaScript/TypeScript codebases using functional style often suffix mutable accumulator variables with a prime (result vs result_). Rust uses mut as a keyword, so the name itself rarely needs to carry it — but a Rust developer writing mut users is not doing the same thing as a Python developer. The convention differs and LLMs flatten them.

```typescript
// Immutable original, mutable working copy
const config = load()
let cfg = { ...config }
```

```rust
let items = fetch_items();      // immutable
let mut items = fetch_items();  // explicitly mutable
```

### Idiomatic abbreviations per language

Every language community has settled on a short vocabulary. Using the community abbreviation reads as fluent; spelling it out reads as foreign.

```
Language  | Common idioms
----------|--------------
Rust      | impl, fn, mut, cfg, buf, len, cap, idx, ptr, rx, tx
Python    | cls (class method arg), cfg, exc, tb, fp, fh, seq
Go        | r (receiver or io.Reader), w (io.Writer), ctx, n, err
TypeScript| el, ref, cb, evt, fn, idx
```

LLMs consistently avoid these. They write receiver instead of r, callback instead of cb, exception instead of exc. The result is code that compiles but does not belong to the language.

### When to break the rules intentionally

Single-letter variables are correct in some contexts. A tight numeric loop is one of them. A mathematical formula is another. The rule is scope: the shorter the scope and the more obvious the role, the shorter the name should be.

```python
# Fine: i, j, k in numeric loops
for i in range(n):
    for j in range(m):
        matrix[i][j] = 0

# Fine: x, y in geometry
def distance(x1, y1, x2, y2): ...

# Not fine: single letters at module scope or in complex logic
u = get_current_user()   # too short for too large a scope
```

LLMs almost never use single-letter variables, even in tight loops where they are the right choice. This produces userIndex, currentRow, columnValue in contexts where i, r, and v would be clearer to any developer who has read a textbook.

### The underlying heuristic

Name length should scale with scope. A variable that lives for three lines can be three characters. A type exported from a library should be unambiguous without context. A constant used across a codebase should be self-explanatory. LLMs apply documentation-level naming everywhere, regardless of scope. The fix is to match name length to the distance between definition and use.
