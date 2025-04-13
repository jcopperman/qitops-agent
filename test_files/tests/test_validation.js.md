# Test Cases for Email Validation Function

## Basic Scenarios

```markdown
- **Test Case 1:**
    * Input: `test@example.com`
    * Expected Output: `true`

- **Test Case 2:**
    * Input: `invalid_email`
    * Expected Output: `false`

- **Test Case 3:**
    * Input: `test.com`
    * Expected Output: `false`

- **Test Case 4:**
    * Input: `test@example`
    * Expected Output: `false`
```

## Edge Cases and Error Handling

```markdown
- **Test Case 5:**
    * Input: `" "` (empty string)
    * Expected Output: `false`

- **Test Case 6:**
    * Input: `test@.com`
    * Expected Output: `false`

- **Test Case 7:**
    * Input: `test@example.com` (wrong TLD)
    * Expected Output: `false`

- **Test Case 8:**
    * Input: `test@example..com` (multiple dots)
    * Expected Output: `false`
```

# Test Cases for Password Validation Function

## Basic Scenarios

```markdown
- **Test Case 1:**
    * Input: `Password1234` (8 characters, uppercase, lowercase, number)
    * Expected Output: `true`

- **Test Case 2:**
    * Input: `passw0rd!` (short, with special character)
    * Expected Output: `false`
```

## Edge Cases and Error Handling

```markdown
- **Test Case 3:**
    * Input: `passw0rd!12345678901` (too long)
    * Expected Output: `false`

- **Test Case 4:**
    * Input: `Password12345` (short and no number)
    * Expected Output: `false`

- **Test Case 5:**
    * Input: `PASSWORD!1234` (no lowercase)
    * Expected Output: `false`
```

# Test Cases for Username Validation Function

## Basic Scenarios

```markdown
- **Test Case 1:**
    * Input: `testUsername`
    * Expected Output: `true`

- **Test Case 2:**
    * Input: `user_NAME` (with underscore)
    * Expected Output: `true`

- **Test Case 3:**
    * Input: `1234567890123` (too long)
    * Expected Output: `false`
```

## Edge Cases and Error Handling

```markdown
- **Test Case 4:**
    * Input: `` (empty string)
    * Expected Output: `false`

- **Test Case 5:**
    * Input: `test_` (no characters after underscore)
    * Expected Output: `false`

- **Test Case 6:**
    * Input: ` testUsername` (leading space)
    * Expected Output: `false`
```

# Additional Test Cases for All Functions

## Empty Input

```markdown
- **Test Case 1:**
    * Input: `""` (empty string for all functions)
    * Expected Output:
        + For email validation: `false`
        + For password validation: `undefined` (since the function returns `true` when conditions are met and throws an error if they are not, an empty string would trigger an error which is not tested here)
        + For username validation: `false`
```