/* content-script.js — Detects login forms and handles autofill */

/**
 * Listen for autofill messages from the popup / background script.
 */
chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message.type === 'autofill') {
    fillCredentials(message.username, message.password);
    sendResponse({ success: true });
  }
});

/**
 * Find and fill username + password fields on the page.
 */
function fillCredentials(username, password) {
  const passwordFields = findPasswordFields();
  const usernameFields = findUsernameFields();

  // Fill the first username field found
  if (usernameFields.length > 0 && username) {
    setFieldValue(usernameFields[0], username);
  }

  // Fill the first password field found
  if (passwordFields.length > 0 && password) {
    setFieldValue(passwordFields[0], password);
  }
}

/**
 * Set a field's value and dispatch events so the page recognizes the change.
 */
function setFieldValue(field, value) {
  // Focus the field
  field.focus();

  // Use native input setter to bypass React/Angular value trapping
  const nativeInputValueSetter = Object.getOwnPropertyDescriptor(
    HTMLInputElement.prototype,
    'value'
  ).set;
  nativeInputValueSetter.call(field, value);

  // Dispatch events that frameworks listen to
  field.dispatchEvent(new Event('input', { bubbles: true }));
  field.dispatchEvent(new Event('change', { bubbles: true }));
}

/**
 * Find password input fields on the page.
 */
function findPasswordFields() {
  return Array.from(document.querySelectorAll('input[type="password"]')).filter(
    (el) => isVisible(el)
  );
}

/**
 * Find username/email input fields near password fields.
 */
function findUsernameFields() {
  // Common selectors for username fields
  const selectors = [
    'input[type="email"]',
    'input[name="email"]',
    'input[name="username"]',
    'input[name="user"]',
    'input[name="login"]',
    'input[autocomplete="username"]',
    'input[autocomplete="email"]',
    'input[id*="user" i]',
    'input[id*="email" i]',
    'input[id*="login" i]',
  ];

  const found = new Set();
  for (const selector of selectors) {
    document.querySelectorAll(selector).forEach((el) => {
      if (isVisible(el) && el.type !== 'hidden') {
        found.add(el);
      }
    });
  }

  // Fallback: find text inputs near a password field
  if (found.size === 0) {
    const passwordField = document.querySelector('input[type="password"]');
    if (passwordField) {
      const form = passwordField.closest('form');
      if (form) {
        form.querySelectorAll('input[type="text"], input:not([type])').forEach(
          (el) => {
            if (isVisible(el)) found.add(el);
          }
        );
      }
    }
  }

  return Array.from(found);
}

/**
 * Check if an element is visible on the page.
 */
function isVisible(el) {
  const style = window.getComputedStyle(el);
  return (
    style.display !== 'none' &&
    style.visibility !== 'hidden' &&
    el.offsetWidth > 0 &&
    el.offsetHeight > 0
  );
}
