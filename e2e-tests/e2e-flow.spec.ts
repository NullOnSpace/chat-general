import { test, expect } from '@playwright/test';

test.describe('E2E User Flow Tests', () => {
  test('should complete full user registration and login flow', async ({ page }) => {
    const testUsername = `e2euser_${Date.now()}`;
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

    const user = await page.evaluate(() => localStorage.getItem('user'));
    expect(user).not.toBeNull();
    const userData = JSON.parse(user!);
    expect(userData.username).toBe(testUsername);
  });

  test('should navigate to friends page from chat', async ({ page }) => {
    const testUsername = `e2euser_${Date.now()}`;
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

    await page.evaluate(() => {
      const link = document.querySelector('nav a[href="/friends.html"]') as HTMLAnchorElement;
      if (link) link.click();
    });
    await expect(page).toHaveURL(/friends\.html/, { timeout: 10000 });
  });

  test('should have navigation link to chat on friends page', async ({ page }) => {
    const testUsername = `e2euser_${Date.now()}`;
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

    await page.evaluate(() => {
      const link = document.querySelector('nav a[href="/friends.html"]') as HTMLAnchorElement;
      if (link) link.click();
    });
    await expect(page).toHaveURL(/friends\.html/, { timeout: 10000 });

    const chatLink = await page.locator('nav a[href="/chat.html"]');
    await expect(chatLink).toBeAttached();
  });

  test('should persist login state across page reload', async ({ page }) => {
    const testUsername = `e2euser_${Date.now()}`;
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

    await page.reload();
    await page.waitForTimeout(1000);

    const token = await page.evaluate(() => localStorage.getItem('access_token'));
    expect(token).not.toBeNull();
  });

  test('should handle form validation correctly', async ({ page }) => {
    await page.goto('/');

    await page.getByRole('button', { name: 'Sign In' }).click();
    await expect(page.locator('#error-message')).toBeVisible();
    await expect(page.locator('#error-message')).toContainText('Please fill in all fields');

    await page.getByRole('link', { name: 'Sign up' }).click();
    await page.getByRole('button', { name: 'Create Account' }).click();
    await expect(page.locator('#error-message')).toBeVisible();

    await page.locator('#reg-username').fill('test');
    await page.locator('#reg-email').fill('test@test.com');
    await page.locator('#reg-password').fill('12345');
    await page.locator('#reg-confirm-password').fill('12345');
    await page.getByRole('button', { name: 'Create Account' }).click();
    await expect(page.locator('#error-message')).toContainText('Password must be at least 6 characters');

    await page.locator('#reg-password').fill('password123');
    await page.locator('#reg-confirm-password').fill('password456');
    await page.getByRole('button', { name: 'Create Account' }).click();
    await expect(page.locator('#error-message')).toContainText('Passwords do not match');
  });

  test('should display correct UI elements on chat page', async ({ page }) => {
    const testUsername = `e2euser_${Date.now()}`;
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

    await expect(page.locator('#sidebar')).toBeVisible();
    await expect(page.locator('#chat-area')).toBeVisible();
    await expect(page.locator('#tab-chats')).toBeVisible();
    await expect(page.locator('#tab-groups')).toBeVisible();
    await expect(page.locator('#search-input')).toBeVisible();
    await expect(page.locator('#no-chat-selected')).toBeVisible();
  });

  test('should display correct UI elements on friends page', async ({ page }) => {
    const testUsername = `e2euser_${Date.now()}`;
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

    await page.evaluate(() => {
      const link = document.querySelector('nav a[href="/friends.html"]') as HTMLAnchorElement;
      if (link) link.click();
    });
    await expect(page).toHaveURL(/friends\.html/, { timeout: 10000 });

    await expect(page.getByRole('heading', { name: 'Friends' })).toBeVisible();
    await expect(page.locator('#tab-friends')).toBeVisible();
    await expect(page.locator('#tab-requests')).toBeVisible();
    await expect(page.locator('#tab-sent')).toBeVisible();
    await expect(page.locator('#friends-list')).toBeAttached();
    await expect(page.locator('#requests-list')).toBeAttached();
    await expect(page.locator('#sent-list')).toBeAttached();
  });
});