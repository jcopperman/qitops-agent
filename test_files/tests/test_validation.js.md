 # Email Validation Test Cases

## Correct Emails

1. `test@example.com` - Should pass
2. `john_doe@domain.co.uk` - Should pass
3. `user123456789@subdomain.maindomain.net` - Should pass
4. `user-123_456@example.org` - Should pass

## Incorrect Emails (Edge Cases and Error Handling)

1. `test@example.com` - Should fail (Incorrect TLD)
2. `test@example..com` - Should fail (Multiple dots)
3. `test@.example.com` - Should fail (Missing domain)
4. `testemail@@example.com` - Should fail (Extra @ symbol)
5. `test.email@.example` - Should fail (Missing TLD)
6. `empty@example.com` - Should fail (Empty email address)
7. `test_email#example.com` - Should fail (Invalid character)
8. `test@1example.com` - Should fail (Too few domain parts)
9. `test@example.cctld` - Should fail (Unknown TLD)
10. `test@[123].com` - Should fail (IPv4 address in domain part)

# Password Validation Test Cases

## Correct Passwords

1. `Pas$w0rd1!` - Should pass (Minimum requirements met)
2. `John_Doe#2021` - Should pass (Minimum requirements met)
3. `AaBbCc12345` - Should pass (Minimum requirements met)

## Incorrect Passwords (Edge Cases and Error Handling)

1. `Password` - Should fail (Too short: < 8 characters)
2. `password123` - Should fail (Missing uppercase letter)
3. `pAssw0rd` - Should fail (Missing number)
4. `Password!2` - Should fail (Incorrect special character)
5. `12345678` - Should fail (Too short: < 8 characters)
6. `ABCabc123` - Should fail (Missing number)
7. `Password#2021` - Should fail (Extra special character)
8. ` password` - Should fail (Case sensitivity)
9. `Password!` - Should fail (Too short: < 8 characters)
10. `123456789` - Should fail (Missing uppercase letter or number)

# Username Validation Test Cases

## Correct Usernames

1. `john_doe` - Should pass
2. `user123_456` - Should pass
3. `User_One-Two` - Should pass

## Incorrect Usernames (Edge Cases and Error Handling)

1. `123` - Should fail (Too short: < 3 characters)
2. `username22` - Should fail (Too long: > 20 characters)
3. `$user-name` - Should fail (Invalid character)
4. `__user__` - Should fail (Consecutive underscores)
5. `user-example.com` - Should fail (Invalid character in username)
6. `user-123abc` - Should fail (Uppercase letter after numbers)
7. `User_123!` - Should fail (Invalid character)
8. `user-123_` - Should fail (Trailing underscore)