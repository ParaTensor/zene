# Automating Data Analysis with Zene

Data cleaning is 80% of a data scientist's job. Zene can automate this tedious process, from loading messy CSVs to generating insightful visualizations.

In this example, we provide Zene with a "dirty" sales dataset containing missing values, invalid dates, and inconsistent formatting. Zene autonomously cleans the data and produces a summary chart.

## The Task

```python
task = """
I have a CSV file `sales_data.csv` with some messy data.
Please help me:
1. Load the data using pandas.
2. Clean the data: remove rows with missing values or invalid dates.
3. Convert the 'date' column to datetime objects.
4. Calculate the total sales amount per product.
5. Generate a bar chart of total sales per product using matplotlib and save it as `sales_chart.png`.
6. Save the cleaned data to `cleaned_sales_data.csv`.
"""
```

## Execution Process

### 1. Planning
Zene's Planner (DeepSeek) breaks this down into a logical sequence:
1.  **Read & Inspect**: Load `sales_data.csv` and print the first few rows to understand the structure.
2.  **Clean**: Filter out rows where `amount` is null or `date` cannot be parsed.
3.  **Transform**: Convert the `date` column to datetime objects.
4.  **Analyze**: Group by `product` and sum the `amount`.
5.  **Visualize**: Use `matplotlib` to plot the aggregated data.
6.  **Save**: Export the clean DataFrame and the plot image.

### 2. Execution & Reflection
-   **Step 1 (Load)**: The Executor runs a python script to load the CSV. The Reflector checks if the file exists and is readable.
-   **Step 2 (Clean)**: The Executor writes pandas code to drop NA values. It encounters a `ValueError` when parsing "invalid_date".
-   **Self-Healing**: The Reflector catches the `ValueError`. It suggests using `pd.to_datetime(..., errors='coerce')` to handle invalid formats gracefully. The Executor applies this fix automatically.
-   **Step 5 (Visualize)**: Zene generates a bar chart. The Reflector verifies that `sales_chart.png` was created.

### 3. Handling Dependency Hell
What if the user's environment doesn't have `pandas` installed?
-   **Error**: The initial script fails with `ModuleNotFoundError: No module named 'pandas'`.
-   **Reflect**: The Reflector identifies the missing package.
-   **Fix**: It plans a new step: "Install pandas using pip".
-   **Execute**: The Executor runs `pip install pandas`.
-   **Retry**: The script runs successfully on the second attempt.

## The Result

**Input (`sales_data.csv`)**:
```csv
date,product,amount,region
2023-01-01,Widget A,100,North
2023-01-03,Widget A,,North        <-- Missing Amount
invalid_date,Widget B,300,West    <-- Invalid Date
```

**Output (`cleaned_sales_data.csv`)**:
```csv
date,product,amount,region
2023-01-01,Widget A,100.0,North
2023-01-02,Widget B,200.0,South
2023-01-04,Widget C,150.0,East
2023-01-05,Widget A,120.0,North
```

**Visualization**:
A clear bar chart showing total sales by product is generated automatically.

## Key Takeaway
Zene doesn't just write code; it handles runtime data errors (like bad CSV formatting) by iterating on its own code until the pipeline runs successfully.
