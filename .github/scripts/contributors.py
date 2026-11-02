#!/usr/bin/env python3
"""Draw everyone who has committed to this repository as one SVG of avatars, sized by commits.

Reads the history with git rather than asking GitHub's contributor graph, so it also credits
what that graph drops: commits whose email is linked to no account, co-authors named in
`Co-authored-by:` trailers, and accounts deleted since. Names and emails go through `.mailmap`;
`.github/contributors.toml` maps further emails to GitHub logins and hides people.

    python3 .github/scripts/contributors.py --repo owner/name --out contributors.svg

Set GITHUB_TOKEN to lift the API rate limit. Standard library only.
"""

import argparse
import base64
import colorsys
import hashlib
import html
import json
import math
import os
import re
import subprocess
import sys
import tomllib
import urllib.error
import urllib.request
from collections import Counter
from dataclasses import dataclass, field

API = "https://api.github.com"
NOREPLY = re.compile(r"^(?:(\d+)\+)?([^@]+)@users\.noreply\.github\.com$", re.IGNORECASE)
CONTACT = re.compile(r"^\s*(.*?)\s*<([^>]+)>\s*$")

MIN_RADIUS = 18
MAX_RADIUS = 78
GAP = 4
# How many times wider than tall the cluster grows; a README is a wide page.
ASPECT = 3.2
PADDING = 6


@dataclass
class Account:
    id: int
    login: str | None  # None when the account was deleted


@dataclass
class Person:
    account: Account | None
    names: Counter = field(default_factory=Counter)
    emails: set = field(default_factory=set)
    commits: int = 0

    @property
    def display(self) -> str:
        if self.account and self.account.login:
            return self.account.login
        return self.names.most_common(1)[0][0]


class GitHub:
    def __init__(self, repo: str, token: str | None):
        self.repo = repo
        self.token = token
        self.cache: dict[str, object] = {}

    def get(self, url: str) -> tuple[bytes | None, str | None]:
        """The body and media type at `url`, or `(None, None)` for a 404."""
        request = urllib.request.Request(url, headers={"User-Agent": "bedrock-rs-contributors"})
        if self.token and url.startswith(API):
            request.add_header("Authorization", f"Bearer {self.token}")
            request.add_header("Accept", "application/vnd.github+json")
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                return response.read(), response.headers.get_content_type()
        except urllib.error.HTTPError as error:
            if error.code == 404:
                return None, None
            raise

    def api(self, path: str):
        if path not in self.cache:
            body, _ = self.get(API + path)
            self.cache[path] = None if body is None else json.loads(body)
        return self.cache[path]

    def user_by_id(self, uid: int) -> Account:
        user = self.api(f"/user/{uid}")
        return Account(uid, user["login"] if user else None)

    def user(self, login: str) -> Account | None:
        user = self.api(f"/users/{login}")
        return Account(user["id"], user["login"]) if user else None

    def commit_author(self, sha: str) -> Account | None:
        commit = self.api(f"/repos/{self.repo}/commits/{sha}")
        author = commit and commit.get("author")
        return Account(author["id"], author["login"]) if author else None


def git(*args: str) -> str:
    return subprocess.run(["git", *args], check=True, capture_output=True, text=True).stdout


def history(rev: str) -> list[tuple[str, list[tuple[str, str]]]]:
    """Each commit as its hash and its people: the author first, then any co-authors."""
    fmt = "%H%x1f%aN%x1f%aE%x1f%(trailers:key=Co-authored-by,valueonly,separator=%x1e)%x00"
    commits, trailers = [], set()
    for record in git("log", f"--format={fmt}", rev).split("\0"):
        record = record.strip("\n")
        if not record:
            continue
        sha, name, email, coauthors = record.split("\x1f")
        people = [(name, email)]
        for contact in filter(None, coauthors.split("\x1e")):
            if match := CONTACT.match(contact):
                people.append(match.groups())
                trailers.add(contact.strip())
        commits.append((sha, people))

    # Trailers bypass `.mailmap`; put them through it as git does for authors.
    if trailers:
        ordered = sorted(trailers)
        mapped = dict(zip(ordered, git("check-mailmap", *ordered).splitlines()))
        for _, people in commits:
            for i, (name, email) in enumerate(people[1:], 1):
                if match := CONTACT.match(mapped.get(f"{name} <{email}>", "")):
                    people[i] = match.groups()
    return commits


