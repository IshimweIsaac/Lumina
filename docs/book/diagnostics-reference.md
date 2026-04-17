# Chapter 5: Diagnostics & Error Reference 🚨

Lumina v1.8 introduces a "Teaching" diagnostic system. Instead of cryptic codes, the engine provides human-readable explanations and suggested fixes for every failure.

---

## 5.1 The Physiology of an Error

Every Lumina error contains:
1.  **Code**: A unique identifier (e.g., `L004`, `R002`) for looking up in this reference.
2.  **Message**: A concise explanation of what went wrong.
3.  **Location**: The exact file, line, and column where the error was detected.
4.  **Help**: A suggested fix or educational tip (the "Teaching" component).

---

## 5.2 Analyzer Errors (L-Codes)

These are detected during the **Analysis Phase**, before your code ever runs.

| Code | Name | Description | Suggested Fix |
| :--- | :--- | :--- | :--- |
| **L001** | Unknown Identifier | You're referencing a variable or entity that hasn't been declared. | Check for typos or ensure the entity is imported. |
| **L002** | Type Mismatch | You're trying to perform an operation on incompatible types (e.g., `Number + Text`). | Ensure both operands are of the same expected type. |
| **L004** | Circular Dependency | A derived field depends on itself through a chain of other fields. | Break the cycle by using a stored field or a different logic path. |
| **L011** | Duplicate Function | You've declared two functions with the same name. | Rename one of the functions to be unique. |
| **L015** | Purity Violation | A pure function (`fn`) is trying to access entity state. | Pass the required state as an argument to the function instead. |
| **L035** | Trigger Limit | A `when` trigger has more than 3 `and` conditions. | Split the logic into multiple rules or use a derived Boolean field. |
| **L041** | Impure Derived Field | You're using `now()` inside a derived field (`:=`). | Derived fields must be deterministic. Use a rule to update a stored field with `now()` instead. |
| **L042** | Invalid Accessor | Using something other than `.age` on a Timestamp. | Timestamps only support `.age` which returns a Duration. |

---

## 5.3 Runtime Errors (R-Codes)

These occur while the engine is running and usually trigger a **State Rollback**.

| Code | Name | Description | suggested Fix |
| :--- | :--- | :--- | :--- |
| **R001** | Null Access | Attempting to access a field on an instance that was deleted. | Add a check to ensure the instance exists before accessing it. |
| **R002** | Math Error | Division by zero or invalid numeric operation. | Guard your division with an `if divisor != 0` check. |
| **R003** | Recursion Limit | Rules are triggering each other in an infinite loop. | Ensure rules have mutually exclusive conditions or use a `cooldown`. |
| **R004** | Out of Bounds | Accessing a list index that doesn't exist. | Check the `len(list)` before performing indexing. |
| **R006** | Range Violation | A value assigned to a field violates its `@range` metadata. | Validate the input value before calling `update`. |
| **R009** | Read-Only Violation | Attempting to `update` a derived field (`:=`). | Derived fields are read-only. Update the source stored fields instead. |

---

## 5.4 Parser & Lexer Errors

These occur when your code doesn't follow Lumina's grammar.

*   **Unexpected Token**: "I expected to see keyword 'then' but found an identifier."
*   **Unclosed Brace**: "You started an entity block on line 10 but never closed it."
*   **Invalid Character**: "I don't recognize the character '@' in this context."

---

## 5.5 Summary: Built-in Mentorship
Lumina's diagnostics are designed to make you a better reactive programmer. If you encounter an error not listed here, or if the "Help" message is unclear, please report it on our GitHub.
