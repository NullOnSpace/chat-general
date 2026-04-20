import { test, expect } from '@playwright/test';

test.describe('Friend Flow Tests', () => {
  test('should show add friend modal with user ID input', async ({ page }) => {
    const testUsername = `frienduser_${Date.now()}`;
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

    await page.locator('nav button.bg-purple-600').click();
    await expect(page.locator('#add-friend-modal')).toBeVisible();
    await expect(page.locator('#add-friend-user-id')).toBeVisible();
    await expect(page.locator('#add-friend-message')).toBeVisible();
  });

  test('should show empty friends list for new user', async ({ page }) => {
    const testUsername = `frienduser_${Date.now()}`;
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

    await expect(page.getByText(/No friends yet/i)).toBeVisible();
  });

  test('should have requests tab available', async ({ page }) => {
    const testUsername = `frienduser_${Date.now()}`;
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

    await expect(page.locator('#tab-requests')).toBeVisible();
    await expect(page.locator('#tab-requests')).toBeEnabled();
  });

  test('should have sent tab available', async ({ page }) => {
    const testUsername = `frienduser_${Date.now()}`;
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

    await expect(page.locator('#tab-sent')).toBeVisible();
    await expect(page.locator('#tab-sent')).toBeEnabled();
  });

  test('should close add friend modal when clicking cancel', async ({ page }) => {
    const testUsername = `frienduser_${Date.now()}`;
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

    await page.locator('nav button.bg-purple-600').click();
    await expect(page.locator('#add-friend-modal')).toBeVisible();
    await page.getByRole('button', { name: 'Cancel' }).click();
    await expect(page.locator('#add-friend-modal')).toBeHidden();
  });

  test('should have request badge element', async ({ page }) => {
    const testUsername = `frienduser_${Date.now()}`;
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

    await expect(page.locator('#request-badge')).toBeAttached();
  });
});