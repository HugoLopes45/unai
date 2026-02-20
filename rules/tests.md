# Test code anti-patterns

LLMs generate tests like a compliance checklist: one per function, happy path only, with elaborate setup and assertion messages that narrate the obvious. Human tests are behavior specifications. They fail for the right reasons and tell you exactly what broke.

---

## 1. Test names that describe code structure, not behavior

LLMs name tests after the function under test. The function becomes the subject. The behavior — the thing that matters — disappears into vague qualifiers like "with_valid_data".

**LLM**
```python
def test_user_creation_with_valid_data():
def test_login_function_returns_true_when_credentials_are_correct():
def test_calculate_tax_method_with_positive_amount_input():
```

**Human**
```python
def test_new_user_requires_email():
def test_login_rejects_wrong_password():
def test_tax_is_zero_on_exempt_items():
```

Rule: name the scenario and expected outcome, not the function under test — if the test fails, the name should tell you what the system stopped doing.

---

## 2. Arrange/Act/Assert comments in every test

LLMs apply the AAA pattern as a template, pasting the three comments regardless of test length. A three-line test with three section headers is scaffolding that was never removed.

**LLM**
```python
def test_calculate_total():
    # Arrange
    items = [Item(price=10), Item(price=20)]
    cart = Cart(items)

    # Act
    result = cart.total()

    # Assert
    assert result == 30
```

**Human**
```python
def test_calculate_total():
    cart = Cart([Item(price=10), Item(price=20)])
    assert cart.total() == 30
```

Rule: use blank lines to separate phases when the test is long enough to need it; the comments are teaching aids, not permanent structure.

---

## 3. `assert x is not None` instead of asserting the actual value

LLMs verify that something was returned before verifying what it contains. The existence check ships instead of the value check.

**LLM**
```python
assert user is not None
assert result is not None
assert response.data is not None
```

**Human**
```python
assert user.id == expected_id
assert result == Decimal("42.50")
assert response.data["status"] == "active"
```

Rule: test the value, not its existence — existence assertions hide incorrect return values that happen to be truthy.

---

## 4. Mock everything, including things that don't need mocking

LLMs default to mocking to avoid I/O. The instinct is right for the network; it is wrong for domain logic, pure functions, and in-memory structures. Five `@patch` decorators on a unit test mean the test is coupled to the implementation, not the behavior.

**LLM**
```python
@patch("myapp.services.uuid4")
@patch("myapp.services.datetime")
@patch("myapp.services.logger")
@patch("myapp.services.db")
@patch("myapp.services.email_client")
def test_register_user(mock_email, mock_db, mock_logger, mock_dt, mock_uuid):
    ...
```

**Human**
```python
def test_register_user():
    repo = MemoryUserRepo()
    svc = UserService(repo)
    user = svc.register("alice@example.com", "hunter2")
    assert repo.find_by_email("alice@example.com") == user
```

Rule: mock external I/O boundaries (network, filesystem, time); use real domain logic and in-memory fakes for everything else.

---

## 5. Happy path only

LLMs generate the obvious success test and stop. Bugs live at the boundaries. No LLM spontaneously tests expired tokens, zero-value inputs, or duplicate registration without being asked.

**LLM**
```python
def test_create_user():
    user = create_user(name="Alice", email="alice@example.com")
    assert user.name == "Alice"
```

**Human**
```python
def test_create_user():
    user = create_user(name="Alice", email="alice@example.com")
    assert user.name == "Alice"

def test_create_user_rejects_empty_name():
    with pytest.raises(ValueError, match="name"):
        create_user(name="", email="alice@example.com")

def test_create_user_rejects_duplicate_email():
    create_user(name="Alice", email="alice@example.com")
    with pytest.raises(DuplicateEmail):
        create_user(name="Bob", email="alice@example.com")

def test_create_user_rejects_malformed_email():
    with pytest.raises(ValueError, match="email"):
        create_user(name="Alice", email="not-an-email")
```

