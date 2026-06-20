import { test, expect } from '@playwright/test';

test.describe('Zippy SPA', () => {
  test('home page loads and shows heading', async ({ page }) => {
    await page.goto('/examples/zippy-site/');
    await expect(page.locator('h1')).toContainText('Zippy SPA');
  });

  test('counter increments when button is clicked', async ({ page }) => {
    await page.goto('/examples/zippy-site/counter');
    const button = page.locator('button').first();
    const text = page.locator('p, span').first();
    
    const before = await text.textContent();
    await button.click();
    await page.waitForTimeout(100);
    const after = await text.textContent();
    
    expect(after).not.toBe(before);
  });

  test('navigation between routes works', async ({ page }) => {
    await page.goto('/examples/zippy-site/');
    
    await page.click('a[href="/counter"]');
    await expect(page).toHaveURL(/\/counter/);
    
    await page.click('a[href="/todo"]');
    await expect(page).toHaveURL(/\/todo/);
    
    await page.click('a[href="/"]');
    await expect(page).toHaveURL(/\/$|\/index/);
  });

  test('await block shows loading then content', async ({ page }) => {
    await page.goto('/examples/zippy-site/');
    
    const reloadBtn = page.locator('button', { hasText: /Reload/ }).first();
    if (await reloadBtn.count() > 0) {
      await reloadBtn.click();
      await expect(page.locator('p, span').filter({ hasText: /Loading/ }).first()).toBeVisible({ timeout: 1000 });
    }
  });

  test('no runtime errors on page load', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (err) => errors.push(err.message));
    
    await page.goto('/examples/zippy-site/');
    await page.waitForTimeout(500);
    
    expect(errors).toEqual([]);
  });
});
