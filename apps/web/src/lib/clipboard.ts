/** Copy text; falls back to a hidden textarea where the async Clipboard API is unavailable (http, old browsers). */
export async function copy(text: string): Promise<boolean> {
	try {
		await navigator.clipboard.writeText(text);
		return true;
	} catch {
		const ta = Object.assign(document.createElement('textarea'), { value: text });
		document.body.append(ta);
		ta.select();
		const ok = document.execCommand('copy');
		ta.remove();
		return ok;
	}
}
