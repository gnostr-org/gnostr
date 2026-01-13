#!/bin/bash

# code2prompt.sh - Recursively generate markdown files containing folder contents
# Usage: ./code2prompt.sh [options] [directories]

set -euo pipefail

# Default configuration
INCLUDE_EXTENSIONS=("rs" "md" "toml" "yaml" "yml" "json" "sh" "conf" "ron" "txt")
EXCLUDE_DIRS=("target" "vendor" "node_modules" ".git" "coverage_tmp" "tmpgit" "coverage")
MAX_FILE_SIZE=1048576  # 1MB
RECURSIVE=true
DRY_RUN=false
VERBOSE=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1" >&2
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" >&2
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" >&2
}

# Show usage information
show_help() {
    cat << EOF
Usage: $0 [options] [directories...]

Recursively generate markdown files containing the concatenated contents of directories.
Creates <directory>/<directory>.md files for each processed directory.

OPTIONS:
    -r, --recursive       Process directories recursively (default: true)
    -d, --dry-run         Show what would be processed without creating files
    -i, --include EXT     File extensions to include (comma-separated, default: rs,md,toml,yaml,yml,json,sh,conf,ron,txt)
    -e, --exclude DIR     Directories to exclude (comma-separated, default: target,vendor,node_modules,.git,coverage_tmp,tmpgit,coverage)
    -s, --max-size SIZE   Maximum file size in bytes to process (default: 1MB)
    -v, --verbose         Enable verbose output
    -h, --help            Show this help message

EXAMPLES:
    $0                                    # Process all code directories in current directory
    $0 src examples                       # Process only src and examples directories
    $0 --dry-run --verbose src            # Show what would be processed in src directory
    $0 --include "rs,toml" --exclude ".git,target" src

EOF
}

# Parse command line arguments
parse_args() {
    local directories=()
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            -r|--recursive)
                RECURSIVE=true
                shift
                ;;
            --no-recursive)
                RECURSIVE=false
                shift
                ;;
            -d|--dry-run)
                DRY_RUN=true
                shift
                ;;
            -i|--include)
                IFS=',' read -ra INCLUDE_EXTENSIONS <<< "$2"
                shift 2
                ;;
            -e|--exclude)
                IFS=',' read -ra EXCLUDE_DIRS <<< "$2"
                shift 2
                ;;
            -s|--max-size)
                MAX_FILE_SIZE="$2"
                shift 2
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            -*)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
            *)
                directories+=("$1")
                shift
                ;;
        esac
    done
    
    # If no directories specified, use default code directories
    if [[ ${#directories[@]} -eq 0 ]]; then
        local default_dirs=("src" "examples" "app" "asyncgit" "crawler" "filetreelist" "git2-hooks" "legit" "qr" "query" "relay" "scopetime" "gnit" "gh" "git2-testing" "invalidstring")
        for dir in "${default_dirs[@]}"; do
            if [[ -d "$dir" ]]; then
                directories+=("$dir")
            fi
        done
    fi
    
    echo "${directories[@]}"
}

# Check if directory should be excluded
should_exclude_dir() {
    local dir_path="$1"
    local dir_name=$(basename "$dir_path")
    
    for exclude in "${EXCLUDE_DIRS[@]}"; do
        if [[ "$dir_name" == "$exclude" ]] || [[ "$dir_path" =~ ^.*$exclude.*$ ]]; then
            return 0
        fi
    done
    return 1
}

# Check if file should be included based on extension
should_include_file() {
    local file_path="$1"
    local extension="${file_path##*.}"
    
    for ext in "${INCLUDE_EXTENSIONS[@]}"; do
        if [[ "$extension" == "$ext" ]]; then
            return 0
        fi
    done
    return 1
}

# Check if file is text and within size limit
can_process_file() {
    local file_path="$1"
    
    # Check if file exists
    if [[ ! -f "$file_path" ]]; then
        return 1
    fi
    
    # Check file size
    local file_size=$(stat -f%z "$file_path" 2>/dev/null || stat -c%s "$file_path" 2>/dev/null || echo 0)
    if [[ $file_size -gt $MAX_FILE_SIZE ]]; then
        log_warn "Skipping large file: $file_path (${file_size} bytes)"
        return 1
    fi
    
    # Check if file is text (basic check)
    if file "$file_path" 2>/dev/null | grep -q "text"; then
        return 0
    elif [[ "${file_path##*.}" =~ ^(rs|md|toml|yaml|yml|json|sh|conf|ron|txt)$ ]]; then
        return 0
    else
        if [[ "$VERBOSE" == true ]]; then
            log_warn "Skipping binary file: $file_path"
        fi
        return 1
    fi
}

# Get language for syntax highlighting
get_language() {
    local file_path="$1"
    local extension="${file_path##*.}"
    
    case "$extension" in
        rs) echo "rust" ;;
        md) echo "markdown" ;;
        js|ts) echo "javascript" ;;
        py) echo "python" ;;
        go) echo "go" ;;
        java) echo "java" ;;
        cpp|c|h|hpp) echo "cpp" ;;
        sh|bash) echo "bash" ;;
        yaml|yml) echo "yaml" ;;
        json) echo "json" ;;
        toml) echo "toml" ;;
        html) echo "html" ;;
        css) echo "css" ;;
        sql) echo "sql" ;;
        *) echo "text" ;;
    esac
}

