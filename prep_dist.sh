#!/bin/bash

if [ -z "$1" ]; then
    echo "Usage: $0 <target> [--binary-name <name>]"
    exit 1
fi

TARGET=$1
BINARY_NAME="pact"
OUTPUT_DIR="dist"

while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        --binary-name)
            BINARY_NAME="$2"
            shift
            shift
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift
            shift
            ;;
        *)
            shift
            ;;
    esac
done
# Rename targets to friendlier names end user names
DIST_TARGET_NAME=${TARGET}
DIST_TARGET_NAME=${DIST_TARGET_NAME//-unknown-/-}
DIST_TARGET_NAME=${DIST_TARGET_NAME//-pc-/-}
DIST_TARGET_NAME=${DIST_TARGET_NAME//-apple-darwin/-macos}

echo "DIST_TARGET_NAME: ${DIST_TARGET_NAME}"
mkdir -p ${OUTPUT_DIR}
## Proces executables
echo "Processing executables"
    cp target/${TARGET}/release/${BINARY_NAME} ${OUTPUT_DIR}/${BINARY_NAME}-${DIST_TARGET_NAME}

## Process shared libs
echo "Processing shared libraries"
for file in target/${TARGET}/release/*.{a,so,dll,dll.lib,dylib}; do
    echo "Processing $file for $DIST_TARGET_NAME"
    # Check if the file exists
    if [ ! -f "$file" ]; then
        echo "File $file does not exist. Skipping..."
        continue
    fi
    
    
    # get file extension
    extension="${file##*.}"
    # get filename without extension
    filename="${file%.*}"
    # remove both extensions if filename ends with .dll.lib
    if [[ $file == *".dll.lib" ]]; then
        filename="${filename%.*}"
        extension="dll.${extension}"
    fi
    DIST_TARGET_FILE="${filename}-${DIST_TARGET_NAME}.${extension}"
    echo "Renaming $file to $DIST_TARGET_FILE"
    cp "$file" "${DIST_TARGET_FILE}"
    # get full filename without base path
    new_file_name="${DIST_TARGET_FILE##*/}"
    mv "${DIST_TARGET_FILE}" "${OUTPUT_DIR}/${new_file_name}"
done

# Check if files exist in dist folder
echo "Checking dist folder for files"
if [ ! -f "${OUTPUT_DIR}/${BINARY_NAME}-${DIST_TARGET_NAME}" ]; then
    echo "Error: ${BINARY_NAME}-${DIST_TARGET_NAME} does not exist in ${OUTPUT_DIR}"
    exit 1
fi

for file in ${OUTPUT_DIR}/*; do
    if [ ! -f "$file" ]; then
        echo "Error: $file does not exist in ${OUTPUT_DIR}"
        exit 1
    fi

    if [[ "$OSTYPE" == "darwin"* ]]; then
        if [[ "$file" == *"${DIST_TARGET_NAME}.dylib" ]] || [[ "$file" == *"${DIST_TARGET_NAME}.a" ]]; then
            echo "Found dylib file: $file"
        fi
    elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
        if [[ "$file" == *"${DIST_TARGET_NAME}.dll" || "$file" == *"${DIST_TARGET_NAME}.dll.lib" ]]; then
            echo "Found DLL file: $file"
        fi
    elif [[ "$OSTYPE" == "linux"* ]]; then
        if [[ "$file" == *"${DIST_TARGET_NAME}.so" ]] || [[ "$file" == *"${DIST_TARGET_NAME}.a" ]]; then
            echo "Found shared library file: $file"
        fi
    fi
done

echo DIST_TARGET_NAME=${DIST_TARGET_NAME} >> $GITHUB_ENV