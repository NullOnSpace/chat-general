import { test, expect } from '@playwright/test';

test.describe('Login Page Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should display login form', async ({ page }) => {
    await expect(page.locator('#login-form')).toBeVisible();
    await expect(page.locator('#username')).toBeVisible();
    await expect(page.locator('#password')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Sign In' })).toBeVisible();
  });

  test('should show error when fields are empty', async ({ page }) => {
    await page.getByRole('button', { name: 'Sign In' }).click();
    await expect(page.locator('#error-message')).toBeVisible();
    await expect(page.locator('#error-message')).toContainText('Please fill in all fields');
  });

  test('should stay on login page for invalid credentials', async ({ page }) => {
    await page.locator('#username').fill('nonexistent_user_12345');
    await page.locator('#password').fill('wrong_password_12345');
    await page.getByRole('button', { name: 'Sign In' }).click();
    await page.waitForTimeout(2000);
    await expect(page).toHaveURL(/\//);
  });

  test('should switch to register form', async ({ page }) => {
    await page.getByRole('link', { name: 'Sign up' }).click();
    await expect(page.locator('#login-form')).toBeHidden();
    await expect(page.locator('#register-form')).toBeVisible();
  });

  test('should login successfully with valid credentials', async ({ page }) => {
    const testUsername = `testuser_${Date.now()}`;
    const testEmail = `${testUsername}@test.com`;
    const testPassword = 'password123';

    await page.getByRole('link', { name: 'Sign up' }).click();
    await page.locator('#reg-username').fill(testUsername);
    await page.locator('#reg-email').fill(testEmail);
    await page.locator('#reg-password').fill(testPassword);
    await page.locator('#reg-confirm-password').fill(testPassword);
    await page.getByRole('button', { name: 'Create Account' }).click();

    await page.waitForTimeout(1000);

    await page.locator('#username').fill(testUsername);
    await page.locator('#password').fill(testPassword);
    await page.getByRole('button', { name: 'Sign In' }).click();

    await expect(page).toHaveURL(/chat\.html/, { timeout: 15000 });
  });
});