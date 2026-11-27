/**
 * Draw everyone who has committed to this repository as one SVG of avatars, sized by commits.
 *
 * Reads the history with git rather than asking GitHub's contributor graph, so it also credits
 * what that graph drops: commits whose email is linked to no account, co-authors named in
 * `Co-authored-by:` trailers, and accounts deleted since. Names and emails go through
 * `.mailmap`; `.github/contributors.json` maps further emails to GitHub logins and hides people.
 *
 *     node .github/scripts/contributors.ts --repo owner/name --out contributors.svg
 *
 * Needs Node 24 or newer, which runs TypeScript directly. Set GITHUB_TOKEN to lift the API rate
 * limit. No dependencies. The drawing has no background, so it sits on light and dark pages.
 */

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { appendFileSync, existsSync, readFileSync, writeFileSync } from "node:fs";
import { parseArgs } from "node:util";

const API = "https://api.github.com";
const NOREPLY = /^(?:(\d+)\+)?([^@]+)@users\.noreply\.github\.com$/i;
const CONTACT = /^\s*(.*?)\s*<([^>]+)>\s*$/;

const MIN_RADIUS = 18;
const MAX_RADIUS = 78;
const GAP = 4;
// How many times wider than tall the cluster grows; the README shows it across the full column.
const ASPECT = 5;
// The exponent of the superellipse the cluster fills: 2 is an ellipse, higher is closer to the
// rectangle the README gives it, with fewer empty corners.
const SQUARENESS = 4;
const PADDING = 6;

/** A GitHub account; `login` is null when the account was deleted. */
interface Account {
  id: number;
  login: string | null;
}

interface Person {
  account: Account | null;
  names: Map<string, number>;
  emails: Set<string>;
  commits: number;
}

type Contact = [name: string, email: string];

interface Commit {
  sha: string;
  /** The author first, then any co-authors. */
  people: Contact[];
}

interface Circle {
  x: number;
  y: number;
  r: number;
}

interface Config {
  hide?: string[];
  aliases?: Record<string, string>;
}

function display(person: Person): string {
  if (person.account?.login) return person.account.login;
  return [...person.names].sort((a, b) => b[1] - a[1])[0][0];
}

class GitHub {
  readonly repo: string;
  readonly token: string | undefined;
  readonly cache = new Map<string, unknown>();

  constructor(repo: string, token: string | undefined) {
    this.repo = repo;
    this.token = token;
  }

  /** The body and media type at `url`, or null for a 404. */
  async get(url: string): Promise<{ body: Buffer; media: string } | null> {
    const headers: Record<string, string> = { "User-Agent": "bedrock-rs-contributors" };
    if (this.token && url.startsWith(API)) {
      headers.Authorization = `Bearer ${this.token}`;
      headers.Accept = "application/vnd.github+json";
    }
    const response = await fetch(url, { headers, signal: AbortSignal.timeout(30_000) });
    if (response.status === 404) return null;
    if (!response.ok) throw new Error(`${url}: HTTP ${response.status}`);
    const media = (response.headers.get("content-type") ?? "").split(";")[0].trim();
    return { body: Buffer.from(await response.arrayBuffer()), media };
  }

  async api<T>(path: string): Promise<T | null> {
    if (!this.cache.has(path)) {
      const found = await this.get(API + path);
      this.cache.set(path, found && JSON.parse(found.body.toString("utf8")));
    }
    return this.cache.get(path) as T | null;
  }

  async userById(id: number): Promise<Account> {
    const user = await this.api<{ login: string }>(`/user/${id}`);
    return { id, login: user?.login ?? null };
  }

  async user(login: string): Promise<Account | null> {
    const user = await this.api<{ id: number; login: string }>(`/users/${login}`);
    return user && { id: user.id, login: user.login };
  }

  async commitAuthor(sha: string): Promise<Account | null> {
    const commit = await this.api<{ author: { id: number; login: string } | null }>(
      `/repos/${this.repo}/commits/${sha}`,
    );
    const author = commit?.author;
    return author ? { id: author.id, login: author.login } : null;
  }
}

