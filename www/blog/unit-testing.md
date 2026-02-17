# Writing Unit Tests with Zene

Tests are essential for reliable software, but writing them is tedious. Zene can generate comprehensive test suites, ensuring your code is robust and bug-free.

In this example, we ask Zene to write tests for a Python math module.

## The Task

```python
task = """
I have a python module `math_utils.py` containing some functions.
Please create a comprehensive test suite `test_math_utils.py` using `pytest`.

Requirements:
1. Test `calculate_factorial` with positive integers, 0, and 1.
2. Test `calculate_factorial` raises ValueError for negative inputs.
3. Test `is_prime` with prime numbers, composite numbers, 1, 0, and negative numbers.
4. Test `safe_divide` with normal division and division by zero.
5. Use parametrized tests where appropriate to cover multiple cases.
6. Run the tests using `pytest` and confirm they pass.
"""
```

## Execution Process

### 1. Planning
The Planner identifies the structure of the module and the testing framework (`pytest`).
1.  **Analyze**: Read `math_utils.py` to understand function signatures and edge cases.
2.  **Test Suite**: Create `test_math_utils.py` with `pytest` fixtures and parametrization.
3.  **Run**: Execute `pytest` and fix any failures.

### 2. Execution & Reflection
-   **Step 1 (Analyze)**: The Executor reads the source code.
-   **Step 2 (Generate Tests)**: It writes `test_math_utils.py` using `@pytest.mark.parametrize` for clean, data-driven tests.
    -   *Detail*: It includes tests for `factorial(0) == 1`, `factorial(-1)` raising `ValueError`, and `is_prime(1) == False`.
-   **Step 3 (Run)**: It executes `pytest`.
    -   *Detail*: The initial run passes, but if the `is_prime` function had a bug (e.g., returning True for 1), the test would fail, and the Reflector would catch it.

## The Result

**test_math_utils.py**:
```python
import pytest
from math_utils import calculate_factorial, is_prime, safe_divide

@pytest.mark.parametrize("n,expected", [
    (0, 1),
    (1, 1),
    (5, 120),
])
def test_calculate_factorial(n, expected):
    assert calculate_factorial(n) == expected

def test_calculate_factorial_negative():
    with pytest.raises(ValueError):
        calculate_factorial(-1)

@pytest.mark.parametrize("n,expected", [
    (2, True),
    (3, True),
    (4, False),
    (1, False),
    (0, False),
    (-5, False),
])
def test_is_prime(n, expected):
    assert is_prime(n) == expected

def test_safe_divide_zero():
    assert safe_divide(10, 0) is None
```

## Key Takeaway
Zene doesn't just write happy-path tests. It considers edge cases (negative numbers, zero division) and uses best practices like parametrization to ensure comprehensive coverage.
