#!/bin/sh -e
## Tested with https://www.shellcheck.net/
# Usage: (install latest)
#   $ curl -fsSL https://raw.githubusercontent.com/you54f/pact-cli/main/install.sh | sh
# or
#   $ wget -q https://raw.githubusercontent.com/you54f/pact-cli/main/install.sh -O- | sh
#
# Usage: (install fixed version) - pass PACT_CLI_VERSION=v<PACT_CLI_VERSION> eg PACT_CLI_VERSION=v1.92.0 or set as an env var
#   $ curl -fsSL https://raw.githubusercontent.com/you54f/pact-cli/main/install.sh | PACT_CLI_VERSION=v1.92.0 sh
# or
#   $ wget -q https://raw.githubusercontent.com/you54f/pact-cli/main/install.sh -O- | PACT_CLI_VERSION=v1.92.0 sh
#
if [ "$tag" ]; then
  echo "setting $tag as PACT_CLI_VERSION for legacy reasons"
  PACT_CLI_VERSION="$tag"
fi

if command -v curl >/dev/null 2>&1; then
  downloader="curl -sLO --fail"
elif command -v wget >/dev/null 2>&1; then
  downloader="wget -q"
else
  echo "Sorry, you need either curl or wget installed to proceed with the installation."
  exit 1
fi

if [ -z "$PACT_CLI_VERSION" ]; then
  if command -v curl >/dev/null 2>&1; then
    PACT_CLI_VERSION=$(basename "$(curl -fs -o/dev/null -w "%{redirect_url}" https://github.com/you54f/pact-cli/releases/latest)")
  elif command -v wget >/dev/null 2>&1; then
    PACT_CLI_VERSION=$(basename "$(wget -q -S -O /dev/null https://github.com/you54f/pact-cli/releases/latest 2>&1 | grep -i "Location:" | awk '{print $2}')")
  else
    echo "Sorry, you need set a version number PACT_CLI_VERSION as we can't determine the latest at this time. See https://github.com/you54f/pact-cli/releases/latest."
    exit 1
  fi
  if [ -z "$PACT_CLI_VERSION" ]; then
    PACT_CLI_VERSION=vlatest
    echo "No version specified, defaulting to $PACT_CLI_VERSION"
  fi

  echo "Thanks for downloading the latest release of pact-cli $PACT_CLI_VERSION."
  echo "-----"
  echo "Note:"
  echo "-----"
  echo "You can download a fixed version by setting the PACT_CLI_VERSION environment variable eg PACT_CLI_VERSION=v1.92.0"
  echo "example:"
  echo "curl -fsSL https://raw.githubusercontent.com/you54f/pact-cli/master/install.sh | PACT_CLI_VERSION=v1.92.0 sh"
else
  echo "Thanks for downloading pact-cli $PACT_CLI_VERSION."
fi

PACT_CLI_VERSION_WITHOUT_V=${PACT_CLI_VERSION#v}
MAJOR_PACT_CLI_VERSION=$(echo "$PACT_CLI_VERSION_WITHOUT_V" | cut -d '.' -f 1)

case $(uname -sm) in
'Linux x86_64')
    if ldd /bin/ls >/dev/null 2>&1; then
        ldd_output=$(ldd /bin/ls)
        case "$ldd_output" in
            *musl*) 
                os='x86_64-linux-musl'
                ;;
            *) 
                os='x86_64-linux-gnu'
                ;;
        esac
    else
      os='x86_64-linux-gnu'
    fi
  ;;
'Linux aarch64')
  if ldd /bin/ls >/dev/null 2>&1; then
      ldd_output=$(ldd /bin/ls)
      case "$ldd_output" in
          *musl*) 
              os='aarch64-linux-musl'
              ;;
          *) 
              os='aarch64-linux-gnu'
              ;;
      esac
  else
    os='aarch64-linux-gnu'
  fi
  ;;
'Darwin arm64')
  os='aarch64-macos'
  ;;
'Darwin x86' | 'Darwin x86_64')
  os='x86_64-macos'
  ;;
"Windows"* | "MINGW64"*)
  if [ "$(uname -m)" = "aarch64" ]; then
    os='aarch64-windows-msvc'
  else
    os='x86_64-windows-msvc'
  fi
  ;;
*)
  echo "Sorry, you'll need to install the pact-cli manually."
  exit 1
  ;;
esac

case $os in
*'windows'*)
  filename="pact-cli-${os}.exe"
  ;;
*'macos'* | *'linux'*)
  filename="pact-cli-${os}"
  ;;
esac

PROJECT_NAME=pact-cli
echo 
echo "-------------"
echo "Downloading ${filename} - version ${PACT_CLI_VERSION}"
echo "-------------"
echo "Url: https://github.com/you54f/pact-cli/releases/download/${PACT_CLI_VERSION}/${filename}"
($downloader https://github.com/you54f/pact-cli/releases/download/"${PACT_CLI_VERSION}"/"${filename}" && echo downloaded "${filename}") || (echo "Failed to download pact-cli, check the version and url." && exit 1)
echo "$PROJECT_NAME ${PACT_CLI_VERSION} installed to $(pwd)"
echo "-------------------"
echo "available commands:"
echo "-------------------"
PACT_BROKER_CLI_BIN_PATH=${PWD}
if [[ "$filename" == *.exe ]]; then
  mv "$filename" "$PROJECT_NAME.exe"
  chmod +x "$PROJECT_NAME.exe"
else
  mv "$filename" "$PROJECT_NAME"
  chmod +x "$PROJECT_NAME"
fi
./pact-cli --help
echo "-------------------"
if [ "$GITHUB_ENV" ]; then
echo "Added the following to your path to make ${PROJECT_NAME} available:"
echo ""
echo "PATH=$PACT_BROKER_CLI_BIN_PATH:\${PATH}"
echo "PATH=$PACT_BROKER_CLI_BIN_PATH:${PATH}" >>"$GITHUB_ENV"
elif [ "$CIRRUS_CI" ]; then
echo "Added the following to your path to make ${PROJECT_NAME} available:"
echo ""
echo "PATH=$PACT_BROKER_CLI_BIN_PATH:\${PATH}"
echo "PATH=$PACT_BROKER_CLI_BIN_PATH:${PATH}" >>"$CIRRUS_ENV"
else
echo "Add the following to your path to make ${PROJECT_NAME} available:"
echo "--- Linux/MacOS/Windows Bash Users --------"
echo ""
echo "  PATH=$PACT_BROKER_CLI_BIN_PATH:\${PATH}"
fi
