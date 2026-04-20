import { test, expect } from '@playwright/test';

test.describe('Register Page Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.getByRole('link', { name: 'Sign up' }).click();
  });

  test('should display register form', async ({ page }) => {
    await expect(page.locator('#register-form')).toBeVisible();
    await expect(page.locator('#reg-username')).toBeVisible();
    await expect(page.locator('#reg-email')).toBeVisible();
    await expect(page.locator('#reg-password')).toBeVisible();
    await expect(page.locator('#reg-confirm-password')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Create Account' })).toBeVisible();
  });

  test('should show error when fields are empty', async ({ page }) => {
    await page.getByRole('button', { name: 'Create Account' }).click();
    await expect(page.locator('#error-message')).toBeVisible();
    await expect(page.locator('#error-message')).toContainText('Please fill in all fields');
  });

  test('should show error when passwords do not match', async ({ page }) => {
    await page.locator('#reg-username').fill('testuser');
    await page.locator('#reg-email').fill('test@test.com');
    await page.locator('#reg-password').fill('password123');
    await page.locator('#reg-confirm-password').fill('password456');
    await page.getByRole('button', { name: 'Create Account' }).click();
    await expect(page.locator('#error-message')).toBeVisible();
    await expect(page.locator('#error-message')).toContainText('Passwords do not match');
  });

  test('should show error when password is too short', async ({ page }) => {
    await page.locator('#reg-username').fill('testuser');
    await page.locator('#reg-email').fill('test@test.com');
    await page.locator('#reg-password').fill('12345');
    await page.locator('#reg-confirm-password').fill('12345');
    await page.getByRole('button', { name: 'Create Account' }).click();
    await expect(page.locator('#error-message')).toBeVisible();
    await expect(page.locator('#error-message')).toContainText('Password must be at least 6 characters');
  });

  test('should register successfully', async ({ page }) => {
    const testUsername = `newuser_${Date.now()}`;
    const testEmail = `${testUsername}@test.com`;

    await page.locator('#reg-username').fill(testUsername);
    await page.locator('#reg-email').fill(testEmail);
    await page.locator('#reg-password').fill('password123');
    await page.locator('#reg-confirm-password').fill('password123');
    await page.getByRole('button', { name: 'Create Account' }).click();

    await page.waitForTimeout(1000);
    await expect(page.locator('#login-form')).toBeVisible();
  });

  test('should switch back to login form', async ({ page }) => {
    await page.getByRole('link', { name: 'Sign in' }).click();
    await expect(page.locator('#register-form')).toBeHidden();
    await expect(page.locator('#login-form')).toBeVisible();
  });
});