# Generate markdown for a directory
process_directory() {
    local dir_path="$1"
    local dir_name=$(basename "$dir_path")
    local output_file="${dir_path}/${dir_name}.md"
    
    if [[ "$VERBOSE" == true ]]; then
        log_info "Processing directory: $dir_path"
    fi
    
    # Collect files to process
    local files_to_process=()
    
    while IFS= read -r -d '' file; do
        if can_process_file "$file" && should_include_file "$file"; then
            files_to_process+=("$file")
        fi
    done < <(find "$dir_path" -type f -print0 2>/dev/null | sort -z)
    
    if [[ ${#files_to_process[@]} -eq 0 ]]; then
        log_warn "No files to process in $dir_path"
        return 0
    fi
    
    if [[ "$DRY_RUN" == true ]]; then
        log_info "[DRY RUN] Would create: $output_file"
        log_info "[DRY RUN] Files to include: ${#files_to_process[@]}"
        for file in "${files_to_process[@]}"; do
            log_info "[DRY RUN]   - $file"
        done
        return 0
    fi
    
    # Generate markdown content
    {
        echo "# $dir_name Code Documentation"
        echo ""
        echo "**Generated on:** $(date '+%Y-%m-%d %H:%M:%S')"
        echo "**Directory:** $(realpath "$dir_path")"
        echo "**Files included:** ${#files_to_process[@]}"
        echo ""
        
        # Add directory tree
        echo "---"
        echo ""
        echo "## Directory Structure"
        echo ""
        echo '```'
        (cd "$dir_path" && find . -type f | sort | head -50)
        if [[ ${#files_to_process[@]} -gt 50 ]]; then
            echo "... ($((${#files_to_process[@]} - 50)) more files)"
        fi
        echo '```'
        echo ""
        
        echo "---"
        echo ""
        echo "## File Contents"
        echo ""
        
        # Add file contents
        for file_path in "${files_to_process[@]}"; do
            local relative_path="${file_path#$dir_path/}"
            local language=$(get_language "$file_path")
            
            echo "### $relative_path"
            echo ""
            
            # Add file metadata
            local file_size=$(stat -f%z "$file_path" 2>/dev/null || stat -c%s "$file_path" 2>/dev/null || echo "unknown")
            local mod_time=$(stat -f%Sm -t%Y-%m-%d\ %H:%M:%S "$file_path" 2>/dev/null || stat -c%y "$file_path" 2>/dev/null || echo "unknown")
            echo "**Size:** $file_size bytes | **Modified:** $mod_time"
            echo ""
            
            # Add code block with syntax highlighting
            echo '```'"$language"
            cat "$file_path" || {
                log_warn "Could not read file: $file_path"
                echo "[Error: Could not read file content]"
            }
            echo '```'
            echo ""
            echo "---"
            echo ""
        done
        
        # Add footer
        echo ""
        echo "---"
        echo "*Generated by code2prompt.sh on $(date '+%Y-%m-%d %H:%M:%S')*"
        
    } > "$output_file"
    
    log_success "Created: $output_file (${#files_to_process[@]} files)"
}

# Main execution
main() {
    local directories=($(parse_args "$@"))
    
    if [[ ${#directories[@]} -eq 0 ]]; then
        log_error "No directories found to process"
        exit 1
    fi
    
    log_info "Starting code2prompt..."
    log_info "Include extensions: ${INCLUDE_EXTENSIONS[*]}"
    log_info "Exclude directories: ${EXCLUDE_DIRS[*]}"
    log_info "Max file size: $MAX_FILE_SIZE bytes"
    log_info "Recursive: $RECURSIVE"
    log_info "Dry run: $DRY_RUN"
    
    local total_dirs=0
    local total_files=0
    
    for dir in "${directories[@]}"; do
        if [[ ! -d "$dir" ]]; then
            log_error "Directory not found: $dir"
            continue
        fi
        
        if should_exclude_dir "$dir"; then
            log_warn "Excluding directory: $dir"
            continue
        fi
        
        if [[ "$RECURSIVE" == true ]]; then
            # Process all subdirectories recursively
            while IFS= read -r -d '' subdir; do
                if ! should_exclude_dir "$subdir"; then
                    process_directory "$subdir"
                    ((total_dirs++))
                fi
            done < <(find "$dir" -type d -print0 | sort -z)
        else
            # Process only the specified directory
            process_directory "$dir"
            ((total_dirs++))
        fi
    done
    
    if [[ "$DRY_RUN" == true ]]; then
        log_info "Dry run completed. Would process $total_dirs directories."
    else
        log_success "Completed! Processed $total_dirs directories."
    fi
}

# Run main function with all arguments
main "$@"