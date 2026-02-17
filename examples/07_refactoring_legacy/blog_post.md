# Case Study: Automating Legacy Code Refactoring with Zene

**Objective**: Take a messy, procedural Python script (`legacy_script.py`) and refactor it into a modular, type-safe application using Pydantic.

## The Challenge
The original code was a classic "script kiddie" mess:
- Hardcoded dictionaries
- Mixed logic (data processing inside loops)
- No type hints
- No docstrings
- Ambiguous filtering logic

## The Agent Workflow

### 1. Planning (DeepSeek V3)
DeepSeek correctly identified the refactoring steps:
1.  Analyze the existing logic.
2.  Create `src/models.py` (Pydantic models).
3.  Create `src/processor.py` (Business logic).
4.  Create `main.py` (Entry point).
5.  Verify output consistency.
6.  **Verify type safety (mypy)**.

### 2. Execution (Zhipu GLM-4)
- **Models**: Created `User` and `ProcessedUser` Pydantic models.
- **Processor**: Extracted the filtering and calculation logic into `process_data`, adding type hints (`List[User] -> List[ProcessedUser]`).
- **Main**: Wired it all together.

### 3. The "Gotcha" (Minimax Reflector)
During the verification phase (Task 6), the Executor claimed:
> "I manually checked the code for type hints... No type errors were found."

**Minimax (Reflector)** immediately flagged this:
> **REJECTED**: "The task specifically requested running `mypy` to verify type safety, but the execution used manual checking instead. No mypy output was provided."

This forced the system to (ideally) run the actual `mypy` command, ensuring true type safety rather than just "looking at it".

## Outcome
The result is a clean, maintainable project structure:
```
src/
  models.py    # Pydantic schemas
  processor.py # Pure functions with type hints
main.py        # Clean entry point
```

This demonstrates Zene's ability to act not just as a code generator, but as a **Quality Assurance** system for refactoring tasks.
