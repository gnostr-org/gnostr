#!/usr/bin/env bash
# scripts/recursive-help.sh
# Recursively parses gnostr command help and outputs structured JSON

set -euo pipefail

# Configuration defaults
BASE_COMMAND="./target/debug/gnostr"
MAX_DEPTH=10
OUTPUT_FILE=""
PRETTY_PRINT=false
VALIDATE_JSON=false
BUILD_FIRST=false

# Global storage for command data
# Using temp files for associative array compatibility on older bash
TEMP_DIR=$(mktemp -d)
COMMANDS_FILE="$TEMP_DIR/commands.json"
PARSED_PATHS_FILE="$TEMP_DIR/paths.txt"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $*" >&2; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*" >&2; }
log_error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $*" >&2; }

# Usage information
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Recursively parses gnostr command help and outputs structured JSON.

Options:
  --output FILE     Output JSON file (default: stdout)
  --max-depth N     Maximum recursion depth (default: 10)
  --base-cmd CMD    Base command (default: ./target/debug/gnostr)
  --pretty          Pretty-print JSON
  --validate        Validate JSON output
  --build           Build gnostr before parsing
  --help            Show this help

Examples:
  $0                         # Parse ./target/debug/gnostr
  $0 --build                 # Build then parse
  $0 --output help.json      # Save to file
  $0 --base-cmd gnostr       # Use system gnostr
  $0 --pretty --validate     # Pretty print and validate

EOF
}

# Parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --output)
                OUTPUT_FILE="$2"
                shift 2
                ;;
            --max-depth)
                MAX_DEPTH="$2"
                shift 2
                ;;
            --base-cmd)
                BASE_COMMAND="$2"
                shift 2
                ;;
            --pretty)
                PRETTY_PRINT=true
                shift
                ;;
            --validate)
                VALIDATE_JSON=true
                shift
                ;;
            --build)
                BUILD_FIRST=true
                shift
                ;;
            --help)
                usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
}

# Build gnostr if requested
build_gnostr() {
    log_info "Building gnostr in debug mode..."
    if cargo build; then
        log_success "Build completed successfully"
        return 0
    else
        log_error "Build failed"
        return 1
    fi
}

# Validate gnostr binary exists and works
validate_gnostr_binary() {
    local gnostr_path="$1"
    
    # Check if command exists in PATH or as absolute/relative path
    if ! command -v "$gnostr_path" >/dev/null 2>&1 && [ ! -f "$gnostr_path" ]; then
        log_error "gnostr binary not found at: $gnostr_path"
        log_info "Run 'cargo build' first to create the debug binary"
        return 1
    fi
    
    # Check if file is executable (skip for commands in PATH)
    if [ -f "$gnostr_path" ] && [ ! -x "$gnostr_path" ]; then
        log_warn "gnostr binary is not executable: $gnostr_path"
        if ! chmod +x "$gnostr_path"; then
            log_error "Failed to make gnostr binary executable"
            return 1
        fi
    fi
    
    # Test that it responds to --help (ignore exit code, just check if it produces output)
    local help_output
    if [ -n "${TIMEOUT_CMD:-}" ]; then
        help_output=$("$TIMEOUT_CMD" 5s "$gnostr_path" --help 2>&1 || true)
    else
        help_output=$("$gnostr_path" --help 2>&1 || true)
    fi
    
    if [ -z "$help_output" ]; then
        log_error "gnostr --help produced no output: $gnostr_path"
        return 1
    fi
    
    log_success "Using gnostr binary: $gnostr_path"
    return 0
}

# Validate environment and dependencies
validate_environment() {
    # Build if requested
    if [ "$BUILD_FIRST" = true ]; then
        build_gnostr || return 1
    fi
    
    # Validate the binary exists and works
    validate_gnostr_binary "$BASE_COMMAND" || return 1
    
    # Check dependencies
    if ! command -v jq >/dev/null 2>&1; then
        log_error "jq is required but not installed"
        log_info "Install with: brew install jq (macOS) or apt install jq (Ubuntu)"
        return 1
    fi
    
    # Check timeout command (optional)
    if command -v timeout >/dev/null 2>&1; then
        TIMEOUT_CMD="timeout"
    elif command -v gtimeout >/dev/null 2>&1; then
        TIMEOUT_CMD="gtimeout"
    else
        log_warn "timeout command not found, commands may hang"
        TIMEOUT_CMD=""
    fi
}

