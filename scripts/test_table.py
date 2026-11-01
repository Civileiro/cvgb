import re
import xml.etree.ElementTree as ET

JUNIT_PROFILE_FILE = "target/nextest/ci/junit.xml"
README_FILE = "README.md"


rows: list[tuple[str, int, int]] = []
for suite in ET.parse(JUNIT_PROFILE_FILE).getroot().iter("testsuite"):
    name = suite.get("name")
    if name is None:
        continue
    name = name.split("::")[-1]
    total = 0
    passed = 0
    for test in suite.iter("testcase"):
        total += 1
        if not any(
            test.find(tag) is not None for tag in ("failure", "error", "skipped")
        ):
            passed += 1
    rows.append((name, passed, total))

rows.sort()
lines = ["| Suite | Passing | Total | % |", "|---|---:|---:|---:|"]
for name, passed, total in rows:
    rate = f"{passed / total:.0%}" if total else "-"
    lines.append(f"| {name} | {passed} | {total} | {rate} |")

total_passed = sum(r[1] for r in rows)
total_total = sum(r[2] for r in rows)
total_rate = f"{total_passed / total_total:.0%}" if total_total else "-"
lines.append(f"| All | {total_passed} | {total_total} | {total_rate} |")

table = "\n".join(lines)

with open(README_FILE, encoding="utf-8") as readme:
    text = readme.read()
new = re.sub(
    r"(<!-- TESTS:START -->).*?(<!-- TESTS:END -->)",
    lambda m: f"{m.group(1)}\n{table}\n{m.group(2)}",
    text,
    flags=re.DOTALL,
)
with open(README_FILE, "w", encoding="utf-8") as readme:
    readme.write(new)
