#!/bin/bash
# Worklog aggregation script
# Usage: ./aggregate.sh [projects|priority|category|all]

set -e

WORKLOG_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FILES=(
    "ARCHITECTURE.md"
    "DUPLICATION.md"
    "DEPENDENCIES.md"
    "INTEGRATION.md"
    "PERFORMANCE.md"
    "RESEARCH.md"
    "GOVERNANCE.md"
)

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

usage() {
    echo "Usage: $0 [projects|priority|category|all|summary]"
    echo ""
    echo "Aggregates worklog entries by different dimensions."
    exit 1
}

extract_entries() {
    local file="$1"
    local content=$(cat "$WORKLOG_DIR/$file" 2>/dev/null || echo "")
    # Extract entries from ## YYYY-MM-DD onwards until ---
    echo "$content" | awk '/^## [0-9]{4}-[0-9]{2}-[0-9]{2} - / { start=1; print; next }
         start { 
           if (/^---$/) { start=0; next }
           print 
         }'
}

get_priority() {
    echo "$1" | grep -oE '\*\*Priority:\*\* P[0-3]' | sed 's/\*\*Priority:\*\* //' | head -1
}

get_project() {
    echo "$1" | grep -oE '\*\*Project:\*\* \[[^]]+\]' | sed 's/\*\*Project:\*\* //' | head -1
}

get_category() {
    echo "$1" | grep -oE '\*\*Category:\*\* [a-z-]+' | sed 's/\*\*Category:\*\* //' | head -1
}

get_status() {
    echo "$1" | grep -oE '\*\*Status:\*\* [a-z_]+' | sed 's/\*\*Status:\*\* //' | head -1
}

get_title() {
    echo "$1" | grep -oE '^## [0-9]{4}-[0-9]{2}-[0-9]{2} - .+' | sed 's/^## [0-9]{4}-[0-9]{2}-[0-9]{2} - //' | head -1
}

agg_by_priority() {
    echo -e "${BLUE}=== Worklogs by Priority ===${NC}"
    echo ""
    
    for p in P0 P1 P2 P3; do
        echo -e "${YELLOW}$p${NC}"
        count=0
        for file in "${FILES[@]}"; do
            if [ -f "$WORKLOG_DIR/$file" ]; then
                entries=$(grep -A2 "\*\*Priority:\*\* $p" "$WORKLOG_DIR/$file" 2>/dev/null | grep "^## " | head -10)
                if [ -n "$entries" ]; then
                    echo "$entries" | while IFS= read -r line; do
                        echo "  - $line"
                        count=$((count + 1))
                    done
                fi
            fi
        done
        echo ""
    done
}

agg_by_project() {
    echo -e "${BLUE}=== Worklogs by Project ===${NC}"
    echo ""
    
    projects=("[AgilePlus]" "[heliosCLI]" "[thegent]" "[cross-repo]")
    
    for proj in "${projects[@]}"; do
        echo -e "${YELLOW}$proj${NC}"
        for file in "${FILES[@]}"; do
            entries=$(extract_entries "$file")
            echo "$entries" | while IFS= read -r entry; do
                if [ -n "$entry" ]; then
                    project=$(get_project "$entry")
                    title=$(get_title "$entry")
                    if [ "$project" = "$proj" ] && [ -n "$title" ]; then
                        echo "  - $title"
                    fi
                fi
            done
        done
        echo ""
    done
}

agg_by_category() {
    echo -e "${BLUE}=== Worklogs by Category ===${NC}"
    echo ""
    
    for file in "${FILES[@]}"; do
        category="${file%.md}"
        count=$(extract_entries "$file" | grep -c "^## 20")
        echo -e "${YELLOW}$category${NC} ($count entries)"
        echo "  File: $file"
        echo ""
    done
}

agg_summary() {
    echo -e "${BLUE}=== Worklog Summary ===${NC}"
    echo ""
    
    total=0
    for file in "${FILES[@]}"; do
        if [ -f "$WORKLOG_DIR/$file" ]; then
            count=$(grep -c "^## 20" "$WORKLOG_DIR/$file" 2>/dev/null || true)
            total=$((total + count))
            echo -e "$file: ${GREEN}$count${NC} entries"
        fi
    done
    echo ""
    echo -e "Total: ${GREEN}$total${NC} entries"
    echo ""
    
    echo "By Priority:"
    for p in P0 P1 P2 P3; do
        count=0
        for file in "${FILES[@]}"; do
            if [ -f "$WORKLOG_DIR/$file" ]; then
                c=$(grep -c "\*\*Priority:\*\* $p" "$WORKLOG_DIR/$file" 2>/dev/null || true)
                count=$((count + c))
            fi
        done
        echo "  $p: $count"
    done
    echo ""
    
    echo "By Project:"
    for proj in "[AgilePlus]" "[heliosCLI]" "[thegent]" "[cross-repo]"; do
        count=0
        escaped_proj=$(echo "$proj" | sed 's/\[/\\[/g; s/\]/\\]/g')
        for file in "${FILES[@]}"; do
            if [ -f "$WORKLOG_DIR/$file" ]; then
                c=$(grep -c "\*\*Project:\*\* $escaped_proj" "$WORKLOG_DIR/$file" 2>/dev/null || true)
                count=$((count + c))
            fi
        done
        echo "  $proj: $count"
    done
}

agg_all() {
    echo -e "${BLUE}=== All Worklog Entries ===${NC}"
    echo ""
    
    for file in "${FILES[@]}"; do
        category="${file%.md}"
        echo -e "${YELLOW}--- $category ---${NC}"
        grep -A1 "^## 20" "$WORKLOG_DIR/$file" 2>/dev/null | head -200 || true
        echo ""
    done
}

case "${1:-}" in
    projects)
        agg_by_project
        ;;
    priority)
        agg_by_priority
        ;;
    category)
        agg_by_category
        ;;
    summary)
        agg_summary
        ;;
    all)
        agg_summary
        echo ""
        agg_all
        ;;
    *)
        echo -e "${RED}Error: Unknown command '$1'${NC}"
        usage
        ;;
esac
