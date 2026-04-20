import { test, expect } from '@playwright/test';

test.describe('Group Tests', () => {
  test('should have groups tab visible', async ({ page }) => {
    const testUsername = `groupuser_${Date.now()}`;
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

    await expect(page.locator('#tab-groups')).toBeVisible();
  });

  test('should have groups tab enabled', async ({ page }) => {
    const testUsername = `groupuser_${Date.now()}`;
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

    await expect(page.locator('#tab-groups')).toBeEnabled();
  });

  test('should have plus button for creating new items', async ({ page }) => {
    const testUsername = `groupuser_${Date.now()}`;
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

    await expect(page.locator('#sidebar button.bg-purple-600')).toBeAttached();
  });
});