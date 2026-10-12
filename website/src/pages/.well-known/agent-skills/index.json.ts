import type { APIRoute } from 'astro';
import { loadSkills, skillUrlPath } from '../../../lib/skills';

// Agent Skills Discovery の index。形式は https://github.com/cloudflare/agent-skills-discovery-rfc
// (v0.2.0, Draft)。digest は SKILL.md の生バイトの SHA-256。
export const GET: APIRoute = () => {
	const index = {
		$schema: 'https://schemas.agentskills.io/discovery/0.2.0/schema.json',
		skills: loadSkills().map((skill) => ({
			name: skill.name,
			type: 'skill-md',
			description: skill.description,
			url: skillUrlPath(skill.name),
			digest: skill.digest,
		})),
	};
	return new Response(JSON.stringify(index, null, 2) + '\n', {
		headers: { 'Content-Type': 'application/json; charset=utf-8' },
	});
};
