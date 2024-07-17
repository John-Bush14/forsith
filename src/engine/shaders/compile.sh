#!/bin/bash

# Remove existing SPIR-V files
rm -f engine/shaders/shader.vert.spv
rm -f engine/shaders/shader.frag.spv

# Compile vertex shader
glslangValidator -V engine/shaders/shader.vert -o engine/shaders/shader.vert.spv
if [ $? -ne 0 ]; then
    echo "Failed to compile vertex shader."
    exit 1
fi

# Compile fragment shader
glslangValidator -V engine/shaders/shader.frag -o engine/shaders/shader.frag.spv
if [ $? -ne 0 ]; then
    echo "Failed to compile fragment shader."
    exit 1
fi

echo "Shaders compiled and renamed successfully."

