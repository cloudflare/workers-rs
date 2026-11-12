// Prints release notes from the pending changesets, or with --level the bump they call for.
import { readdirSync, readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";

const dir = ".changeset";
const changesets = readdirSync(dir)
  .filter((file) => file.endsWith(".md") && file !== "README.md")
  .sort()
  .map((file) => {
    const path = `${dir}/${file}`;
    const [, frontmatter, body] = readFileSync(path, "utf8").match(/^---\n([\s\S]*?)\n---\n([\s\S]*)$/);
    const level = /:\s*"?(major|minor)"?/.test(frontmatter) ? "minor" : "patch";
    // Squash merges end the subject with the PR number.
    const subjects = execFileSync("git", ["log", "--diff-filter=A", "--format=%s", "--", path], { encoding: "utf8" });
    const pr = subjects.match(/\(#(\d+)\)$/m)?.[1];
    const paragraphs = body.trim().split(/\n\s*\n/).map((p) => p.replace(/\s*\n\s*/g, " "));
    const text = paragraphs.join("\n\n  ") + (pr ? ` (#${pr})` : "");
    return { level, text };
  });

if (process.argv.includes("--level")) {
  console.log(changesets.some((c) => c.level === "minor") ? "minor" : "patch");
} else {
  const sections = [];
  for (const [level, title] of [["minor", "Minor Changes"], ["patch", "Patch Changes"]]) {
    const items = changesets.filter((c) => c.level === level);
    if (items.length) sections.push(`### ${title}\n\n${items.map((c) => `- ${c.text}`).join("\n")}`);
  }
  console.log(sections.join("\n\n"));
}