Rule: after writing the success case, immediately ask what inputs should fail, what edge cases exist (empty, zero, max, duplicate, expired), and write those tests.

---

## 6. Over-specified fixture data

LLMs fill every field of a model when only one field is relevant to the test. The noise obscures the signal. It also creates fragile fixtures that break when the model changes.

**LLM**
```python
user = User(
    id="550e8400-e29b-41d4-a716-446655440000",
    email="john.doe@example.com",
    first_name="John",
    last_name="Doe",
    age=30,
    phone="+1-555-555-5555",
    created_at=datetime(2024, 1, 1, 0, 0, 0),
    is_active=True,
    role="admin",
)
```

**Human**
```python
user = UserFactory(email="alice@test.com")
```

Rule: specify only the fields the test cares about; use factories or sensible defaults for everything else.

---

## 7. Testing the mock, not the behavior

LLMs assert that mocks were called instead of asserting what the system produced. This couples tests to implementation details. Refactoring internals breaks the tests even when behavior is unchanged.

**LLM**
```python
def test_save_user():
    mock_db.save.assert_called_once()
    mock_logger.info.assert_called_with("User saved successfully")
    mock_email.send.assert_called_once_with(user.email)
```

**Human**
```python
def test_save_user():
    repo = MemoryUserRepo()
    svc = UserService(repo)
    user = svc.save(User(email="alice@example.com"))
    assert repo.find(user.id).email == "alice@example.com"
```

Rule: test observable outcomes — query the state, check the response body, verify the side effect through a real boundary — not that a mock method was invoked.

---

## 8. Parametrize avoidance

LLMs write a separate test function for every input variant. The functions are nearly identical. Each adds maintenance burden with no additional clarity.

**LLM**
```python
def test_parse_date_january():
    assert parse_date("2024-01-15") == date(2024, 1, 15)

def test_parse_date_december():
    assert parse_date("2024-12-31") == date(2024, 12, 31)

def test_parse_date_leap_year():
    assert parse_date("2024-02-29") == date(2024, 2, 29)

def test_parse_date_invalid_format():
    with pytest.raises(ValueError): parse_date("01-15-2024")

def test_parse_date_invalid_month():
    with pytest.raises(ValueError): parse_date("2024-13-01")
```

**Human**
```python
@pytest.mark.parametrize("value,expected", [
    ("2024-01-15", date(2024, 1, 15)),
    ("2024-12-31", date(2024, 12, 31)),
    ("2024-02-29", date(2024, 2, 29)),
])
def test_parse_date(value, expected):
    assert parse_date(value) == expected

@pytest.mark.parametrize("value", ["01-15-2024", "2024-13-01", "", "not-a-date"])
def test_parse_date_rejects_invalid(value):
    with pytest.raises(ValueError):
        parse_date(value)
```

Rule: when test logic is identical and only inputs vary, use `parametrize` — five near-identical functions are harder to extend and scan than one table.

---

## 9. One test per function, even trivial functions

LLMs mirror the module structure: one test per public method, regardless of whether the method has meaningful behavior to verify. Getters get tests. Constants get tests. Trivial delegation gets tests.

**LLM**
```python
def test_get_name(): ...
def test_set_name(): ...
def test_get_email(): ...
def test_set_email(): ...
def test_get_id(): ...
def test_is_active(): ...
```

**Human**
```python
def test_user_defaults_to_active_on_creation():
    user = User(name="Alice", email="alice@example.com")
    assert user.is_active
    assert user.id is not None

def test_suspension_deactivates_user():
    user = User(name="Alice", email="alice@example.com")
    user.suspend()
    assert not user.is_active
```

Rule: test behavior and invariants, not function existence — if an accessor breaks, a behavior test will catch it.

---

## 10. Assertion messages that repeat the assertion

LLMs add `assert x == y, "Expected x to equal y"` because it looks thorough. Modern test runners print both sides of the assertion on failure. The message adds nothing.