function git(...args: string[]): string {
  return execFileSync("git", args, { encoding: "utf8", maxBuffer: 1 << 28 });
}

function history(rev: string): Commit[] {
  const format = "%H%x1f%aN%x1f%aE%x1f%(trailers:key=Co-authored-by,valueonly,separator=%x1e)%x00";
  const commits: Commit[] = [];
  const trailers = new Set<string>();
  for (const raw of git("log", `--format=${format}`, rev).split("\0")) {
    const record = raw.replace(/^\n+|\n+$/g, "");
    if (!record) continue;
    const [sha, name, email, coauthors] = record.split("\x1f");
    const people: Contact[] = [[name, email]];
    for (const contact of coauthors.split("\x1e").filter(Boolean)) {
      const match = CONTACT.exec(contact);
      if (!match) continue;
      people.push([match[1], match[2]]);
      trailers.add(contact.trim());
    }
    commits.push({ sha, people });
  }

  // Trailers bypass `.mailmap`; put them through it as git does for authors.
  if (trailers.size) {
    const ordered = [...trailers].sort();
    const mapped = git("check-mailmap", ...ordered).split("\n");
    const lookup = new Map(ordered.map((contact, i) => [contact, mapped[i]]));
    for (const { people } of commits) {
      for (let i = 1; i < people.length; i++) {
        const match = CONTACT.exec(lookup.get(`${people[i][0]} <${people[i][1]}>`) ?? "");
        if (match) people[i] = [match[1], match[2]];
      }
    }
  }
  return commits;
}

function isBot(name: string, email: string, account: Account | null): boolean {
  return [name, email.split("@")[0], account?.login].some((s) => s?.endsWith("[bot]"));
}

async function collect(
  commits: Commit[],
  github: GitHub,
  aliases: Map<string, string>,
  hidden: Set<string>,
): Promise<Person[]> {
  const firstAuthored = new Map<string, string>();
  for (const { sha, people } of [...commits].reverse()) {
    const email = people[0][1].toLowerCase();
    if (!firstAuthored.has(email)) firstAuthored.set(email, sha);
  }

  const resolved = new Map<string, Account | null>();
  async function resolve(email: string): Promise<Account | null> {
    const key = email.toLowerCase();
    if (!resolved.has(key)) {
      const alias = aliases.get(key);
      const noreply = NOREPLY.exec(email);
      const sha = firstAuthored.get(key);
      let account: Account | null = null;
      if (alias) account = await github.user(alias);
      else if (noreply)
        account = noreply[1] ? await github.userById(Number(noreply[1])) : await github.user(noreply[2]);
      else if (sha) account = await github.commitAuthor(sha);
      resolved.set(key, account);
    }
    return resolved.get(key)!;
  }

  const people = new Map<string, Person>();
  for (const { people: contacts } of commits) {
    const credited = new Set<string>();
    for (const [name, email] of contacts) {
      const account = await resolve(email);
      if (isBot(name, email, account)) continue;
      if ([account?.login, name, email].some((s) => s && hidden.has(s.toLowerCase()))) continue;
      const key = account ? `account:${account.id}` : `email:${email.toLowerCase()}`;
      let person = people.get(key);
      if (!person) {
        person = { account, names: new Map(), emails: new Set(), commits: 0 };
        people.set(key, person);
      }
      person.names.set(name, (person.names.get(name) ?? 0) + 1);
      person.emails.add(email.toLowerCase());
      credited.add(key);
    }
    for (const key of credited) people.get(key)!.commits += 1;
  }

  return [...people.values()].sort(
    (a, b) => b.commits - a.commits || display(a).toLowerCase().localeCompare(display(b).toLowerCase()),
  );
}

function radius(commits: number, most: number): number {
  if (most <= 1) return MIN_RADIUS;
  return MIN_RADIUS + ((MAX_RADIUS - MIN_RADIUS) * Math.log(commits)) / Math.log(most);
}

