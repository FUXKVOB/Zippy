# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: zippy.spec.ts >> Zippy SPA >> navigation between routes works
- Location: tests\e2e\zippy.spec.ts:22:3

# Error details

```
Test timeout of 30000ms exceeded.
```

```
Error: page.click: Test timeout of 30000ms exceeded.
Call log:
  - waiting for locator('a[href="/counter"]')

```

# Page snapshot

```yaml
- generic [ref=e3]:
  - heading "Zippy SPA" [level=1] [ref=e4]
  - navigation [ref=e5]:
    - link "Home" [ref=e6] [cursor=pointer]:
      - /url: "#/"
    - link "Counter" [ref=e7] [cursor=pointer]:
      - /url: "#/counter"
    - link "Todo" [ref=e8] [cursor=pointer]:
      - /url: "#/todo"
```

# Test source

```ts
  1  | import { test, expect } from '@playwright/test';
  2  | 
  3  | test.describe('Zippy SPA', () => {
  4  |   test('home page loads and shows heading', async ({ page }) => {
  5  |     await page.goto('/examples/zippy-site/');
  6  |     await expect(page.locator('h1')).toContainText('Zippy SPA');
  7  |   });
  8  | 
  9  |   test('counter increments when button is clicked', async ({ page }) => {
  10 |     await page.goto('/examples/zippy-site/counter');
  11 |     const button = page.locator('button').first();
  12 |     const text = page.locator('p, span').first();
  13 |     
  14 |     const before = await text.textContent();
  15 |     await button.click();
  16 |     await page.waitForTimeout(100);
  17 |     const after = await text.textContent();
  18 |     
  19 |     expect(after).not.toBe(before);
  20 |   });
  21 | 
  22 |   test('navigation between routes works', async ({ page }) => {
  23 |     await page.goto('/examples/zippy-site/');
  24 |     
> 25 |     await page.click('a[href="/counter"]');
     |                ^ Error: page.click: Test timeout of 30000ms exceeded.
  26 |     await expect(page).toHaveURL(/\/counter/);
  27 |     
  28 |     await page.click('a[href="/todo"]');
  29 |     await expect(page).toHaveURL(/\/todo/);
  30 |     
  31 |     await page.click('a[href="/"]');
  32 |     await expect(page).toHaveURL(/\/$|\/index/);
  33 |   });
  34 | 
  35 |   test('await block shows loading then content', async ({ page }) => {
  36 |     await page.goto('/examples/zippy-site/');
  37 |     
  38 |     const reloadBtn = page.locator('button', { hasText: /Reload/ }).first();
  39 |     if (await reloadBtn.count() > 0) {
  40 |       await reloadBtn.click();
  41 |       await expect(page.locator('p, span').filter({ hasText: /Loading/ }).first()).toBeVisible({ timeout: 1000 });
  42 |     }
  43 |   });
  44 | 
  45 |   test('no runtime errors on page load', async ({ page }) => {
  46 |     const errors: string[] = [];
  47 |     page.on('pageerror', (err) => errors.push(err.message));
  48 |     
  49 |     await page.goto('/examples/zippy-site/');
  50 |     await page.waitForTimeout(500);
  51 |     
  52 |     expect(errors).toEqual([]);
  53 |   });
  54 | });
  55 | 
```