**LLM**
```python
assert result == True, "Expected result to be True"
assert len(users) == 0, "Expected users list to have length 0"
assert user.name == "Alice", "Expected user name to be Alice"
```

**Human**
```python
assert result
assert not users
assert user.name == "Alice"
```

Rule: write a custom message only when the assertion itself is ambiguous; if it is ambiguous, rewrite the assertion first.

---

## 11. Test files named `test_everything.py`

LLMs generate a single test file to "cover the module." Everything ends up in one flat list with no grouping by scenario, domain concept, or feature boundary.

**LLM**
```
tests/
  test_everything.py  # 800 lines, all tests
  test_utils.py
```

**Human**
```
tests/
  auth/
    test_login.py
    test_registration.py
    test_password_reset.py
  billing/
    test_invoicing.py
    test_refunds.py
```

Rule: organize tests by feature or domain, mirroring the source structure — a test file that covers everything covers nothing legibly.

---

## 12. setUp/tearDown that does too much

LLMs construct every possible dependency in `setUp` to cover all tests in the class. Each test then depends on state it did not ask for, making failures hard to trace and test order matter.

**LLM**
```python
class TestUserService(TestCase):
    def setUp(self):
        self.db = create_test_database()
        self.cache = create_redis_mock()
        self.email_service = MockEmailService()
        self.user_service = UserService(self.db, self.cache, self.email_service)
        self.admin_user = create_admin_user(self.db)
        self.regular_user = create_regular_user(self.db)
        self.suspended_user = create_suspended_user(self.db)
        self.test_org = create_organization(self.db)
```

**Human**
```python
def make_user_service(db=None):
    db = db or MemoryUserRepo()
    return UserService(db)

def test_suspend_blocks_login():
    svc = make_user_service()
    user = svc.create("alice@example.com")
    svc.suspend(user.id)
    assert not svc.can_login(user.id)
```

Rule: each test should construct only what it needs using factory functions; shared `setUp` creates hidden coupling and makes individual test failures harder to diagnose.

---

## 13. Tests that test the language, not the code

LLMs generate structural assertions to fill test coverage: checking that a function returns a list, that a dict has keys, that a value is not None. These tests verify that Python works, not that your code does the right thing.

**LLM**
```python
def test_config_is_dict():
    assert isinstance(get_config(), dict)

def test_users_returns_list():
    assert isinstance(get_users(), list)

def test_result_not_none():
    assert process() is not None
```

**Human**
```python
def test_config_contains_required_keys():
    config = get_config()
    assert "database_url" in config
    assert "secret_key" in config

def test_get_users_returns_only_active():
    create_user(active=True)
    create_user(active=False)
    assert all(u.is_active for u in get_users())
```

Rule: test the behavior that matters to the caller — values, ordering, side effects, invariants — not the data structure type.

---

## 14. Perfectly symmetric test methods that mirror function signatures

LLMs produce test suites that are structurally isomorphic to the source module: same order, same names, same arity. The symmetry looks like coverage but is actually a mirror, not a specification. Human test suites are asymmetric because edge cases and failure modes outnumber happy paths.

Rule: if your test file looks like a 1:1 reflection of your source file, you are testing the code's shape, not its behavior.

---

## 15. LLM placeholder tells in fixture data

`"john.doe@example.com"`, `"SecurePassword123!"`, `"Test User"`, `id=1` — these are tell-tale signs of LLM-generated fixtures. They are not wrong, but they reveal that no thought went into what the data represents or why those specific values were chosen.

**LLM**
```python
user = User(
    email="john.doe@example.com",
    password="SecurePassword123!",
    name="Test User",
)
```

**Human**
```python
# Email chosen to test case-insensitive deduplication
user = UserFactory(email="Alice@Example.COM")
```

Rule: if a fixture value is arbitrary, use a factory default; if it is meaningful to the test, comment why that specific value was chosen.

---

Human tests read like specifications: they describe what the system must do, not how the test is structured. When a test fails, the name and assertion together should tell a developer exactly what broke and what the expected behavior was.
