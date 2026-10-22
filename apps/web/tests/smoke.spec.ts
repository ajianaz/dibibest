import { expect, test } from '@playwright/test';

test('landing links to the workspace', async ({ page }) => {
	await page.goto('/');
	await page.getByRole('link', { name: /open the workspace/i }).click();
	await expect(page).toHaveURL(/\/app$/);
});

test('text → canvas → layouts → fidelity → copy', async ({ page, context, browserName }) => {
	if (browserName === 'chromium') await context.grantPermissions(['clipboard-read', 'clipboard-write']);
	await page.goto('/app');

	// Text-first: sample DBML parsed by the real core (WASM) and rendered.
	const nodes = page.locator('.svelte-flow__node-table');
	await expect(nodes).toHaveCount(2);
	await expect(page.getByRole('button', { name: /round-trip 100% lossless/ })).toBeVisible();

	// Typing a new table updates the canvas (debounced autosync).
	await page.getByRole('textbox', { name: 'dbml source' }).click();
	await page.keyboard.press('ControlOrMeta+End');
	await page.keyboard.type('\nTable tags {\n  id integer [pk]\n}\n');
	await expect(nodes).toHaveCount(3);

	// Fidelity panel shows the real struct (3 tables kept of 3).
	await page.getByRole('button', { name: /round-trip/ }).click();
	await expect(page.getByRole('complementary', { name: 'Fidelity report' })).toContainText('3 / 3');

	// Canvas-first: generated DBML is read-only and mirrors the same state.
	await page.getByRole('tab', { name: /Canvas-first/ }).click();
	await expect(nodes).toHaveCount(3);
	await expect(page.getByRole('textbox', { name: 'dbml source' })).toContainText('Table tags');
	await page.getByRole('button', { name: '＋ Table' }).click();
	await expect(nodes).toHaveCount(4);
	await expect(page.getByRole('textbox', { name: 'dbml source' })).toContainText('table_4');

	// Explorer: sidebar lists every table; hiding one removes its node.
	await page.getByRole('tab', { name: /Explorer/ }).click();
	await expect(page.getByLabel('Schema explorer').getByRole('checkbox')).toHaveCount(4);
	await page.getByLabel('Schema explorer').getByRole('checkbox').first().uncheck();
	await expect(nodes).toHaveCount(3);

	// Copy-to-clipboard: Export → SQL.
	await page.getByRole('button', { name: /Export/ }).click();
	await page.getByRole('menuitem', { name: 'Copy as SQL' }).click();
	await expect(page.getByRole('button', { name: /Copied sql/ })).toBeVisible();
	if (browserName === 'chromium') {
		expect(await page.evaluate(() => navigator.clipboard.readText())).toContain('CREATE TABLE');
	}
});