def is_bot(name: str, email: str, account: Account | None) -> bool:
    login = account.login if account else None
    return any(s and s.endswith("[bot]") for s in (name, email.split("@")[0], login))


def collect(commits, github: GitHub, aliases: dict, hidden: set) -> list[Person]:
    first_authored: dict[str, str] = {}
    for sha, people in reversed(commits):
        first_authored.setdefault(people[0][1].lower(), sha)

    resolved: dict[str, Account | None] = {}

    def resolve(email: str) -> Account | None:
        key = email.lower()
        if key not in resolved:
            if login := aliases.get(key):
                resolved[key] = github.user(login)
            elif match := NOREPLY.match(email):
                uid, login = match.groups()
                resolved[key] = github.user_by_id(int(uid)) if uid else github.user(login)
            elif sha := first_authored.get(key):
                resolved[key] = github.commit_author(sha)
            else:
                resolved[key] = None
        return resolved[key]

    people: dict[tuple, Person] = {}
    for _, contacts in commits:
        credited = set()
        for name, email in contacts:
            account = resolve(email)
            if is_bot(name, email, account):
                continue
            login = account.login if account else None
            if any(s and s.lower() in hidden for s in (login, name, email)):
                continue
            key = ("account", account.id) if account else ("email", email.lower())
            person = people.setdefault(key, Person(account))
            person.names[name] += 1
            person.emails.add(email.lower())
            credited.add(key)
        for key in credited:
            people[key].commits += 1

    return sorted(people.values(), key=lambda p: (-p.commits, p.display.lower()))


def radius(commits: int, most: int) -> float:
    if most <= 1:
        return MIN_RADIUS
    return MIN_RADIUS + (MAX_RADIUS - MIN_RADIUS) * math.log(commits) / math.log(most)


def touching(a, b, r):
    """The centres where a circle of radius `r` touches both placed circles `a` and `b`."""
    (x1, y1, r1), (x2, y2, r2) = a, b
    d1, d2 = r1 + r + GAP, r2 + r + GAP
    dx, dy = x2 - x1, y2 - y1
    d = math.hypot(dx, dy)
    if d == 0 or d > d1 + d2 or d < abs(d1 - d2):
        return []
    along = (d1 * d1 - d2 * d2 + d * d) / (2 * d)
    across = math.sqrt(max(d1 * d1 - along * along, 0))
    mx, my = x1 + dx * along / d, y1 + dy * along / d
    return [(mx - dy * across / d, my + dx * across / d), (mx + dy * across / d, my - dx * across / d)]


def pack(radii: list[float]) -> list[tuple[float, float, float]]:
    """Place circles largest first, each at the free spot nearest a wide ellipse's centre."""
    placed: list[tuple[float, float, float]] = []

    def free(x, y, r):
        return all(math.hypot(x - px, y - py) >= r + pr + GAP - 1e-6 for px, py, pr in placed)

    for r in radii:
        if not placed:
            placed.append((0.0, 0.0, r))
            continue
        spots = []
        for i, circle in enumerate(placed):
            x, y, pr = circle
            reach = pr + r + GAP
            spots += [
                (x + reach * math.cos(t), y + reach * math.sin(t))
                for t in (k * math.pi / 12 for k in range(24))
            ]
            for other in placed[i + 1 :]:
                spots += touching(circle, other, r)
        x, y = min(
            (s for s in spots if free(*s, r)),
            key=lambda s: (math.hypot(s[0] / ASPECT, s[1]), s[1], s[0]),
        )
        placed.append((x, y, r))
    return placed


def avatar(person: Person, size: int, github: GitHub) -> str | None:
    """A data URI of the person's picture: their GitHub avatar, else a Gravatar, else none."""
    urls = []
    if person.account and person.account.login:
        urls.append(f"https://avatars.githubusercontent.com/u/{person.account.id}?s={size}&v=4")
    urls += [
        f"https://www.gravatar.com/avatar/{hashlib.sha256(e.encode()).hexdigest()}?s={size}&d=404"
        for e in sorted(person.emails)
    ]
    for url in urls:
        try:
            body, media = github.get(url)
        except (urllib.error.URLError, TimeoutError) as error:
            print(f"warning: {url}: {error}", file=sys.stderr)
            continue
        if body:
            return f"data:{media};base64,{base64.b64encode(body).decode()}"
    return None


