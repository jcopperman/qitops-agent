/**
 * Email validation function
 * @param {string} email - The email to validate
 * @returns {boolean} - True if email is valid, false otherwise
 */
function validateEmail(email) {
  const re = /\S+@\S+\.\S+/;
  return re.test(email);
}

/**
 * Password validation function
 * @param {string} password - The password to validate
 * @returns {boolean} - True if password is valid, false otherwise
 */
function validatePassword(password) {
  // Password must be at least 8 characters
  if (password.length < 8) {
    return false;
  }
  
  // Password must contain at least one uppercase letter
  if (!/[A-Z]/.test(password)) {
    return false;
  }
  
  // Password must contain at least one lowercase letter
  if (!/[a-z]/.test(password)) {
    return false;
  }
  
  // Password must contain at least one number
  if (!/[0-9]/.test(password)) {
    return false;
  }
  
  return true;
}

/**
 * Username validation function
 * @param {string} username - The username to validate
 * @returns {boolean} - True if username is valid, false otherwise
 */
function validateUsername(username) {
  // Username must be between 3 and 20 characters
  if (username.length < 3 || username.length > 20) {
    return false;
  }
  
  // Username must start with a letter
  if (!/^[a-zA-Z]/.test(username)) {
    return false;
  }
  
  // Username can only contain letters, numbers, and underscores
  if (!/^[a-zA-Z0-9_]+$/.test(username)) {
    return false;
  }
  
  return true;
}

module.exports = {
  validateEmail,
  validatePassword,
  validateUsername
};
