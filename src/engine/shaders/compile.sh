#!/bin/bash

# Directory to search for shader files
SCRIPT_DIR="${1:-.}"

# Find and delete all .spv files
find "$SCRIPT_DIR" -type f -name "*.spv" -exec rm -v {} +

# Function to compile shaders
compile_shader() {
    local shader_file="$1"

    local spv_file="${shader_file}.spv"
    
    # Compile shader using glslangValidator
    glslangValidator -V "$shader_file" -o "$spv_file"
    
    if [ $? -eq 0 ]; then
        echo "Compiled: $shader_file -> $spv_file"
    else
        echo "Failed to compile: $shader_file"
    fi
}

# Export the function so it can be used by find's -exec option
export -f compile_shader

# Find and compile shader files (excluding .spv files)
find "$SCRIPT_DIR" -type f ! -name "*.spv" ! -name "*.sh" -exec bash -c 'compile_shader "$0"' {} \;

echo "Shader compilation complete."
