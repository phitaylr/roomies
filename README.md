# 🏨 Roomies Program Documentation

A program designed to optimize grouping people, originally for assigning hotel rooms with occupants who know and tolerate each other.

---

## 🛠️ Setup and Data Preparation

### 1. Gather Data

The most efficient way to collect the necessary data is by using an online form.

* Start with this Google Form [template](https://docs.google.com/forms/d/10zUon41iw6LAqGx24GNpuxVu8uSNLtY33j2wYKphAW4/copy).
* **Populate Names:** Paste the names of all people who need to be grouped into both the "What is your name?" and the "Choose" sections.
    > **Tip:** To simplify data processing, **do not** use commas (`,`) within the names themselves.
* **Customize Categories:** If necessary (e.g., separating by gender), adapt the "boys/girls" section as needed. This column serves as your grouping **Category**.

### 2. Download and Format the Spreadsheet

After everyone has completed the form, download the results as an **Excel spreadsheet**.

The key step is transforming the list of chosen roommates (which are often combined into one cell, separated by commas) into individual columns.

#### Converting Data with "Text to Columns"

1.  Open the downloaded spreadsheet in Excel.
2.  **Select the Column:** Click the header (letter) of the column containing the comma-separated names of choices/preferences (e.g., "Choose 1 - 6 people you would be happiest to room with").
3.  Go to the **Data** tab on the Excel ribbon.
4.  Click **Text to Columns** (found in the Data Tools group).
5.  In the wizard:
    * **Step 1:** Choose the **Delimited** option and click **Next**.
    * **Step 2:** Under **Delimiters**, check the box for **Comma**. You should see the data in the preview window separate into distinct columns. Uncheck any other delimiters (like Tab). Click **Next**.
    * **Step 3:** For **Column data format**, ensure **General** is selected. Click **Finish**.

#### Standardizing Column Headers

After separation, rename the column headers to the following standard format:

* **Name** (The person's full name)
* **Category** (e.g., Boy, Girl, Cabin, ADA, etc.)
* **Choice1, Choice2, Choice3...** (The person's preferred roommates)
* **Avoid1, Avoid2...** (Optional: People to avoid rooming with)

> **Manual Step:** Manually add any **_Avoids_** you need to ensure two specific people are **not** grouped together. Simply enter the name of the person to avoid in the corresponding cell under an **Avoid** column.

### 3. Save the File

Save your newly formatted spreadsheet (using a standard format like **.xlsx**).

---

## 🚀 Running the Roomies Program

1.  **Open the Roomies program.**
2.  **Choose your file** (the spreadsheet you just saved).
3.  **Configure Settings:**
    * **Edit the Event Name** (e.g., "Annual Conference 2026").
    * **Max People per Group:** Set the maximum number of people allowed in a single group (e.g., 4 for a standard hotel room). The program automatically seeks to minimize the total number of groups and balance them as much as possible.
    * **Number of Iterations:** This is the number of different grouping combinations the program will attempt to find the best fit.
        * The more iterations you choose, the longer the process will take, but the more likely you are to find an optimal combination that maximizes happy choices and minimizes conflicts.
        * Even a large number like **1,000,000** iterations processes quickly. If you have a large dataset, increase this number until the results meet your satisfaction.

4.  **Download Results:** Once the program finishes, select **Download** to receive a clear PDF file of the final group assignments.
