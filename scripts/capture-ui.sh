#!/usr/bin/env sh
set -eu

out="target/ui-shots/studio.png"
width="1320"
height="780"
debug_overlay="0"
root_id=""
workspace_id=""
project_id=""
group_id=""
flow_id=""
lab_view=""
scroll_x=""
scroll_y=""

while [ "$#" -gt 0 ]; do
    case "$1" in
        --out)
            out="$2"
            shift 2
            ;;
        --width)
            width="$2"
            shift 2
            ;;
        --height)
            height="$2"
            shift 2
            ;;
        --debug-overlay)
            debug_overlay="1"
            shift
            ;;
        --root-id)
            root_id="$2"
            shift 2
            ;;
        --workspace-id)
            workspace_id="$2"
            shift 2
            ;;
        --project-id)
            project_id="$2"
            shift 2
            ;;
        --group-id)
            group_id="$2"
            shift 2
            ;;
        --flow-id)
            flow_id="$2"
            shift 2
            ;;
        --lab-view)
            lab_view="$2"
            shift 2
            ;;
        --scroll-x)
            scroll_x="$2"
            shift 2
            ;;
        --scroll-y)
            scroll_y="$2"
            shift 2
            ;;
        -h|--help)
            printf '%s\n' "Usage: scripts/capture-ui.sh [--out PATH] [--width PX] [--height PX] [--debug-overlay] [--lab-view NAME]"
            exit 0
            ;;
        *)
            printf '%s\n' "Unknown argument: $1" >&2
            exit 2
            ;;
    esac
done

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)

case "$out" in
    /*) output_path="$out" ;;
    *) output_path="$repo_root/$out" ;;
esac

output_dir=$(dirname -- "$output_path")
mkdir -p "$output_dir"
rm -f "$output_path"

export EFRAME_SCREENSHOT_TO="$output_path"
export DES_UI_HARNESS_WIDTH="$width"
export DES_UI_HARNESS_HEIGHT="$height"
export DES_UI_HARNESS_TITLE="Data Engine Studio UI Harness"

if [ "$debug_overlay" = "1" ]; then
    export DES_UI_DEBUG_OVERLAY="1"
fi
if [ -n "$root_id" ]; then
    export DES_UI_SELECTED_ROOT="$root_id"
fi
if [ -n "$workspace_id" ]; then
    export DES_UI_SELECTED_WORKSPACE="$workspace_id"
fi
if [ -n "$project_id" ]; then
    export DES_UI_SELECTED_PROJECT="$project_id"
fi
if [ -n "$group_id" ]; then
    export DES_UI_SELECTED_GROUP="$group_id"
fi
if [ -n "$flow_id" ]; then
    export DES_UI_SELECTED_FLOW="$flow_id"
fi
if [ -n "$lab_view" ]; then
    export DES_UI_LAB_VIEW="$lab_view"
fi
if [ -n "$scroll_x" ]; then
    export DES_UI_LAB_SCROLL_X="$scroll_x"
fi
if [ -n "$scroll_y" ]; then
    export DES_UI_LAB_SCROLL_Y="$scroll_y"
fi

cargo run -p des-ui-lab --features ui-screenshot --bin des-ui-shot

if [ ! -f "$output_path" ]; then
    printf '%s\n' "UI screenshot was not created: $output_path" >&2
    exit 1
fi

printf '%s\n' "$output_path"
