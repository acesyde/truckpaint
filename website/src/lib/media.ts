import { existsSync } from 'node:fs';
import { join } from 'node:path';

/** The page's language: its locale, English for the default locale. */
export function pageLanguage(astro: { currentLocale?: string | undefined }): string {
	return astro.currentLocale ?? 'en';
}

/**
 * The public file of a medium for a language, checked to exist: a missing
 * medium fails the build, naming the page and the file.
 */
export function mediaFile(language: string, file: string, page: string): { path: string; url: string } {
	const path = join(process.cwd(), 'public', 'media', language, file);
	if (!existsSync(path)) {
		throw new Error(
			`Missing documentation medium "media/${language}/${file}" on page ${page}: ` +
				'render it with `mise run doc:media <section>`.'
		);
	}
	return { path, url: `${import.meta.env.BASE_URL}media/${language}/${file}` };
}

const CLOSE_LABELS: Record<string, string> = {
	en: 'Close',
	fr: 'Fermer',
	es: 'Cerrar',
	de: 'Schließen',
};

/** The label of the screenshot preview's close button, English for an unknown language. */
export function closeLabel(language: string): string {
	return CLOSE_LABELS[language] ?? CLOSE_LABELS.en!;
}
