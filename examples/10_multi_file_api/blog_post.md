# Multi-File API: AI as a Backend Architect

Building a complex application requires more than just generating a single file. It requires understanding architecture, imports, database models, and API routes.

## The Experiment
We task Zene with building a REST API using FastAPI. The requirements are:
1. **Models**: Define Pydantic models and SQLAlchemy schemas in `src/models.py`.
2. **Database**: Setup SQLite connection in `src/database.py`.
3. **Routes**: Implement CRUD endpoints in `src/routers/users.py`.
4. **App**: Wire everything together in `main.py`.

This scenario tests the Planner's ability to structure a project (creating multiple files and directories) and the Executor's ability to handle cross-file imports and dependencies. The Reflector will verify that the server can actually start and respond to requests.
