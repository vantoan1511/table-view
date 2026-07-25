import { execSync } from 'child_process';
import { writeFileSync } from 'fs';

export function parseCommitLine(line) {
  if (!line || typeof line !== 'string') return null;

  const trimmed = line.trim();
  if (!trimmed) return null;

  // Match optional git commit hash followed by conventional commit format: type(scope)?: message
  const match = trimmed.match(
    /^(?:[a-f0-9]{7,40}\s+)?(?<type>[a-z]+)(?:\((?<scope>[^)]+)\))?: (?<message>.+)$/i
  );

  if (!match || !match.groups) return null;

  const { type, scope, message } = match.groups;
  const lowerType = type.toLowerCase();

  return {
    type: lowerType,
    scope: scope ? scope.trim() : null,
    message: message.trim()
  };
}

export function categorizeCommits(lines) {
  const features = [];
  const bugFixes = [];
  const improvements = [];

  const ignoredTypes = new Set(['chore', 'ci', 'build', 'test', 'doc', 'docs']);
  const improvementTypes = new Set(['perf', 'refactor', 'style', 'impr', 'enhancement', 'ui']);

  for (const line of lines) {
    // Skip merge commits and skip ci commits
    if (
      /merge (branch|pull request|commit)/i.test(line) ||
      /\[skip ci\]/i.test(line) ||
      /chore: update manifest/i.test(line)
    ) {
      continue;
    }

    const parsed = parseCommitLine(line);
    if (!parsed) continue;

    const { type, scope, message } = parsed;

    if (ignoredTypes.has(type)) {
      continue;
    }

    // Capitalize first letter of message
    const formattedMsg = message.charAt(0).toUpperCase() + message.slice(1);
    const itemStr = scope ? `**${scope}**: ${formattedMsg}` : formattedMsg;

    if (type === 'feat') {
      features.push(itemStr);
    } else if (type === 'fix') {
      bugFixes.push(itemStr);
    } else if (improvementTypes.has(type)) {
      improvements.push(itemStr);
    }
  }

  return { features, bugFixes, improvements };
}

export function formatReleaseNotes({ features, bugFixes, improvements }) {
  const sections = [];

  if (features && features.length > 0) {
    sections.push(`### 🚀 Features\n\n${features.map((item) => `- ${item}`).join('\n')}`);
  }

  if (bugFixes && bugFixes.length > 0) {
    sections.push(`### 🐛 Bug Fixes\n\n${bugFixes.map((item) => `- ${item}`).join('\n')}`);
  }

  if (improvements && improvements.length > 0) {
    sections.push(`### ⚡ Improvements\n\n${improvements.map((item) => `- ${item}`).join('\n')}`);
  }

  if (sections.length === 0) {
    return 'General maintenance, performance, and stability improvements.';
  }

  return sections.join('\n\n');
}

export function getGitCommitLines() {
  try {
    // Find the previous tag
    let range = '';
    try {
      const prevTag = execSync('git describe --tags --abbrev=0 HEAD^ 2>/dev/null', {
        encoding: 'utf8'
      }).trim();
      if (prevTag) {
        range = `${prevTag}..HEAD`;
      }
    } catch {
      // No previous tag found, use full log
      range = 'HEAD';
    }

    const logCmd = range ? `git log ${range} --oneline` : 'git log --oneline';
    const output = execSync(logCmd, { encoding: 'utf8' });
    return output.split('\n').filter(Boolean);
  } catch (err) {
    console.warn('Failed to retrieve git log:', err.message);
    return [];
  }
}

export function generateReleaseNotes(outputPath = 'release_notes.md', customLines = null) {
  const lines = customLines || getGitCommitLines();
  const categorized = categorizeCommits(lines);
  const markdown = formatReleaseNotes(categorized);

  if (outputPath) {
    writeFileSync(outputPath, markdown, 'utf8');
    console.log(`Release notes written to ${outputPath}`);
  }

  return markdown;
}

// Auto-run if executed directly from CLI
if (process.argv[1] && process.argv[1].replace(/\\/g, '/').endsWith('generate-release-notes.js')) {
  const outputPath = process.argv[2] || 'release_notes.md';
  generateReleaseNotes(outputPath);
}