# Extract command names from help output
extract_command_names() {
    local help_output="$1"
    
    echo "$help_output" | awk '
        /^Commands:/ {
            in_commands = 1
            next
        }
        /^Options:/ || /^Arguments:/ || /^$/ {
            if (in_commands) exit
        }
        in_commands && /^[[:space:]]+[a-zA-Z]/ {
            # Extract command name (first word)
            if (match($0, /^[[:space:]]+([a-zA-Z][a-zA-Z0-9_-]*)/)) {
                name_start = RSTART + 1
                name_end = RSTART + RLENGTH - 1
                # Find the end of the command name
                while (name_end <= length($0) && substr($0, name_end, 1) ~ /[a-zA-Z0-9_-]/) {
                    name_end++
                }
                name_end--  # Adjust back to last valid character
                name = substr($0, name_start, name_end - name_start + 1)
                if (name != "") print name
            }
        }
    '
}

# Detect command type based on help output
detect_command_type() {
    local help_output="$1"
    
    if echo "$help_output" | grep -q "^Commands:"; then
        echo "container_command"
    else
        echo "leaf_command"
    fi
}

# Parse options from help output
parse_options() {
    local help_output="$1"
    
    echo "$help_output" | awk '
        /^Options:/ {
            in_options = 1
            next
        }
        /^Arguments:/ || /^$/ {
            if (in_options) exit
        }
        in_options && /^[[:space:]]+-/ {
            line = $0
            # Extract long option
            if (match(line, /--[a-zA-Z][a-zA-Z0-9_-]*/)) {
                name = substr(line, RSTART + 2, RLENGTH - 2)
                
                # Extract short option
                short = ""
                if (match(line, /-[a-zA-Z]/)) {
                    short = substr(line, RSTART, RLENGTH)
                }
                
                # Extract type and description
                desc = ""
                type = "String"
                if (match(line, /<[A-Za-z][A-Za-z0-9_]*>/)) {
                    type = substr(line, RSTART + 1, RLENGTH - 2)
                }
                
                # Extract description after option
                if (match(line, />[[:space:]]+/)) {
                    desc = substr(line, RSTART + RLENGTH)
                    gsub(/^[[:space:]]+|[[:space:]]+$/, "", desc)
                }
                
                # Handle default values
                default_val = ""
                if (match(desc, /\[default: [^]]+\]/)) {
                    default_val = substr(desc, RSTART, RLENGTH)
                    # Remove from description
                    gsub(/\[default: [^]]+\][[:space:]]*/, "", desc)
                }
                
                if (short != "") {
                    printf "{\"name\":\"%s\",\"short\":\"%s\",\"type\":\"%s\",\"description\":\"%s\"", name, short, type, desc
                } else {
                    printf "{\"name\":\"%s\",\"type\":\"%s\",\"description\":\"%s\"", name, type, desc
                }
                
                if (default_val != "") {
                    printf ",\"default\":\"%s\"", default_val
                }
                print "}"
            }
        }
    ' | jq -s '.' 2>/dev/null || echo "[]"
}

# Parse arguments from help output
parse_command_arguments() {
    local help_output="$1"
    
    echo "$help_output" | awk '
        /^Arguments:/ {
            in_arguments = 1
            next
        }
        /^Options:/ || /^$/ {
            if (in_arguments) exit
        }
        in_arguments && /^[[:space:]]+\[/ {
            line = $0
            # Extract argument name using simpler regex
            if (match(line, /\[[A-Z_][A-Z0-9_]*\]/)) {
                arg_start = RSTART + 1
                arg_end = RSTART + RLENGTH - 2
                name = substr(line, arg_start, arg_end - arg_start + 1)
                variadic = (index(line, "...") > 0)
                
                # Extract description
                desc = ""
                if (match(line, /\][[:space:]]+/)) {
                    desc = substr(line, RSTART + RLENGTH)
                    gsub(/^[[:space:]]+|[[:space:]]+$/, "", desc)
                }
                
                printf "{\"name\":\"%s\",\"variadic\":%s,\"description\":\"%s\"}", name, $variadic, desc
            }
        }
    ' | jq -s '.' 2>/dev/null || echo "[]"
}

