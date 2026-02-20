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