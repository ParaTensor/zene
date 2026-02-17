# Web Scraping Made Easy with Zene

Extracting data from websites is a common task for data analysts and researchers. Zene can automate this process, writing scripts to fetch, parse, and structure web content.

In this example, we ask Zene to scrape quotes from a public website.

## The Task

```python
task = """
Write a Python script `scrape_quotes.py` to scrape quotes from 'http://quotes.toscrape.com'.

Requirements:
1. Use `requests` to fetch the page content.
2. Use `BeautifulSoup` (bs4) to parse the HTML.
3. Extract the quote text, the author name, and the tags for each quote on the first page.
4. Store the data as a list of dictionaries.
5. Save the result to a JSON file named `quotes.json` with indentation for readability.
6. Print the number of quotes scraped.
"""
```

## Execution Process

### 1. Planning
The Planner recognizes this as a scraping task:
1.  **Analyze**: Visit the website (simulated via requests) to inspect the structure.
2.  **Scrape**: Write `scrape_quotes.py` using `requests` and `bs4`.
3.  **Parse**: Extract `span.text`, `small.author`, and `div.tags`.
4.  **Save**: Write to `quotes.json`.
5.  **Run**: Execute the script and verify the output file.

### 2. Execution & Reflection
-   **Step 1 (Analyze)**: The Executor writes the scraping logic. It correctly identifies the CSS selectors for quotes (`.quote`), text (`.text`), author (`.author`), and tags (`.tag`).
-   **Step 2 (Run)**: It executes the script.
-   **Step 3 (Verify)**: The Reflector checks if `quotes.json` exists and contains valid JSON data.

## The Result

**scrape_quotes.py**:
```python
import requests
from bs4 import BeautifulSoup
import json

url = 'http://quotes.toscrape.com'
response = requests.get(url)
soup = BeautifulSoup(response.text, 'html.parser')

quotes = []
for quote in soup.select('.quote'):
    text = quote.select_one('.text').get_text(strip=True)
    author = quote.select_one('.author').get_text(strip=True)
    tags = [tag.get_text(strip=True) for tag in quote.select('.tag')]
    
    quotes.append({
        'text': text,
        'author': author,
        'tags': tags
    })

with open('quotes.json', 'w') as f:
    json.dump(quotes, f, indent=4)

print(f"Scraped {len(quotes)} quotes.")
```

**quotes.json** (excerpt):
```json
[
    {
        "text": "The world as we have created it is a process of our thinking. It cannot be changed without changing our thinking.",
        "author": "Albert Einstein",
        "tags": [
            "change",
            "deep-thoughts",
            "thinking",
            "world"
        ]
    },
    ...
]
```

## Key Takeaway
Zene can automate web interactions, parsing HTML and structuring unstructured data into clean JSON for further analysis.
