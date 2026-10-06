#!/usr/bin/env python3
"""Collect facts about the Tantu workspace for the code-to-docs skill.

Read-only: it reads the repository and writes JSON (default) or the rendered HTML page.
It never edits anything in the repository except the --out file you name.

    python3 collect.py --root <repo> --json                      # facts as JSON on stdout
    python3 collect.py --root <repo> --out docs/interactive/index.html

The spec/test matching mirrors `cargo xtask spec-coverage` (xtask/src/lib.rs, spec
docs/specs/xtask/spec-coverage.md). If the two disagree, the xtask is right: fix this file.
"""

import argparse
import datetime
import glob
import json
import os
import re
import subprocess
import sys
import tomllib

HERE = os.path.dirname(os.path.abspath(__file__))

# Crates that may depend on these external crates (AGENTS.md dependency rule).
RESTRICTED_EXTERNAL = {
    "wgpu": "tantu-render-",
    "tiny-skia": "tantu-render-",
    "winit": "tantu-platform-",
}


def read(path):
    with open(path, encoding="utf-8") as f:
        return f.read()


def rel(root, path):
    return os.path.relpath(path, root).replace(os.sep, "/")


def walk_files(root, ext):
    """Files under root, skipping `target` and hidden directories (XTASK-COV-11)."""
    out = []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = sorted(d for d in dirnames if d != "target" and not d.startswith("."))
        for name in sorted(filenames):
            if name.endswith(ext):
                out.append(os.path.join(dirpath, name))
    return out


def section(text, heading):
    """Body of the markdown section whose heading line is exactly `heading`."""
    lines = text.splitlines()
    level = len(heading) - len(heading.lstrip("#"))
    body, inside = [], False
    for line in lines:
        if inside:
            m = re.match(r"^(#+)\s", line)
            if m and len(m.group(1)) <= level:
                break
            body.append(line)
        elif line.strip() == heading:
            inside = True
    return "\n".join(body).strip()


def section_prefix(text, prefix):
    for line in text.splitlines():
        if line.startswith(prefix):
            return section(text, line.strip()), line.lstrip("#").strip()
    return "", ""


# ---------------------------------------------------------------- specs (mirrors xtask)

RULE_ID = re.compile(r"^[A-Z][A-Z0-9]*(?:-[A-Z][A-Z0-9]*)*-[0-9]{2,}$")


def is_rule_id(rule_id):
    return "-" in rule_id and bool(RULE_ID.match(rule_id))


def parse_spec(text):
    status_line = None
    meta = {}
    title = ""
    rules = []
    in_code = False
    lines = text.splitlines()
    current = None
    for index, line in enumerate(lines):
        trimmed = line.lstrip()
        if trimmed.startswith("```"):
            in_code = not in_code
            current = None
            continue
        if in_code:
            continue
        if not title and line.startswith("# "):
            title = line[2:].strip()
        if trimmed.startswith("- **Status:**"):
            if status_line is None:
                status_line = trimmed[len("- **Status:**"):].strip()
            current = None
            continue
        m = re.match(r"^- \*\*(Crate|Plan item|Related):\*\*\s*(.*)$", trimmed)
        if m and m.group(1) not in meta:
            meta[m.group(1)] = m.group(2).strip()
        removed = False
        rest = None
        if trimmed.startswith("- ~~**"):
            rest, removed = trimmed[len("- ~~**"):], True
        elif trimmed.startswith("- **"):
            rest = trimmed[len("- **"):]
        if rest is not None and ":**" in rest:
            rule_id, after = rest.split(":**", 1)
            if is_rule_id(rule_id):
                current = {
                    "id": rule_id,
                    "removed": removed,
                    "line": index + 1,
                    "text": after.strip(),
                    "indent": len(line) - len(trimmed),
                }
                rules.append(current)
                continue
        # Continuation lines of the current rule: indented, non-blank, not a new item.
        if current is not None:
            indent = len(line) - len(trimmed)
            if trimmed and indent > current["indent"] and not trimmed.startswith("- "):
                current["text"] += " " + trimmed.strip()
            else:
                current = None
    for r in rules:
        r.pop("indent", None)
        if r["removed"]:
            r["text"] = r["text"].replace("~~", "").strip()
    first = status_line.split()[0] if status_line else None
    status = first if first in ("Draft", "Agreed", "Implemented") else None
    return {"title": title, "status": status, "status_line": status_line, "meta": meta,
            "rules": rules}