# Extract description from help output
extract_description() {
    local help_output="$1"
    
    echo "$help_output" | awk '
        /^Usage:/ { skip_next = 1; next }
        skip_next && /^$/ { skip_next = 0; next }
        !skip_next && NF > 0 && !/^Commands:/ && !/^Options:/ && !/^Arguments:/ && !/^[[:space:]]+-/ {
            print $0
            exit
        }
    ' | sed 's/^[[:space:]]*//' | head -1
}

# Extract usage line from help output
extract_usage() {
    local help_output="$1"
    
    echo "$help_output" | grep "^Usage:" | cut -d: -f2- | sed 's/^[[:space:]]*//' || echo ""
}

# Parse help output into JSON structure
parse_help_output() {
    local help_output="$1"
    local command_path="$2"
    
    # Extract command name (last element of path)
    local cmd_name=$(echo "$command_path" | jq -r '.[-1]')
    
    # Parse various components
    local description=$(extract_description "$help_output")
    local usage=$(extract_usage "$help_output")
    local command_type=$(detect_command_type "$help_output")
    local options=$(parse_options "$help_output")
    local arguments=$(parse_command_arguments "$help_output")
    
    # Create JSON object
    jq -n \
        --arg name "$cmd_name" \
        --argjson path "$command_path" \
        --arg description "$description" \
        --arg usage "$usage" \
        --arg type "$command_type" \
        --argjson options "$options" \
        --argjson arguments "$arguments" \
        '{
            name: $name,
            path: $path,
            description: $description,
            usage: $usage,
            type: $type,
            options: $options,
            arguments: $arguments,
            subcommands: []
        }'
}

# Store command JSON in global storage
store_command_json() {
    local path_key="$1"
    local command_json="$2"
    
    echo "$path_key|$command_json" >> "$COMMANDS_FILE"
}

# Recursive parsing function
recursive_parse() {
    local base_command="$1"
    local current_path="$2" 
    local depth="$3"
    
    # Create path key for storage
    local path_key=$(echo "$current_path" | jq -r 'join("/")')
    
    # Avoid infinite loops and duplicates
    if grep -q "^$path_key$" "$PARSED_PATHS_FILE" 2>/dev/null; then
        log_warn "Already parsed: $base_command"
        return
    fi
    echo "$path_key" >> "$PARSED_PATHS_FILE"
    
    log_info "Parsing [$depth]: $base_command"
    
    # Depth limiting
    if [ "$depth" -gt "$MAX_DEPTH" ]; then
        log_warn "Maximum depth ($MAX_DEPTH) reached at: $base_command"
        return
    fi
    
    # Get help output with timeout if available
    local help_output
    if [ -n "${TIMEOUT_CMD:-}" ]; then
        help_output=$("$TIMEOUT_CMD" 10s "$base_command" --help 2>/dev/null || true)
    else
        help_output=$("$base_command" --help 2>/dev/null || true)
    fi
    
    if [ -n "$help_output" ]; then
        # Parse and store JSON for this command
        local command_json=$(parse_help_output "$help_output" "$current_path")
        store_command_json "$path_key" "$command_json"
        
        # If has subcommands, recurse
        if echo "$help_output" | grep -q "^Commands:"; then
            local subcommands=($(extract_command_names "$help_output"))
            log_info "Found ${#subcommands[@]} subcommands for: $base_command"
            
            for subcmd in "${subcommands[@]}"; do
                # Skip 'help' commands as they're usually recursive
                if [ "$subcmd" = "help" ]; then
                    continue
                fi
                
                local new_base="$base_command $subcmd"
                local new_path=$(echo "$current_path" | jq --arg subcmd "$subcmd" '. + [$subcmd]')
                recursive_parse "$new_base" "$new_path" $((depth + 1))
            done
        fi
    else
        log_warn "Failed to get help for: $base_command (timeout or error)"
    fi
}

