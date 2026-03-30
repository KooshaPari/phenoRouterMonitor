import subprocess
import json
import sys

def run_command(command, cwd=None):
    try:
        result = subprocess.run(command, shell=True, capture_output=True, text=True, cwd=cwd)
        return result.stdout.strip(), result.stderr.strip(), result.returncode
    except Exception as e:
        return "", str(e), 1

def main():
    repo_path = "/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus"
    
    # Get unmerged branches
    print("Getting unmerged branches...")
    stdout, stderr, code = run_command("git branch -r --no-merged origin/main", cwd=repo_path)
    if code != 0:
        print(f"Error getting branches: {stderr}")
        return

    branches = []
    for line in stdout.splitlines():
        line = line.strip()
        if line.startswith("origin/") and "origin/HEAD" not in line:
            branches.append(line.replace("origin/", ""))
    
    print(f"Found {len(branches)} unmerged branches.")

    # Get open PR head branches
    print("Getting open PR head branches...")
    stdout, stderr, code = run_command("gh pr list --repo KooshaPari/AgilePlus --state open --limit 1000 --json headRefName", cwd=repo_path)
    if code != 0:
        print(f"Error getting PRs: {stderr}")
        return
    
    open_pr_refs = [pr["headRefName"] for pr in json.loads(stdout)]
    print(f"Found {len(open_pr_refs)} open PRs.")

    results = []

    for branch in branches:
        if branch in open_pr_refs:
            print(f"Skipping {branch}: PR already exists")
            continue
        
        print(f"----------------------------------------")
        print(f"Processing {branch}...")

        # Try to create PR
        title = f"[{branch}] sync: align with main"
        body = f"Automated PR created by subagent to ensure all branches have an open PR against main.\n\nMade with [Cursor](https://cursor.com)"
        
        cmd = f'gh pr create --repo KooshaPari/AgilePlus --base main --head "{branch}" --title "{title}" --body "{body}"'
        stdout, stderr, code = run_command(cmd, cwd=repo_path)
        
        if code == 0:
            pr_url = stdout.strip()
            print(f"Successfully created PR for {branch}: {pr_url}")
            results.append(f"Created PR for {branch}: {pr_url}")
        elif "no commits between main and" in stderr.lower() or "no commits between main and" in stdout.lower():
            print(f"Skipping {branch}: No differences from main")
        elif "already exists" in stderr.lower() or "already exists" in stdout.lower():
            print(f"Skipping {branch}: PR already exists (detected by gh)")
        else:
            print(f"PR creation failed for {branch}, checking if behind main...")
            # Try to merge main and push
            run_command(f'git checkout "{branch}" || git checkout -b "{branch}" "origin/{branch}"', cwd=repo_path)
            stdout_merge, stderr_merge, code_merge = run_command("git merge origin/main --no-edit", cwd=repo_path)
            
            if code_merge == 0:
                print(f"Merged main into {branch}, pushing...")
                run_command(f'git push origin "{branch}"', cwd=repo_path)
                stdout_retry, stderr_retry, code_retry = run_command(cmd, cwd=repo_path)
                if code_retry == 0:
                    pr_url = stdout_retry.strip()
                    print(f"Successfully created PR for {branch} after merge: {pr_url}")
                    results.append(f"Created PR for {branch} (after merge): {pr_url}")
                else:
                    print(f"Failed to create PR for {branch} even after merge: {stderr_retry}")
            else:
                print(f"Failed to merge origin/main into {branch}. Likely conflicts. Skipping.")
                run_command("git merge --abort", cwd=repo_path)
            
            run_command("git checkout main", cwd=repo_path)

    print("\nSummary of PRs created:")
    for res in results:
        print(res)

if __name__ == "__main__":
    main()
