// リポジトリ直下の skills/<name>/SKILL.md を、ビルド時にそのまま読み込む。
// サイト側にコピーは持たない（単一の情報源は skills/ 配下）。
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';

// ビルドは website/ をカレントディレクトリにして実行する（npm run build / withastro/action の path）。
const SKILLS_DIR = path.resolve(process.cwd(), '..', 'skills');

/** /skill.md で配信するスキル。 */
export const PRIMARY_SKILL = 'cwsweep';

export interface Skill {
	name: string;
	description: string;
	/** SKILL.md の生バイト。digest はこのバイト列に対して計算する。 */
	bytes: Buffer;
	/** `sha256:<hex>` */
	digest: string;
}

/** frontmatter の単一行 `key: value` を取り出す。複数行の YAML は扱わず、見つからなければビルドを失敗させる。 */
function frontmatterValue(markdown: string, key: string, file: string): string {
	const match = markdown.match(/^---\r?\n([\s\S]*?)\r?\n---(?:\r?\n|$)/);
	if (!match) throw new Error(`${file}: frontmatter (--- ... ---) not found`);
	const line = match[1].split(/\r?\n/).find((l) => l.startsWith(`${key}: `));
	if (!line) throw new Error(`${file}: single-line "${key}: ..." not found in frontmatter`);
	return line.slice(key.length + 2).trim();
}

export function loadSkills(): Skill[] {
	if (!existsSync(SKILLS_DIR)) {
		throw new Error(
			`skills directory not found: ${SKILLS_DIR} (run the build from website/, with skills/ at the repository root)`,
		);
	}
	const skills = readdirSync(SKILLS_DIR, { withFileTypes: true })
		.filter((entry) => entry.isDirectory() && existsSync(path.join(SKILLS_DIR, entry.name, 'SKILL.md')))
		.map((entry) => {
			const file = path.join(SKILLS_DIR, entry.name, 'SKILL.md');
			const bytes = readFileSync(file);
			const text = bytes.toString('utf8');
			const name = frontmatterValue(text, 'name', file);
			if (name !== entry.name) {
				throw new Error(`${file}: frontmatter name "${name}" must match the directory name "${entry.name}"`);
			}
			return {
				name,
				description: frontmatterValue(text, 'description', file),
				bytes,
				digest: `sha256:${createHash('sha256').update(bytes).digest('hex')}`,
			};
		})
		.sort((a, b) => a.name.localeCompare(b.name));
	if (skills.length === 0) throw new Error(`no skills found under ${SKILLS_DIR}`);
	return skills;
}

export function primarySkill(): Skill {
	const skill = loadSkills().find((s) => s.name === PRIMARY_SKILL);
	if (!skill) throw new Error(`skill "${PRIMARY_SKILL}" not found under ${SKILLS_DIR}`);
	return skill;
}

export function skillUrlPath(name: string): string {
	return `/.well-known/agent-skills/${name}/SKILL.md`;
}
