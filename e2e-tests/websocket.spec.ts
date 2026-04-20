import { test, expect } from '@playwright/test';

test.describe('WebSocket Tests', () => {
  test('should have access token after login', async ({ page }) => {
    const testUsername = `wsuser_${Date.now()}`;
    const testEmail = `${testUsername}@test.com`;
    const testPassword = 'password123';

    await page.goto('/');
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

    const token = await page.evaluate(() => localStorage.getItem('access_token'));
    expect(token).not.toBeNull();
  });

  test('should have user data stored after login', async ({ page }) => {
    const testUsername = `wsuser_${Date.now()}`;
    const testEmail = `${testUsername}@test.com`;
    const testPassword = 'password123';

    await page.goto('/');
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

    const user = await page.evaluate(() => localStorage.getItem('user'));
    expect(user).not.toBeNull();
    const userData = JSON.parse(user!);
    expect(userData.username).toBe(testUsername);
  });

  test('should have refresh token stored', async ({ page }) => {
    const testUsername = `wsuser_${Date.now()}`;
    const testEmail = `${testUsername}@test.com`;
    const testPassword = 'password123';

    await page.goto('/');
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

    const refreshToken = await page.evaluate(() => localStorage.getItem('refresh_token'));
    expect(refreshToken).not.toBeNull();
  });

  test('should have message input element on chat page', async ({ page }) => {
    const testUsername = `wsuser_${Date.now()}`;
    const testEmail = `${testUsername}@test.com`;
    const testPassword = 'password123';

    await page.goto('/');
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

    await expect(page.locator('#message-input')).toBeAttached();
  });
});