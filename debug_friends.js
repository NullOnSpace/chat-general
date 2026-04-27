const { chromium } = require('playwright');
(async () => {
  const browser = await chromium.launch();
  
  const ctx1 = await browser.newContext();
  const page1 = await ctx1.newPage();
  const ctx2 = await browser.newContext();
  const page2 = await ctx2.newPage();
  
  // User 1 register and login
  await page1.goto('http://localhost:12315/');
  await page1.getByRole('link', { name: 'Sign up' }).click();
  const uid1 = Date.now();
  await page1.locator('#reg-username').fill('friend1_' + uid1);
  await page1.locator('#reg-email').fill('f1_' + uid1 + '@test.com');
  await page1.locator('#reg-password').fill('password123');
  await page1.locator('#reg-confirm-password').fill('password123');
  await page1.getByRole('button', { name: 'Create Account' }).click();
  await page1.locator('#login-form').waitFor();
  await page1.locator('#username').fill('friend1_' + uid1);
  await page1.locator('#password').fill('password123');
  await page1.getByRole('button', { name: 'Sign In' }).click();
  await page1.waitForURL(/chat/);
  const userId1 = await page1.evaluate(() => JSON.parse(localStorage.getItem('user')).id);
  console.log('User1 ID:', userId1);
  
  // User 2 register and login
  await page2.goto('http://localhost:12315/');
  await page2.getByRole('link', { name: 'Sign up' }).click();
  const uid2 = Date.now();
  await page2.locator('#reg-username').fill('friend2_' + uid2);
  await page2.locator('#reg-email').fill('f2_' + uid2 + '@test.com');
  await page2.locator('#reg-password').fill('password123');
  await page2.locator('#reg-confirm-password').fill('password123');
  await page2.getByRole('button', { name: 'Create Account' }).click();
  await page2.locator('#login-form').waitFor();
  await page2.locator('#username').fill('friend2_' + uid2);
  await page2.locator('#password').fill('password123');
  await page2.getByRole('button', { name: 'Sign In' }).click();
  await page2.waitForURL(/chat/);
  const userId2 = await page2.evaluate(() => JSON.parse(localStorage.getItem('user')).id);
  console.log('User2 ID:', userId2);
  
  // User 2 sends friend request to User 1
  const sendResult = await page2.evaluate(async (targetUid) => {
    const token = localStorage.getItem('access_token');
    if (!token) return { error: 'no token' };
    const res = await fetch('/api/v1/friends/requests', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${token}` },
      body: JSON.stringify({ to_user_id: targetUid, message: null }),
    });
    const data = await res.json();
    return { status: res.status, ok: res.ok, data: data };
  }, userId1);
  console.log('Send request result:', JSON.stringify(sendResult));
  
  // User 2 navigates to friends page
  await page2.locator('nav a[href="/friends.html"]').click();
  await page2.waitForURL(/friends\.html/);
  await page2.waitForTimeout(1000);
  
  // Switch to sent tab
  await page2.locator('#tab-sent').click();
  await page2.waitForTimeout(500);
  
  // Check sent list content
  const sentListContent = await page2.locator('#sent-list').textContent();
  console.log('Sent list content:', sentListContent);
  
  const hasPending = await page2.getByText(/pending/i).count();
  console.log('Pending count:', hasPending);
  
  await browser.close();
})();
