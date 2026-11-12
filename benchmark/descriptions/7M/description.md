# Task 7M: Multi-Author Document with Figures and Cross-References

Two writers are co-authoring a three-chapter **DOCUMENT**. A **FIGURE_CREATOR** produces visual content that must be inserted into specific chapters. A **FACT_CHECKER** verifies claims in completed chapters. An **EDITOR** ensures overall consistency.

## Agents

### WRITER_A
Writes chapter 1 (introduction and background) and chapter 2 (methodology).

### WRITER_B
Writes chapter 3 (results and discussion).

### FIGURE_CREATOR
Produces figures and charts based on writer requests, inserts them into the appropriate chapter.

### FACT_CHECKER
Verifies factual claims in completed chapters, flags issues for revision.

### EDITOR
Reviews the complete **DOCUMENT** for consistency, requests revisions.

## Shared Resources

- **DOCUMENT**: shared document for collaborative writing
- **FIGURE_STORE**: shared figure repository

## Workflow

1. **WRITER_A** writes chapters 1 and 2; **WRITER_B** writes chapter 3 concurrently.
2. Writers may cross-reference other chapters (e.g., "As shown in Chapter 3..."). Use `read_section` to obtain the other chapter's completed content and version ID before using it. Missing or still-running writes return no content. Reads return snapshots and do not freeze later edits; reread when the current version is needed. Independent chapter preparation has no prescribed order.
3. **FIGURE_CREATOR** creates figures and inserts them into chapters. Writers may still be editing when figures need to be inserted.
4. Writers update the references in **DOCUMENT** to add citations.
5. The **FACT_CHECKER** can only check a chapter after the writer marks it as complete. The **FACT_CHECKER** flags issues or approves claims.
6. The **EDITOR** reviews only after both fact checking and writing are done. The **EDITOR** accepts or requests revisions for consistency.
7. The **EDITOR** combines the final sections into the completed **DOCUMENT**.

## Goal

All writers complete their chapters, figures are inserted, facts are verified, the editor reviews and approves all content, and the final document is assembled.
