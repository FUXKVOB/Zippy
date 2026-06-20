import { test, expect } from '@playwright/test';

test.describe('Zippy SPA', () => {
  test('home page loads and shows heading', async ({ page }) => {
    await page.goto('/examples/zippy-site/');
    await expect(page.locator('h1')).toContainText('Zippy SPA', { timeout: 10_000 });
  });

  test('no runtime errors on page load', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (err) => errors.push(err.message));
    
    await page.goto('/examples/zippy-site/');
    await page.waitForTimeout(2000);
    
    expect(errors).toEqual([]);
  });

  test('page has no 404s for assets', async ({ page }) => {
    const failed: string[] = [];
    page.on('response', (resp) => {
      if (resp.status() === 404) failed.push(resp.url());
    });
    
    await page.goto('/examples/zippy-site/');
    await page.waitForTimeout(2000);
    
    expect(failed).toEqual([]);
  });

  test('home page has navigation links', async ({ page }) => {
    await page.goto('/examples/zippy-site/');
    await page.waitForTimeout(1000);
    
    const counterLink = page.locator('a[href="/counter"]');
    await expect(counterLink).toHaveCount(1);
  });
});
