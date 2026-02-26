PR-02: Key-diff engine using git refs (add/remove/update)
Goal

Compute added / removed / updated keys between BASE and HEAD for en.json (key-level diff, not line-level).

Changes

crates/greentic-i18n-translator/src/git_diff.rs

merge_base(origin/main, HEAD) helper (optional; allow --base explicit)

read_file_at_ref(ref, path) using git show <ref>:<path>

diff_en_maps(base_map, head_map) -> {added, removed, updated}

Wire into CLI diff command:

output JSON report like:

{ "added": [...], "removed": [...], "updated": [...] }
Tests

Use temp git repo fixture (or inline “maps” tests if you prefer no git dependency in unit tests):

verify added/updated/removed computed correctly

If avoiding real git in tests: keep diff_maps() pure and test that; git show wrapper can be lightly tested in an ignored integration test.