# Get version information
get_gnostr_version() {
    local gnostr_path="$1"
    "$gnostr_path" --version 2>/dev/null | head -1 || echo "unknown"
}

# Get git commit hash for build info
get_git_commit() {
    git rev-parse --short HEAD 2>/dev/null || echo "unknown"
}

# Generate final JSON output
generate_final_json() {
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    local version=$(get_gnostr_version "$BASE_COMMAND")
    local git_commit=$(get_git_commit)
    local build_info=""
    
    # Build info JSON
    if [ "$BUILD_FIRST" = true ]; then
        build_info=$(jq -n --arg built_at "$timestamp" '{
            debug: true,
            built_at: $built_at,
            cargo_build: true
        }')
    else
        build_info=$(jq -n '{
            debug: true,
            cargo_build: false
        }')
    fi
    
    # Parse config JSON
    local parse_config=$(jq -n \
        --arg max_depth "$MAX_DEPTH" \
        --arg base_cmd "$BASE_COMMAND" \
        '{
            max_depth: ($max_depth | tonumber),
            base_command: $base_cmd
        }')
    
    # Create base JSON structure
    local base_json=$(jq -n \
        --arg tool "gnostr" \
        --arg base_cmd "$BASE_COMMAND" \
        --arg version "$version" \
        --arg timestamp "$timestamp" \
        --argjson build_info "$build_info" \
        --argjson parse_config "$parse_config" \
        '{
            tool: $tool,
            binary_path: $base_cmd,
            version: $version,
            build_info: $build_info,
            generated_at: $timestamp,
            parsing_config: $parse_config,
            commands: []
        }')
    
    # Merge all stored commands
    while IFS='|' read -r path_key command_json; do
        base_json=$(echo "$base_json" | jq --argjson cmd "$command_json" '.commands += [$cmd]')
    done < "$COMMANDS_FILE"
    
    # Build command hierarchy by linking subcommands
    echo "$base_json" | jq '
        def link_subcommands:
            .commands |= map(
                . as $cmd
                | .subcommands = (
                    .commands
                    | map(select(.path[:-1] == $cmd.path))
                    | map(.name)
                )
            );
        link_subcommands
    '
}

# Validate JSON output
validate_json() {
    local json_content="$1"
    if echo "$json_content" | jq empty 2>/dev/null; then
        log_success "JSON validation: PASSED"
        return 0
    else
        log_error "JSON validation: FAILED"
        echo "$json_content" | jq . 2>&1 >&2
        return 1
    fi
}

# Cleanup function
cleanup() {
    rm -rf "$TEMP_DIR" 2>/dev/null || true
}

# Set trap for cleanup
trap cleanup EXIT

# Main execution function
main() {
    parse_arguments "$@"
    
    # Initialize temp files
    > "$COMMANDS_FILE"
    > "$PARSED_PATHS_FILE"
    
    # Validate environment
    log_info "Validating environment..."
    validate_environment || exit 1
    
    # Start recursive parsing
    log_info "Starting recursive help parsing..."
    recursive_parse "$BASE_COMMAND" '["gnostr"]' 0
    
    log_info "Parsed $(wc -l < "$COMMANDS_FILE") commands"
    
    # Generate final JSON
    log_info "Generating JSON output..."
    local json_output
    json_output=$(generate_final_json)
    
    # Validation if requested
    if [ "$VALIDATE_JSON" = true ]; then
        validate_json "$json_output" || exit 1
    fi
    
    # Pretty printing if requested
    if [ "$PRETTY_PRINT" = true ]; then
        json_output=$(echo "$json_output" | jq .)
    fi
    
    # Output result
    if [ -n "$OUTPUT_FILE" ]; then
        echo "$json_output" > "$OUTPUT_FILE"
        log_success "JSON help tree written to: $OUTPUT_FILE"
        log_info "Total commands parsed: $(wc -l < "$COMMANDS_FILE")"
    else
        echo "$json_output"
    fi
}

# Run main function with all arguments
main "$@"
