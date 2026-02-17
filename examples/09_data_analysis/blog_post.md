# Data Analysis: AI as a Data Scientist

In this scenario, we challenge Zene to act as a data scientist. Given a raw CSV dataset, can it clean the data, calculate meaningful statistics, and visualize the results?

## The Experiment
We provide a messy CSV file with missing values and inconsistent formatting. Zene's task is to:
1. Load and inspect the data.
2. Clean the data (handle NaNs, fix types).
3. Generate summary statistics.
4. Create a visualization (matplotlib/seaborn).

This tests the Executor's ability to handle runtime errors (like `KeyError` or type errors during plotting) and the Reflector's ability to verify that the output files (images, reports) were actually created.