def test_name(rule_id):
    return rule_id.lower().replace("-", "_")


FN_RE = re.compile(r"(?<!\S)fn\s+([A-Za-z0-9_]+)")


def collect_fns(root):
    """Every fn name in every .rs file, with file:line (XTASK-COV-11, -12)."""
    fns = []
    for path in walk_files(root, ".rs"):
        try:
            src = read(path)
        except (UnicodeDecodeError, OSError):
            continue
        starts = [0] + [m.end() for m in re.finditer(r"\n", src)]
        for m in FN_RE.finditer(src):
            line = _line_of(starts, m.start())
            fns.append({"name": m.group(1), "file": rel(root, path), "line": line})
    return fns


def _line_of(starts, offset):
    lo, hi = 0, len(starts) - 1
    while lo < hi:
        mid = (lo + hi + 1) // 2
        if starts[mid] <= offset:
            lo = mid
        else:
            hi = mid - 1
    return lo + 1


def collect_specs(root, fns):
    specs, problems = [], []
    checked = draft = 0
    by_name = {}
    for f in fns:
        by_name.setdefault(f["name"], []).append(f)
    files = sorted(
        p for p in walk_files(os.path.join(root, "docs", "specs"), ".md")
        if os.path.basename(p) not in ("README.md", "TEMPLATE.md")
    )
    for path in files:
        rp = rel(root, path)
        try:
            spec = parse_spec(read(path))
        except UnicodeDecodeError:
            problems.append({"path": rp, "message": "cannot be read as UTF-8 text"})
            continue
        spec["path"] = rp
        spec["area"] = rp.split("/")[2] if rp.count("/") >= 3 else ""
        enforced = spec["status"] in ("Agreed", "Implemented")
        if spec["status"] is None:
            problems.append({"path": rp, "message": "no valid status line"})
        elif spec["status"] == "Draft":
            draft += 1
        else:
            checked += 1
        seen, active = set(), 0
        for rule in spec["rules"]:
            name = test_name(rule["id"])
            rule["test_name"] = name
            rule["tests"] = [
                t for n, ts in by_name.items() if n == name or n.startswith(name + "_")
                for t in ts
            ]
            rule["tests"].sort(key=lambda t: (t["file"], t["line"]))
            if not enforced:
                continue
            if rule["id"] in seen:
                problems.append({"path": rp, "message":
                                 f"duplicate rule id {rule['id']} (line {rule['line']})"})
            seen.add(rule["id"])
            if rule["removed"]:
                continue
            active += 1
            if not rule["tests"]:
                problems.append({"path": rp, "message":
                                 f"{rule['id']} (line {rule['line']}) has no test named "
                                 f"{name} or {name}_*"})
        if enforced and active == 0:
            problems.append({"path": rp, "message": "no active rules"})
        specs.append(spec)
    return specs, {"checked": checked, "draft": draft, "problems": problems}


# ---------------------------------------------------------------- crates

PUB_RE = re.compile(
    r"^(\s*)pub\s+(?:(?:const|unsafe|async|extern\s+\"[^\"]*\")\s+)*"
    r"(fn|struct|enum|trait|type|const|static|mod|use|union)\b\s*(.*)$"
)
IMPL_RE = re.compile(r"^(\s*)(?:unsafe\s+)?impl\b(.*)$")


def _impl_owner(rest):
    rest = rest.strip()
    if rest.startswith("<"):
        depth = 0
        for i, ch in enumerate(rest):
            depth += ch == "<"
            depth -= ch == ">"
            if depth == 0:
                rest = rest[i + 1:].strip()
                break
    if re.search(r"\bfor\b", rest):
        rest = re.split(r"\bfor\b", rest, maxsplit=1)[1].strip()
        trait_impl = True
    else:
        trait_impl = False
    m = re.match(r"[&\w:]+", rest)
    return (m.group(0).split("::")[-1] if m else None), trait_impl


def doc_text(lines, index):
    """The `///` comment above line `index`, skipping attributes."""
    docs, i = [], index - 1
    while i >= 0:
        s = lines[i].strip()
        if s.startswith("///"):
            docs.append(s[3:][1:] if s[3:].startswith(" ") else s[3:])
        elif s.startswith("#[") or s.startswith("#!["):
            pass
        else:
            break
        i -= 1
    docs.reverse()
    return "\n".join(docs).strip()


def summary(doc):
    para = doc.split("\n\n", 1)[0].replace("\n", " ").strip()
    return para


