import { test, expect } from '@playwright/test';

test.describe('Friends Page Tests', () => {
  let testUsername: string;
  let testEmail: string;
  const testPassword = 'password123';

  test.beforeEach(async ({ page }) => {
    testUsername = `frienduser_${Date.now()}`;
    testEmail = `${testUsername}@test.com`;

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

    await page.evaluate(() => {
      const link = document.querySelector('nav a[href="/friends.html"]') as HTMLAnchorElement;
      if (link) link.click();
    });
    await expect(page).toHaveURL(/friends\.html/, { timeout: 10000 });
    await page.waitForTimeout(500);
  });

  test('should display friends page', async ({ page }) => {
    await expect(page).toHaveURL(/friends\.html/);
    await expect(page.getByRole('heading', { name: 'Friends' })).toBeVisible();
  });

  test('should display tabs for friends, requests, and sent', async ({ page }) => {
    await expect(page.locator('#tab-friends')).toBeVisible();
    await expect(page.locator('#tab-requests')).toBeVisible();
    await expect(page.locator('#tab-sent')).toBeVisible();
  });

  test('should have requests list element', async ({ page }) => {
    await expect(page.locator('#requests-list')).toBeAttached();
  });

  test('should have sent list element', async ({ page }) => {
    await expect(page.locator('#sent-list')).toBeAttached();
  });

  test('should show add friend modal', async ({ page }) => {
    await page.locator('nav button.bg-purple-600').click();
    await expect(page.locator('#add-friend-modal')).toBeVisible();
  });

  test('should close add friend modal', async ({ page }) => {
    await page.locator('nav button.bg-purple-600').click();
    await expect(page.locator('#add-friend-modal')).toBeVisible();
    await page.getByRole('button', { name: 'Cancel' }).click();
    await expect(page.locator('#add-friend-modal')).toBeHidden();
  });

  test('should navigate back to chat page', async ({ page }) => {
    await page.evaluate(() => {
      const link = document.querySelector('nav a[href="/chat.html"]') as HTMLAnchorElement;
      if (link) link.click();
    });
    await expect(page).toHaveURL(/chat\.html/, { timeout: 10000 });
  });

  test('should show friends list element', async ({ page }) => {
    const friendsList = page.locator('#friends-list');
    await expect(friendsList).toBeAttached();
  });
});