/** The centres where a circle of radius `r` touches both placed circles `a` and `b`. */
function touching(a: Circle, b: Circle, r: number): [number, number][] {
  const d1 = a.r + r + GAP;
  const d2 = b.r + r + GAP;
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const d = Math.hypot(dx, dy);
  if (d === 0 || d > d1 + d2 || d < Math.abs(d1 - d2)) return [];
  const along = (d1 * d1 - d2 * d2 + d * d) / (2 * d);
  const across = Math.sqrt(Math.max(d1 * d1 - along * along, 0));
  const mx = a.x + (dx * along) / d;
  const my = a.y + (dy * along) / d;
  return [
    [mx - (dy * across) / d, my + (dx * across) / d],
    [mx + (dy * across) / d, my - (dx * across) / d],
  ];
}

/** Place circles largest first, each at the free spot nearest the centre of a wide superellipse. */
function pack(radii: number[]): Circle[] {
  const placed: Circle[] = [];
  const free = (x: number, y: number, r: number) =>
    placed.every((p) => Math.hypot(x - p.x, y - p.y) >= r + p.r + GAP - 1e-6);
  const cost = ([x, y]: [number, number]) =>
    Math.abs(x / ASPECT) ** SQUARENESS + Math.abs(y) ** SQUARENESS;

  for (const r of radii) {
    if (!placed.length) {
      placed.push({ x: 0, y: 0, r });
      continue;
    }
    const spots: [number, number][] = [];
    placed.forEach((circle, i) => {
      const reach = circle.r + r + GAP;
      for (let k = 0; k < 24; k++) {
        const t = (k * Math.PI) / 12;
        spots.push([circle.x + reach * Math.cos(t), circle.y + reach * Math.sin(t)]);
      }
      for (const other of placed.slice(i + 1)) spots.push(...touching(circle, other, r));
    });
    const [x, y] = spots
      .filter(([x, y]) => free(x, y, r))
      .sort((a, b) => cost(a) - cost(b) || a[1] - b[1] || a[0] - b[0])[0];
    placed.push({ x, y, r });
  }
  return placed;
}

/** A data URI of the person's picture: their GitHub avatar, else a Gravatar, else null. */
async function avatar(person: Person, size: number, github: GitHub): Promise<string | null> {
  const urls: string[] = [];
  if (person.account?.login) {
    urls.push(`https://avatars.githubusercontent.com/u/${person.account.id}?s=${size}&v=4`);
  }
  for (const email of [...person.emails].sort()) {
    const hash = createHash("sha256").update(email).digest("hex");
    urls.push(`https://www.gravatar.com/avatar/${hash}?s=${size}&d=404`);
  }
  for (const url of urls) {
    try {
      const found = await github.get(url);
      if (found) return `data:${found.media};base64,${found.body.toString("base64")}`;
    } catch (error) {
      console.warn(`warning: ${url}: ${error}`);
    }
  }
  return null;
}

function initials(name: string): string {
  const words = name.match(/[\p{L}\p{N}]+/gu) ?? [name];
  const [first, second] = words;
  return (first[0] + (second ? second[0] : first.slice(1, 2))).toUpperCase();
}

/** The `index`th badge colour; golden-angle hues, so no two badges look alike. */
function colour(index: number): string {
  const hue = ((0.58 + index * 0.618034) % 1) * 360;
  const [saturation, lightness] = [0.45, 0.55];
  const chroma = saturation * Math.min(lightness, 1 - lightness);
  const channel = (k: number) => {
    const at = (k + hue / 30) % 12;
    const value = lightness - chroma * Math.max(-1, Math.min(at - 3, 9 - at, 1));
    return Math.round(value * 255).toString(16).padStart(2, "0");
  };
  return `#${channel(0)}${channel(8)}${channel(4)}`;
}

function escape(text: string): string {
  return text.replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);
}

function n(value: number): string {
  return String(Math.round(value * 10) / 10);
}

