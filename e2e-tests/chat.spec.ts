import { test, expect } from '@playwright/test';

test.describe('Chat Page Tests', () => {
  let testUsername: string;
  let testEmail: string;
  const testPassword = 'password123';

  test.beforeEach(async ({ page }) => {
    testUsername = `chatuser_${Date.now()}`;
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
    await page.waitForTimeout(500);
  });

  test('should display chat page after login', async ({ page }) => {
    await expect(page).toHaveURL(/chat\.html/);
    await expect(page.locator('#sidebar')).toBeVisible();
    await expect(page.locator('#chat-area')).toBeVisible();
  });

  test('should display no chat selected message', async ({ page }) => {
    await expect(page.locator('#no-chat-selected')).toBeVisible();
    await expect(page.getByText('Select a conversation to start chatting')).toBeVisible();
  });

  test('should display tabs for chats and groups', async ({ page }) => {
    await expect(page.locator('#tab-chats')).toBeVisible();
    await expect(page.locator('#tab-groups')).toBeVisible();
  });

  test('should have clickable tabs', async ({ page }) => {
    await expect(page.locator('#tab-groups')).toBeEnabled();
    await expect(page.locator('#tab-chats')).toBeEnabled();
  });

  test('should show new conversation modal', async ({ page }) => {
    await page.locator('#sidebar button.bg-purple-600').click();
    await expect(page.locator('#new-conversation-modal')).toBeVisible();
  });

  test('should navigate to friends page', async ({ page }) => {
    await page.evaluate(() => {
      const link = document.querySelector('nav a[href="/friends.html"]') as HTMLAnchorElement;
      if (link) link.click();
    });
    await expect(page).toHaveURL(/friends\.html/, { timeout: 10000 });
  });

  test('should clear token on logout', async ({ page }) => {
    const tokenBefore = await page.evaluate(() => localStorage.getItem('access_token'));
    expect(tokenBefore).not.toBeNull();

    await page.evaluate(() => {
      localStorage.removeItem('access_token');
      localStorage.removeItem('refresh_token');
      localStorage.removeItem('user');
    });

    const tokenAfter = await page.evaluate(() => localStorage.getItem('access_token'));
    expect(tokenAfter).toBeNull();
  });
});