def pub_items(root, crate_dir, crate_name):
    items = []
    for path in walk_files(os.path.join(crate_dir, "src"), ".rs"):
        lines = read(path).splitlines()
        owner = None  # (name, indent, trait_impl)
        in_test_mod = False
        test_indent = -1
        for i, line in enumerate(lines):
            stripped = line.strip()
            indent = len(line) - len(line.lstrip())
            if stripped.startswith("#[cfg(test)]"):
                in_test_mod, test_indent = True, indent
                continue
            if in_test_mod:
                if stripped == "}" and indent == test_indent:
                    in_test_mod = False
                continue
            if owner and stripped.startswith("}") and indent <= owner[1]:
                owner = None
            m = IMPL_RE.match(line)
            if m and stripped.endswith("{") or (m and "where" in stripped):
                name, trait_impl = _impl_owner(m.group(2).rstrip("{ ").strip())
                owner = (name, len(m.group(1)), trait_impl)
                continue
            m = PUB_RE.match(line)
            if not m:
                continue
            kind, rest = m.group(2), m.group(3)
            if kind == "use":
                name = rest.rstrip(";").strip()
            else:
                nm = re.match(r"([A-Za-z_][A-Za-z0-9_]*)", rest)
                if not nm:
                    continue
                name = nm.group(1)
            sig = stripped.rstrip("{").rstrip()
            j = i
            while kind == "fn" and not re.search(r"[{;]\s*$", lines[j]) and j + 1 < len(lines) \
                    and j - i < 12:
                j += 1
                sig += " " + lines[j].strip().rstrip("{").rstrip()
            sig = re.sub(r"\s+", " ", sig).replace("( ", "(").replace(" )", ")").strip()
            parent = owner[0] if owner and len(m.group(1)) > owner[1] else None
            if kind == "fn" and parent:
                kind = "method"
            doc = doc_text(lines, i)
            items.append({
                "crate": crate_name,
                "kind": kind,
                "name": name,
                "path": f"{parent}::{name}" if parent else name,
                "signature": sig,
                "doc": doc,
                "summary": summary(doc),
                "file": rel(root, path),
                "line": i + 1,
            })
    return items


def crate_doc(lib_path):
    if not os.path.exists(lib_path):
        return "", ""
    doc = []
    for line in read(lib_path).splitlines():
        s = line.strip()
        if s.startswith("//!"):
            doc.append(s[3:][1:] if s[3:].startswith(" ") else s[3:])
        elif s and not s.startswith("#!["):
            break
    text = "\n".join(doc).strip()
    title = ""
    m = re.match(r"#\s+(.*)", text)
    if m:
        title = m.group(1).strip()
    return title, text


def agents_table(agents):
    rows = {}
    order = []
    body = section(agents, "## Workspace layout (target)")
    for line in body.splitlines():
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) != 3 or not cells[0].startswith("`"):
            continue
        name = cells[0].strip("`").rstrip("/")
        order.append(name)
        rows[name] = {"responsibility": cells[1], "allowed_raw": cells[2]}
    for idx, name in enumerate(order):
        raw = re.sub(r"\(.*?\)", "", rows[name]["allowed_raw"]).strip()
        if raw in ("—", "-", ""):
            allowed = []
        elif raw == "everything above":
            allowed = [n for n in order[:idx] if n.startswith("tantu")]
        else:
            allowed = []
            for part in raw.split(","):
                part = part.strip()
                if not part:
                    continue
                allowed.append(part if part.startswith("tantu") else "tantu-" + part)
        rows[name]["allowed"] = allowed
    return order, rows


