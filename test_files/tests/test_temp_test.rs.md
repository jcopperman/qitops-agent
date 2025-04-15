 # Test Cases for `add` function in Rust

## Normal Functionality

### Basic Addition

- **Description**: Check if the function correctly adds two integers.
- **Inputs**: (5, 7)
- **Expected Outputs**: 12

### Large Number Addition

- **Description**: Check if the function correctly handles large numbers.
- **Inputs**: (9007199254740991, 9007199254740991)
- **Expected Outputs**: 18014398509323982

## Edge Cases

### Minimum i32 Value Addition

- **Description**: Check if the function correctly adds the minimum value of i32 (`i32::MIN`) with a positive number.
- **Inputs**: ((i32::MIN - 1), 1)
- **Expected Outputs**: `i32::MIN + 1`

### Maximum i32 Value Addition

- **Description**: Check if the function correctly adds the maximum value of i32 (`i32::MAX`) with a positive number.
- **Inputs**: ((i32::MAX), 1)
- **Expected Outputs**: `i32::MAX + 1`

### Zero Addition

- **Description**: Check if the function correctly adds zero to any number.
- **Inputs**: (0, 5), (5, 0), (5, 0)
- **Expected Outputs**: 5, 5, 5

## Error Handling

### Negative Number Addition

- **Description**: Check if the function correctly handles adding negative numbers.
- **Inputs**: (-5, 7), (5, -7)
- **Expected Outputs**: 2, -12

### Overflow with Large Numbers

- **Description**: Check if the function throws an error when trying to add two numbers that exceed the maximum value of i32.
- **Inputs**: ((i32::MAX) + 1, (i32::MAX) + 1)
- **Expected Outputs**: Function should throw an overflow error

### Underflow with Small Numbers

- **Description**: Check if the function throws an error when trying to subtract two numbers that exceed the minimum value of i32.
- **Inputs**: ((i32::MIN) - 1, (i32::MIN) + 1)
- **Expected Outputs**: Function should throw an underflow error