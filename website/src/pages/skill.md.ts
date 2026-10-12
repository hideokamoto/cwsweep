import type { APIRoute } from 'astro';
import { primarySkill } from '../lib/skills';

// /skill.md: skills/cwsweep/SKILL.md をそのままのバイト列で配信する。
export const GET: APIRoute = () =>
	new Response(new Uint8Array(primarySkill().bytes), {
		headers: { 'Content-Type': 'text/markdown; charset=utf-8' },
	});
