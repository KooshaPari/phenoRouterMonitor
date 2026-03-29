#!/bin/bash
# Worklog aggregation script
# Usage: ./worklogs/aggregate.sh [project|priority|category|all]

set -e

WORKLOGS_DIR="$(dirname "$0")"
cd "$WORKLOGS_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

usage() {
    echo "Usage: $0 [project|priority|category|projects|all]"
    echo ""
    echo "Aggregates worklog entries by:"
    echo "  project   - Group by project ([AgilePlus], [thegent], etc.)"
    echo "  priority - Group by priority (P0, P1, P2, P3)"
    echo "  category - Group by category (architecture, duplication, etc.)"
    echo "  projects - Show project-level worklog summaries"
    echo "  all      - Show all worklogs"
    echo ""
    echo "Examples:"
    echo "  $0 project    # Show all AgilePlus entries"
    echo "  $0 priority   # Show P0 items first"
    echo "  $0 projects  # Show project-level summaries"
    echo "  $0 all       # Show everything"
}

# Extract entries from markdown files
extract_entries() {
    local pattern="$1"
    local files="*.md"

    # Extract date and title lines followed by project/priority markers
    awk -v pat="$pattern" '
        /^## [0-9]{4}-[0-9]{2}-[0-9]{2}/ { date = $0 }
        /\*\*Project:\*\*/ && /'"$pattern"'/ { print date }
    ' $files 2>/dev/null | sort -u
}

case "${1:-all}" in
    project)
        echo -e "${BLUE}=== WORKLOGS BY PROJECT ===${NC}"
        echo ""

        for project in "AgilePlus" "thegent" "heliosCLI" "cross-repo"; do
            echo -e "${GREEN}## $project${NC}"
            entries=$(awk '
                /^## [0-9]{4}-[0-9]{2}-[0-9]{2}/ { date = $0 }
                /\*\*Project:\*\*/ && /'"$project"'/ { print date }
            ' *.md 2>/dev/null | sort -u)
            if [ -n "$entries" ]; then
                echo "$entries" | sed 's/^/  /'
            else
                echo "  (none)"
            fi
            echo ""
        done
        ;;

    priority)
        echo -e "${BLUE}=== WORKLOGS BY PRIORITY ===${NC}"
        echo ""

        for p in P0 P1 P2 P3; do
            echo -e "${RED}## Priority $p${NC}"
            entries=$(awk '
                /^## [0-9]{4}-[0-9]{2}-[0-9]{2}/ { date = $0 }
                /\*\*Priority:\*\*/ && /'"$p"'/ { print date }
            ' *.md 2>/dev/null | sort -u)
            if [ -n "$entries" ]; then
                echo "$entries" | sed 's/^/  /'
            else
                echo "  (none)"
            fi
            echo ""
        done
        ;;

    category)
        echo -e "${BLUE}=== WORKLOGS BY CATEGORY ===${NC}"
        echo ""

        for cat in ARCHITECTURE DUPLICATION DEPENDENCIES INTEGRATION PERFORMANCE RESEARCH GOVERNANCE; do
            if [ -f "${cat}.md" ]; then
                echo -e "${GREEN}## $cat${NC}"
                grep "^## 20" "${cat}.md" | head -5 | sed 's/^/  /'
                echo ""
            fi
        done
        ;;

    projects)
        echo -e "${BLUE}=== PROJECT-LEVEL WORKLOGS ===${NC}"
        echo ""

        for proj in PROJECTS_agileplus PROJECTS_thegent PROJECTS_heliosCLI; do
            if [ -f "${proj}.md" ]; then
                echo -e "${GREEN}## ${proj#PROJECTS_}${NC}"
                # Show summary table
                grep -E "^\| Priority" -A 3 "${proj}.md" | head -5 | sed 's/^/  /'
                # Show recent entries
                echo "  Recent entries:"
                grep "^## 20" "${proj}.md" | head -3 | sed 's/^/    /'
                echo ""
            fi
        done

        echo -e "${YELLOW}Full project worklogs:${NC}"
        echo "  worklogs/PROJECTS_agileplus.md"
        echo "  worklogs/PROJECTS_thegent.md"
        echo "  worklogs/PROJECTS_heliosCLI.md"
        echo ""
        ;;

    all)
        echo -e "${BLUE}=== ALL WORKLOG ENTRIES ===${NC}"
        echo ""
        grep "^## 20" *.md | sort -r | head -30 | sed 's/^/  /'
        echo ""
        ;;

    *)
        usage
        exit 1
        ;;
esac

echo ""
echo -e "${YELLOW}Run '$0 --help' for usage information${NC}"