def collect_crates(root, specs):
    manifest = tomllib.loads(read(os.path.join(root, "Cargo.toml")))
    members = []
    for pattern in manifest.get("workspace", {}).get("members", []):
        members += sorted(glob.glob(os.path.join(root, pattern)))
    order, table = agents_table(read(os.path.join(root, "AGENTS.md")))
    crates, violations = [], []
    names = set()
    for crate_dir in members:
        cargo = os.path.join(crate_dir, "Cargo.toml")
        if not os.path.exists(cargo):
            continue
        pkg = tomllib.loads(read(cargo))
        name = pkg["package"]["name"]
        names.add(name)
        deps = sorted(pkg.get("dependencies", {}).keys())
        dev = sorted(pkg.get("dev-dependencies", {}).keys())
        internal = [d for d in deps if d.startswith("tantu") or d == "xtask"]
        external = [d for d in deps if d not in internal]
        dev_internal = [d for d in dev if d.startswith("tantu")]
        dev_external = [d for d in dev if d not in dev_internal]
        row = table.get(name)
        allowed = row["allowed"] if row else None
        if row is None:
            violations.append({"crate": name, "message": "not in the AGENTS.md crate table"})
        else:
            for d in internal:
                if d not in allowed:
                    violations.append({"crate": name, "message":
                                       f"depends on {d}, which AGENTS.md does not allow"})
        for ext, prefix in RESTRICTED_EXTERNAL.items():
            if ext in deps + dev and not name.startswith(prefix):
                violations.append({"crate": name, "message":
                                   f"depends on {ext}; only {prefix}* crates may"})
        title, doc = crate_doc(os.path.join(crate_dir, "src", "lib.rs"))
        items = pub_items(root, crate_dir, name)
        short = name.replace("tantu-", "") if name != "tantu" else "tantu"
        spec_paths = [
            s["path"] for s in specs
            if f"`{name}`" in s["meta"].get("Crate", "") or s["area"] == short
        ]
        placeholder = "placeholder" in doc.lower() or not [
            it for it in items if it["kind"] not in ("mod", "use")]
        src_lines = sum(len(read(p).splitlines())
                        for p in walk_files(os.path.join(crate_dir, "src"), ".rs"))
        test_count = sum(len(re.findall(r"#\[test\]", read(p)))
                         for p in walk_files(crate_dir, ".rs"))
        crates.append({
            "name": name,
            "dir": rel(root, crate_dir),
            "description": pkg["package"].get("description", ""),
            "title": title,
            "doc": doc,
            "responsibility": row["responsibility"] if row else "",
            "allowed": allowed,
            "deps": internal,
            "external_deps": external,
            "dev_deps": dev_internal,
            "dev_external_deps": dev_external,
            "status": "placeholder" if placeholder else "implemented",
            "items": items,
            "specs": spec_paths,
            "src_lines": src_lines,
            "test_count": test_count,
            "tooling": name == "xtask",
        })
    for name in order:
        if name.startswith("tantu") and name not in names:
            violations.append({"crate": name, "message": "in AGENTS.md but not in the workspace"})
    return crates, violations


# ---------------------------------------------------------------- plan, handoff, adrs

def collect_plan(plan):
    phases = []
    body = section(plan, "## Phases")
    current = None
    for line in body.splitlines():
        if line.startswith("### "):
            current = {"title": line[4:].strip(), "items": [], "notes": []}
            phases.append(current)
            continue
        if current is None:
            continue
        m = re.match(r"^(\s*)- \[( |x|X)\] (.*)$", line)
        if m:
            current["items"].append({"depth": len(m.group(1)) // 2,
                                     "done": m.group(2) != " ", "text": m.group(3).strip()})
            continue
        m = re.match(r"^(\s*)- (.*)$", line)
        if m:
            current["notes"].append(m.group(2).strip())
        elif line.strip() and current["notes"] and line.startswith("  "):
            current["notes"][-1] += " " + line.strip()
    for p in phases:
        top = [i for i in p["items"] if i["depth"] == 0]
        p["done"] = sum(i["done"] for i in top)
        p["total"] = len(top)
    current_phase = next((p["title"] for p in phases if p["total"] and p["done"] < p["total"]),
                         None)
    return {"vision": section(plan, "## Vision"), "phases": phases,
            "current_phase": current_phase,
            "decisions": section(plan, "## Key design decisions (initial ADRs to write)"),
            "success": section(plan, "## Success criteria for 0.1"),
            "risks": section(plan, "## Risks")}


def collect_handoff(text):
    m = re.search(r"_Last updated: ([^_]+)_", text)
    resume, resume_title = section_prefix(text, "## Resume here")
    return {"last_updated": m.group(1).strip() if m else "",
            "resume_title": resume_title, "resume": resume,
            "next_steps": section_prefix(text, "## Next steps")[0],
            "open_questions": section(text, "## Open questions")}


