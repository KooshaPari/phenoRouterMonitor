import subprocess
import json
import sys

GH_PATH = "/opt/homebrew/bin/gh"

def run_command(cmd, cwd=None):
    try:
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True, cwd=cwd)
        return result
    except Exception as e:
        print(f"Error running command: {cmd}\n{e}")
        return None

def main():
    repo_dir = "/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus"
    
    # 1. Get all remote branches (excluding main and HEAD)
    res = run_command(f"{GH_PATH} api repos/KooshaPari/AgilePlus/branches --paginate --jq '.[].name'", cwd=repo_dir)
    if not res or res.returncode != 0:
        print("Failed to get branches")
        return
    branches = [b.strip() for b in res.stdout.strip().split("\n") if b.strip() and b.strip() != "main"]
    
    # 2. Get all open PRs
    res = run_command(f"{GH_PATH} pr list --repo KooshaPari/AgilePlus --state open --json headRefName", cwd=repo_dir)
    if not res or res.returncode != 0:
        print("Failed to get open PRs")
        return
    open_prs_data = json.loads(res.stdout)
    open_pr_branches = {pr['headRefName'] for pr in open_prs_data}
    
    branches_without_pr = [b for b in branches if b not in open_pr_branches]
    
    print(f"Total branches: {len(branches)}")
    print(f"Open PR branches: {len(open_pr_branches)}")
    print(f"Branches without PR: {len(branches_without_pr)}")
    
    report = []
    
    for branch in branches_without_pr:
        print(f"\nProcessing branch: {branch}")
        
        # 1. Try to open PR
        res = run_command(f"{GH_PATH} pr create --repo KooshaPari/AgilePlus --head {branch} --base main --title '[{branch}] sync: align with main' --body 'Ensuring every branch has an open PR.'", cwd=repo_dir)
        
        if res and res.returncode == 0:
            print(f"Successfully created PR for {branch}")
            report.append(f"CREATED: {branch}")
            continue
            
        # 2. Check if it failed because of no diff
        if res and "No commits between main and" in res.stderr:
            print(f"Skipping {branch}: No diff with main")
            report.append(f"SKIPPED (no diff): {branch}")
            continue
            
        # 3. Check if it failed because it's behind main (or other reasons needing merge)
        # Actually, if it's behind main but has diffs, PR creation usually works but shows conflict.
        # But if the user wants us to merge main if it fails, we should check common failure messages.
        # "is not up to date" or similar.
        
        # To be safe, let's try to merge main into it if it's not a 'no diff' error.
        print(f"Attempting to merge main into {branch}...")
        
        # Fetch, checkout, merge, push
        run_command("git fetch origin", cwd=repo_dir)
        run_command(f"git checkout {branch}", cwd=repo_dir)
        merge_res = run_command("git merge origin/main --no-edit", cwd=repo_dir)
        
        if merge_res and merge_res.returncode == 0:
            push_res = run_command(f"git push origin {branch}", cwd=repo_dir)
            if push_res and push_res.returncode == 0:
                # Try creating PR again
                res = run_command(f"{GH_PATH} pr create --repo KooshaPari/AgilePlus --head {branch} --base main --title '[{branch}] sync: align with main' --body 'Ensuring every branch has an open PR after merging main.'", cwd=repo_dir)
                if res and res.returncode == 0:
                    print(f"Successfully created PR for {branch} after merge")
                    report.append(f"CREATED (after merge): {branch}")
                else:
                    print(f"Failed to create PR for {branch} even after merge: {res.stderr}")
                    report.append(f"FAILED (after merge): {branch} - {res.stderr.strip()}")
            else:
                print(f"Failed to push {branch} after merge: {push_res.stderr}")
                report.append(f"FAILED (push): {branch}")
        else:
            # Check for conflict
            if merge_res and "CONFLICT" in merge_res.stdout:
                print(f"Merge conflict for {branch}")
                run_command("git merge --abort", cwd=repo_dir)
                report.append(f"FAILED (conflict): {branch}")
            else:
                print(f"Merge failed for {branch}: {merge_res.stderr if merge_res else 'Unknown'}")
                report.append(f"FAILED (merge): {branch}")
                
        # Return to main
        run_command("git checkout main", cwd=repo_dir)

    print("\n--- Final Report ---")
    for item in report:
        print(item)

if __name__ == "__main__":
    main()