async function render(people: Person[], circles: Circle[], github: GitHub): Promise<string> {
  const left = Math.min(...circles.map((c) => c.x - c.r)) - PADDING;
  const top = Math.min(...circles.map((c) => c.y - c.r)) - PADDING;
  const width = Math.max(...circles.map((c) => c.x + c.r)) + PADDING - left;
  const height = Math.max(...circles.map((c) => c.y + c.r)) + PADDING - top;
  const pictures = await Promise.all(
    people.map((person, i) => avatar(person, Math.min(460, Math.ceil(circles[i].r * 4)), github)),
  );

  const out = [
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${n(left)} ${n(top)} ${n(width)} ${n(height)}" ` +
      `width="${Math.ceil(width)}" height="${Math.ceil(height)}">`,
  ];
  let badges = 0;
  people.forEach((person, i) => {
    const { x, y, r } = circles[i];
    const name = display(person);
    const picture = pictures[i];
    const at = `cx="${n(x)}" cy="${n(y)}" r="${n(r)}"`;
    out.push(`<g><title>${escape(name)}: ${person.commits} commit${person.commits === 1 ? "" : "s"}</title>`);
    if (picture) {
      out.push(`<clipPath id="c${i}"><circle ${at}/></clipPath>`);
      out.push(
        `<image href="${picture}" x="${n(x - r)}" y="${n(y - r)}" width="${n(2 * r)}" ` +
          `height="${n(2 * r)}" clip-path="url(#c${i})" preserveAspectRatio="xMidYMid slice"/>`,
      );
    } else {
      out.push(`<circle ${at} fill="${colour(badges++)}"/>`);
      out.push(
        `<text x="${n(x)}" y="${n(y)}" font-size="${n(r * 0.8)}" fill="#fff" ` +
          `font-family="-apple-system,Segoe UI,Helvetica,Arial,sans-serif" font-weight="600" ` +
          `text-anchor="middle" dominant-baseline="central">${escape(initials(name))}</text>`,
      );
    }
    out.push(`<circle ${at} fill="none" stroke="#808080" stroke-opacity="0.35" stroke-width="1"/></g>`);
  });
  out.push("</svg>\n");
  return out.join("\n");
}

function summary(people: Person[]): string {
  const rows = ["| | Contributor | Commits |", "| ---: | --- | ---: |"];
  people.forEach((person, i) => {
    const name = display(person);
    let who = name;
    if (person.account?.login) who = `[${name}](https://github.com/${name})`;
    else if (person.account) who += " (deleted account)";
    else who += " (no linked account)";
    rows.push(`| ${i + 1} | ${who} | ${person.commits} |`);
  });
  return rows.join("\n") + "\n";
}

async function main(): Promise<void> {
  const { values } = parseArgs({
    options: {
      repo: { type: "string", default: process.env.GITHUB_REPOSITORY },
      rev: { type: "string", default: "HEAD" },
      config: { type: "string", default: ".github/contributors.json" },
      out: { type: "string", default: "contributors.svg" },
    },
  });
  if (!values.repo) throw new Error("--repo or GITHUB_REPOSITORY is required");

  const config: Config = existsSync(values.config!)
    ? JSON.parse(readFileSync(values.config!, "utf8"))
    : {};
  const aliases = new Map(
    Object.entries(config.aliases ?? {}).map(([email, login]) => [email.toLowerCase(), login]),
  );
  const hidden = new Set((config.hide ?? []).map((entry) => entry.toLowerCase()));

  const github = new GitHub(values.repo, process.env.GITHUB_TOKEN);
  const people = await collect(history(values.rev!), github, aliases, hidden);
  const circles = pack(people.map((p) => radius(p.commits, people[0].commits)));
  writeFileSync(values.out!, await render(people, circles, github));

  const table = summary(people);
  console.log(table);
  if (process.env.GITHUB_STEP_SUMMARY) {
    appendFileSync(process.env.GITHUB_STEP_SUMMARY, `## Contributors\n\n${table}`);
  }
}

await main();