def collect_adrs(root):
    adrs = []
    for path in sorted(glob.glob(os.path.join(root, "docs", "adr", "[0-9][0-9][0-9][0-9]-*.md"))):
        text = read(path)
        lines = text.splitlines()
        title = lines[0].lstrip("# ").strip() if lines else ""
        status, date, body_start = "", "", 1
        i = 1
        while i < len(lines):
            s = lines[i]
            if s.startswith("- **Status:**"):
                status = s[len("- **Status:**"):].strip()
                while i + 1 < len(lines) and lines[i + 1].startswith("  "):
                    i += 1
                    status += " " + lines[i].strip()
            elif s.startswith("- **Date:**"):
                date = s[len("- **Date:**"):].strip()
            elif s.startswith("## "):
                body_start = i
                break
            i += 1
        word = re.match(r"\w+", status)
        adrs.append({"number": os.path.basename(path)[:4], "title": title,
                     "file": rel(root, path), "status": status,
                     "status_word": word.group(0) if word else "", "date": date,
                     "body": "\n".join(lines[body_start:]).strip()})
    return adrs


def git(root, *args):
    try:
        return subprocess.run(["git", "-C", root, *args], capture_output=True, text=True,
                              check=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return ""


def collect(root):
    agents = read(os.path.join(root, "AGENTS.md"))
    fns = collect_fns(root)
    specs, coverage = collect_specs(root, fns)
    crates, violations = collect_crates(root, specs)
    rule_total = sum(len(s["rules"]) for s in specs)
    active = [r for s in specs for r in s["rules"] if not r["removed"]]
    return {
        "generated_at": datetime.datetime.now().astimezone().isoformat(timespec="seconds"),
        "git": {"branch": git(root, "rev-parse", "--abbrev-ref", "HEAD"),
                "commit": git(root, "rev-parse", "--short", "HEAD"),
                "commit_date": git(root, "log", "-1", "--format=%cs"),
                "dirty": bool(git(root, "status", "--porcelain"))},
        "overview": {"what": section(agents, "## What this project is"),
                     "authoring": section(agents, "## Authoring style we are aiming for"),
                     "architecture": section(agents, "## Architecture (the pipeline)")},
        "plan": collect_plan(read(os.path.join(root, "PLAN.md"))),
        "handoff": collect_handoff(read(os.path.join(root, "HANDOFF.md"))),
        "crates": crates,
        "violations": violations,
        "specs": specs,
        "coverage": coverage,
        "adrs": collect_adrs(root),
        "counts": {
            "crates": len(crates),
            "implemented_crates": sum(c["status"] == "implemented" for c in crates),
            "specs": len(specs),
            "rules": rule_total,
            "active_rules": len(active),
            "covered_rules": sum(bool(r["tests"]) for r in active),
            "pub_items": sum(len(c["items"]) for c in crates),
            "adrs": len(collect_adrs(root)),
        },
    }


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--root", default=".")
    ap.add_argument("--json", action="store_true", help="print the facts as JSON and exit")
    ap.add_argument("--template", default=os.path.join(HERE, "template.html"))
    ap.add_argument("--out", default=None, help="write the rendered page here")
    args = ap.parse_args()
    root = os.path.abspath(args.root)
    data = collect(root)
    if args.json or not args.out:
        json.dump(data, sys.stdout, indent=1, ensure_ascii=False)
        print()
        return
    blob = json.dumps(data, ensure_ascii=False, separators=(",", ":"))
    # Keep the JSON safe inside <script>: no "</" and no HTML comment openers.
    blob = blob.replace("</", "<\\/").replace("<!--", "<\\u0021--")
    page = read(args.template).replace("/*__TANTU_DATA__*/null", blob, 1)
    os.makedirs(os.path.dirname(os.path.abspath(args.out)), exist_ok=True)
    with open(args.out, "w", encoding="utf-8") as f:
        f.write(page)
    c = data["counts"]
    cov = data["coverage"]
    print(f"wrote {args.out}: {c['crates']} crates ({c['implemented_crates']} implemented), "
          f"{c['pub_items']} public items, {c['specs']} specs, {c['active_rules']} active rules "
          f"({c['covered_rules']} with tests), {c['adrs']} ADRs, "
          f"{len(data['violations'])} dependency issue(s)")
    print(f"spec-coverage (mirror): {cov['checked']} checked, {cov['draft']} draft, "
          f"{len(cov['problems'])} problem(s)")
    for p in cov["problems"]:
        print(f"  {p['path']}: {p['message']}")
    for v in data["violations"]:
        print(f"  dependency: {v['crate']}: {v['message']}")
    undocumented = [i for cr in data["crates"] for i in cr["items"]
                    if not i["doc"] and i["kind"] not in ("mod", "use")]
    print(f"public items without a doc comment: {len(undocumented)}")
    for i in undocumented:
        print(f"  {i['file']}:{i['line']}: {i['kind']} {i['path']}")


if __name__ == "__main__":
    main()