def initials(name: str) -> str:
    words = re.findall(r"[^\W_]+", name) or [name]
    letters = words[0][0] + (words[1][0] if len(words) > 1 else words[0][1:2])
    return letters.upper()


def colour(index: int) -> str:
    """The `index`th badge colour; golden-angle hues, so no two badges look alike."""
    hue = (0.58 + index * 0.618034) % 1
    r, g, b = colorsys.hls_to_rgb(hue, 0.55, 0.45)
    return f"#{int(r * 255):02x}{int(g * 255):02x}{int(b * 255):02x}"


def render(people: list[Person], circles, github: GitHub) -> str:
    left = min(x - r for x, _, r in circles) - PADDING
    top = min(y - r for _, y, r in circles) - PADDING
    width = max(x + r for x, _, r in circles) + PADDING - left
    height = max(y + r for _, y, r in circles) + PADDING - top

    def n(v: float) -> str:
        return f"{v:.1f}".rstrip("0").rstrip(".")

    out = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{n(left)} {n(top)} {n(width)} {n(height)}" '
        f'width="{math.ceil(width)}" height="{math.ceil(height)}">'
    ]
    badges = 0
    for i, (person, (x, y, r)) in enumerate(zip(people, circles)):
        noun = "commit" if person.commits == 1 else "commits"
        out.append(f"<g><title>{html.escape(person.display)}: {person.commits} {noun}</title>")
        picture = avatar(person, min(460, math.ceil(r * 4)), github)
        if picture:
            out.append(f'<clipPath id="c{i}"><circle cx="{n(x)}" cy="{n(y)}" r="{n(r)}"/></clipPath>')
            out.append(
                f'<image href="{picture}" x="{n(x - r)}" y="{n(y - r)}" width="{n(2 * r)}" '
                f'height="{n(2 * r)}" clip-path="url(#c{i})" preserveAspectRatio="xMidYMid slice"/>'
            )
        else:
            out.append(f'<circle cx="{n(x)}" cy="{n(y)}" r="{n(r)}" fill="{colour(badges)}"/>')
            badges += 1
            out.append(
                f'<text x="{n(x)}" y="{n(y)}" font-size="{n(r * 0.8)}" fill="#fff" '
                f'font-family="-apple-system,Segoe UI,Helvetica,Arial,sans-serif" font-weight="600" '
                f'text-anchor="middle" dominant-baseline="central">'
                f"{html.escape(initials(person.display))}</text>"
            )
        out.append(
            f'<circle cx="{n(x)}" cy="{n(y)}" r="{n(r)}" fill="none" '
            f'stroke="#808080" stroke-opacity="0.35" stroke-width="1"/></g>'
        )
    out.append("</svg>\n")
    return "\n".join(out)


def summary(people: list[Person]) -> str:
    rows = ["| | Contributor | Commits |", "| ---: | --- | ---: |"]
    for rank, person in enumerate(people, 1):
        who = person.display
        if person.account and person.account.login:
            who = f"[{who}](https://github.com/{who})"
        elif person.account:
            who += " (deleted account)"
        else:
            who += " (no linked account)"
        rows.append(f"| {rank} | {who} | {person.commits} |")
    return "\n".join(rows) + "\n"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--repo", default=os.environ.get("GITHUB_REPOSITORY"), required=False)
    parser.add_argument("--rev", default="HEAD")
    parser.add_argument("--config", default=".github/contributors.toml")
    parser.add_argument("--out", default="contributors.svg")
    args = parser.parse_args()
    if not args.repo:
        parser.error("--repo or GITHUB_REPOSITORY is required")

    config = {}
    if os.path.exists(args.config):
        with open(args.config, "rb") as file:
            config = tomllib.load(file)
    aliases = {email.lower(): login for email, login in config.get("aliases", {}).items()}
    hidden = {entry.lower() for entry in config.get("hide", [])}

    github = GitHub(args.repo, os.environ.get("GITHUB_TOKEN"))
    people = collect(history(args.rev), github, aliases, hidden)
    most = people[0].commits
    circles = pack([radius(p.commits, most) for p in people])

    with open(args.out, "w", encoding="utf-8") as file:
        file.write(render(people, circles, github))

    table = summary(people)
    print(table)
    if step_summary := os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(step_summary, "a", encoding="utf-8") as file:
            file.write(f"## Contributors\n\n{table}")


if __name__ == "__main__":
    main()
