# Infrastructure as Code: Dockerizing Apps with Zene

Writing Dockerfiles and docker-compose configurations is repetitive but error-prone. One wrong instruction, and your container fails to start. Zene understands the nuances of containerization and can automate this process.

In this example, we ask Zene to containerize a simple Flask application.

## The Task

```python
task = """
I have a Python Flask application in `app.py` with `requirements.txt`.
Please help me containerize it:
1. Create a `Dockerfile` that uses python:3.9-slim, installs dependencies, exposes port 5000, and runs the app.
2. Create a `docker-compose.yml` file to run the service, mapping port 5000:5000.
3. Add a `.dockerignore` file to exclude unnecessary files (like __pycache__, .git, venv).
"""
```

## Execution Process

### 1. Planning
The Planner recognizes this as a DevOps task:
1.  **Analyze**: Check `requirements.txt` to ensure dependencies are listed.
2.  **Dockerfile**: Write the build instructions.
3.  **Docker Compose**: Define the service orchestration.
4.  **Optimization**: Create `.dockerignore` to keep the build context small.

### 2. Execution & Reflection
-   **Step 1 (Dockerfile)**: The Executor writes the `Dockerfile`.
    -   *Detail*: It correctly sets `WORKDIR /app`, copies `requirements.txt` first (for layer caching), runs `pip install`, and then copies the rest of the code.
-   **Step 2 (Compose)**: It creates `docker-compose.yml` with version '3' syntax, defining the service `web` and port mapping `5000:5000`.
-   **Step 3 (Ignore)**: It adds `.dockerignore` to exclude `__pycache__` and `.git`.

## The Result

**Dockerfile**:
```dockerfile
FROM python:3.9-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

EXPOSE 5000

CMD ["python", "app.py"]
```

**docker-compose.yml**:
```yaml
version: '3.8'
services:
  web:
    build: .
    ports:
      - "5000:5000"
    volumes:
      - .:/app
```

## Key Takeaway
Zene isn't just for application code. It understands infrastructure configuration, best practices (like layer caching), and can set up your deployment pipeline in seconds.
