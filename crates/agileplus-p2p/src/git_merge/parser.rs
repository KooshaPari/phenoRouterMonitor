/// Represents one side of a git conflict block.
#[derive(Debug)]
pub(crate) struct ConflictBlock {
    pub ours: String,
    pub theirs: String,
}

/// Parse a file that may contain standard git conflict markers:
/// ```text
/// <<<<<<< HEAD
/// ... ours ...
/// =======
/// ... theirs ...
/// >>>>>>> branch
/// ```
/// Returns the list of conflict blocks found. If none are found the file is
/// returned as-is in a synthetic block with `theirs` set to the same content.
pub(crate) fn parse_conflict_blocks(content: &str) -> Vec<ConflictBlock> {
    let mut blocks: Vec<ConflictBlock> = Vec::new();
    let mut ours_lines: Vec<&str> = Vec::new();
    let mut theirs_lines: Vec<&str> = Vec::new();
    let mut in_conflict = false;
    let mut in_theirs = false;
    let mut found_any = false;

    for line in content.lines() {
        if line.starts_with("<<<<<<<") {
            in_conflict = true;
            in_theirs = false;
            ours_lines.clear();
            theirs_lines.clear();
            found_any = true;
        } else if line.starts_with("=======") && in_conflict {
            in_theirs = true;
        } else if line.starts_with(">>>>>>>") && in_conflict {
            blocks.push(ConflictBlock {
                ours: ours_lines.join("\n"),
                theirs: theirs_lines.join("\n"),
            });
            in_conflict = false;
            in_theirs = false;
        } else if in_conflict {
            if in_theirs {
                theirs_lines.push(line);
            } else {
                ours_lines.push(line);
            }
        }
    }

    if !found_any {
        blocks.push(ConflictBlock {
            ours: content.to_string(),
            theirs: content.to_string(),
        });
    }

    blocks
}
