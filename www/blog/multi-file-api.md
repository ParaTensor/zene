# Building a Multi-File FastAPI Application with Zene

Modern web applications are complex, requiring multiple modules for models, schemas, and database logic. Zene handles this complexity effortlessly, creating a fully structured, multi-file FastAPI project in one go.

In this example, we ask Zene to build a complete Todo List API with SQLite storage.

## The Task

```python
task = """
Create a RESTful API for a Todo List application using FastAPI and SQLite.
The structure should be modular:
1. `models.py`: SQLAlchemy models for the `Todo` table (id, title, description, completed).
2. `schemas.py`: Pydantic schemas for request/response validation (TodoCreate, TodoResponse).
3. `database.py`: Database connection and session management.
4. `crud.py`: Functions to create, read, update, and delete todos.
5. `main.py`: The FastAPI application with endpoints for CRUD operations.

Finally, create a test script `test_api.py` using `requests` to verify that we can create and list todos.
"""
```

## Execution Process

### 1. Planning
The Planner (DeepSeek) identifies the dependencies between files:
1.  **Dependencies**: Start by creating `requirements.txt` (fastapi, uvicorn, sqlalchemy).
2.  **Database Layer**: Create `database.py` first, as other files depend on the session.
3.  **Models**: Create `models.py` next, defining the table structure.
4.  **Schemas**: Define Pydantic models in `schemas.py` for API validation.
5.  **CRUD Logic**: Implement `crud.py` using the models and database session.
6.  **API Endpoints**: Wire everything together in `main.py`.
7.  **Verification**: Write and run `test_api.py`.

### 2. Execution & Reflection
-   **Step 1 (Setup)**: Zene creates the project structure and installs dependencies.
-   **Step 3 (Models)**: The Executor writes the SQLAlchemy model. The Reflector checks for correct imports and table definitions.
-   **Step 6 (Main)**: The Executor implements the API endpoints. It correctly handles dependency injection for the database session (`Depends(get_db)`).
-   **Step 7 (Test)**: Zene writes a test script that:
    1.  Starts the FastAPI server in a background thread.
    2.  Sends a POST request to create a todo.
    3.  Sends a GET request to verify it was saved.
    4.  Asserts the response status codes and JSON content.

## The Result

A complete, production-ready project structure is generated:

```
workspace/
├── database.py
├── models.py
├── schemas.py
├── crud.py
├── main.py
└── test_api.py
```

**`main.py` Snippet**:
```python
from fastapi import FastAPI, Depends, HTTPException
from sqlalchemy.orm import Session
from . import crud, models, schemas
from .database import SessionLocal, engine

models.Base.metadata.create_all(bind=engine)

app = FastAPI()

def get_db():
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()

@app.post("/todos/", response_model=schemas.TodoResponse)
def create_todo(todo: schemas.TodoCreate, db: Session = Depends(get_db)):
    return crud.create_todo(db=db, todo=todo)
```

## Key Takeaway
Zene understands software architecture. It doesn't just dump code into a single file; it respects separation of concerns, creates modular components, and manages dependencies between files intelligently.
