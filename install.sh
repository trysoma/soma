#!/bin/sh

# This script is for installing the latest version of Soma on your machine.
# Usage:
#   curl -sSL https://raw.githubusercontent.com/trysoma/soma/main/install.sh | sh
#   curl -sSL https://raw.githubusercontent.com/trysoma/soma/main/install.sh | sh -s -- 0.0.1
#   VERSION=0.0.1-snapshot-2 sh install.sh

set -e

# Terminal ANSI escape codes.
reset="\033[0m"
bright_blue="${reset}\033[34;1m"
bright_green="${reset}\033[32;1m"
bright_yellow="${reset}\033[33;1m"

# Parse version from arguments or environment variable
VERSION="${VERSION:-${1:-latest}}"

probe_arch() {
    ARCH=$(uname -m)
    case $ARCH in
        x86_64) ARCH="x86_64"  ;;
        aarch64) ARCH="aarch64" ;;
        arm64) ARCH="aarch64" ;;
        *) printf "Architecture ${ARCH} is not supported by this installation script\n"; exit 1 ;;
    esac
}

probe_os() {
    OS=$(uname -s)
    case $OS in
        Darwin) OS="apple-darwin" ;;
        Linux) OS="unknown-linux-gnu" ;;
        *) printf "Operating system ${OS} is not supported by this installation script\n"; exit 1 ;;
    esac
}

detect_profile() {
  local DETECTED_PROFILE
  DETECTED_PROFILE=''
  local SHELLTYPE
  SHELLTYPE="$(basename "/$SHELL")"

  if [ "$SHELLTYPE" = "bash" ]; then
    if [ -f "$HOME/.bashrc" ]; then
      DETECTED_PROFILE="$HOME/.bashrc"
    elif [ -f "$HOME/.bash_profile" ]; then
      DETECTED_PROFILE="$HOME/.bash_profile"
    fi
  elif [ "$SHELLTYPE" = "zsh" ]; then
    DETECTED_PROFILE="${ZDOTDIR:-$HOME}/.zshrc"
  elif [ "$SHELLTYPE" = "fish" ]; then
    DETECTED_PROFILE="$HOME/.config/fish/conf.d/soma.fish"
  fi

  if [ -z "$DETECTED_PROFILE" ]; then
    if [ -f "$HOME/.profile" ]; then
      DETECTED_PROFILE="$HOME/.profile"
    elif [ -f "$HOME/.bashrc" ]; then
      DETECTED_PROFILE="$HOME/.bashrc"
    elif [ -f "$HOME/.bash_profile" ]; then
      DETECTED_PROFILE="$HOME/.bash_profile"
    elif [ -f "${ZDOTDIR:-$HOME}/.zshrc" ]; then
      DETECTED_PROFILE="${ZDOTDIR:-$HOME}/.zshrc"
    elif [ -d "$HOME/.config/fish" ]; then
      DETECTED_PROFILE="$HOME/.config/fish/conf.d/soma.fish"
    fi
  fi

  if [ ! -z "$DETECTED_PROFILE" ]; then
    echo "$DETECTED_PROFILE"
  fi
}

update_profile() {
   PROFILE_FILE=$(detect_profile)
   if [ -n "$PROFILE_FILE" ]; then
     if ! grep -q "\.soma/bin" $PROFILE_FILE; then
        printf "\n${bright_blue}Updating profile ${reset}$PROFILE_FILE\n"
        printf "\n# Soma\nexport PATH=\"\$PATH:$INSTALL_DIRECTORY\"\n" >> $PROFILE_FILE
        printf "\nSoma will be available when you open a new terminal.\n"
        printf "If you want to make Soma available in this terminal, please run:\n"
        printf "\n${bright_blue}source $PROFILE_FILE${reset}\n"
     fi
   else
     printf "\n${bright_yellow}Unable to detect profile file location. ${reset}Please add the following to your profile file:\n"
     printf "\nexport PATH=\"$INSTALL_DIRECTORY:\$PATH\"\n"
   fi
}

print_logo() {
    printf "${bright_blue}

                                                                                                    
         ######             ##########                    ###          ###              %#####      
     ######  ######%     ###############     ######## ##########   ##########      #######  #####   
    ###         ####   #####        #######    #######      ########     #####     ####       ##### 
   ####%         ##  ######           ######   ######       #####        #####     ##         ##### 
   ########      ## ####               #####   #####        #####        #####    ##        ####### 
    ##########   %#####                 ####   #####        #####       ######    ##  ###### ###### 
     ############  ###          ######  ####   #####       #####        #####       #####    #####  
        ########## ###          ######  ####  #####        #####        #####     #####      #####  
           ####### ###         ####### #####  #####        #####        #####   #####        #####  
  #           #########               #####   #####        #####       #####    #####       #####   
 ##           #### #######        ########   ######       #####        #####   #####        #####   
 ###         ####   #####################    ######       #####        #####   #####       ######   
 ####      #####      #################     #######     ########     ########   ####### ##  ####### 
 ############            ##########       ###########  ##########  ###########   ########   ####    

${reset}
"
}

install_completions() {
  local SOMA_BIN="$INSTALL_DIRECTORY/soma"
  local SHELLTYPE
  SHELLTYPE="$(basename "/$SHELL")"

  printf "\n${bright_blue}Installing shell completions...${reset}\n"

  # Generate and install bash completions
  if [ "$SHELLTYPE" = "bash" ] || command -v bash >/dev/null 2>&1; then
    # Try user-level completion directory first
    if [ -d "$HOME/.local/share/bash-completion/completions" ]; then
      BASH_COMP_DIR="$HOME/.local/share/bash-completion/completions"
      mkdir -p "$BASH_COMP_DIR"
    elif [ -d "$HOME/.bash_completion.d" ]; then
      BASH_COMP_DIR="$HOME/.bash_completion.d"
    elif [ -w "/usr/local/etc/bash_completion.d" ]; then
      BASH_COMP_DIR="/usr/local/etc/bash_completion.d"
    elif [ -w "/etc/bash_completion.d" ]; then
      BASH_COMP_DIR="/etc/bash_completion.d"
    else
      # Fallback to user directory
      BASH_COMP_DIR="$HOME/.local/share/bash-completion/completions"
      mkdir -p "$BASH_COMP_DIR"
    fi

    if "$SOMA_BIN" completions bash > "$BASH_COMP_DIR/soma" 2>/dev/null; then
      printf "  ${bright_green}✓${reset} Bash completions installed to $BASH_COMP_DIR/soma\n"
    else
      printf "  ${bright_yellow}⚠${reset} Could not generate bash completions (soma completions bash failed)\n"
    fi
  fi

  # Generate and install zsh completions
  if [ "$SHELLTYPE" = "zsh" ] || command -v zsh >/dev/null 2>&1; then
    # Try user-level completion directory
    if [ -d "${ZDOTDIR:-$HOME}/.zsh/completions" ]; then
      ZSH_COMP_DIR="${ZDOTDIR:-$HOME}/.zsh/completions"
    elif [ -d "$HOME/.zsh/completions" ]; then
      ZSH_COMP_DIR="$HOME/.zsh/completions"
    elif [ -w "/usr/local/share/zsh/site-functions" ]; then
      ZSH_COMP_DIR="/usr/local/share/zsh/site-functions"
    else
      # Fallback to user directory
      ZSH_COMP_DIR="$HOME/.zsh/completions"
      mkdir -p "$ZSH_COMP_DIR"
    fi

    if "$SOMA_BIN" completions zsh > "$ZSH_COMP_DIR/_soma" 2>/dev/null; then
      printf "  ${bright_green}✓${reset} Zsh completions installed to $ZSH_COMP_DIR/_soma\n"

      # Add fpath to .zshrc if not already present
      ZSHRC="${ZDOTDIR:-$HOME}/.zshrc"
      if [ -f "$ZSHRC" ] && [ "$ZSH_COMP_DIR" = "$HOME/.zsh/completions" ]; then
        if ! grep -q "fpath=($HOME/.zsh/completions" "$ZSHRC"; then
          printf "\n# Soma completions\nfpath=($HOME/.zsh/completions \$fpath)\nautoload -Uz compinit && compinit\n" >> "$ZSHRC"
          printf "  ${bright_blue}Note:${reset} Added completion path to $ZSHRC\n"
        fi
      fi
    else
      printf "  ${bright_yellow}⚠${reset} Could not generate zsh completions (soma completions zsh failed)\n"
    fi
  fi

  # Generate and install fish completions
  if [ "$SHELLTYPE" = "fish" ] || command -v fish >/dev/null 2>&1; then
    FISH_COMP_DIR="$HOME/.config/fish/completions"
    mkdir -p "$FISH_COMP_DIR"

    if "$SOMA_BIN" completions fish > "$FISH_COMP_DIR/soma.fish" 2>/dev/null; then
      printf "  ${bright_green}✓${reset} Fish completions installed to $FISH_COMP_DIR/soma.fish\n"
    else
      printf "  ${bright_yellow}⚠${reset} Could not generate fish completions (soma completions fish failed)\n"
    fi
  fi
}

install_soma() {
  print_logo
  printf "\n"

  # Map architecture and OS to release asset naming convention
  # Convert from Rust target triple to GitHub release asset names
  case "${OS}" in
    apple-darwin)
      OS_NAME="macos"
      ;;
    unknown-linux-gnu)
      OS_NAME="linux"
      ;;
    *)
      OS_NAME="${OS}"
      ;;
  esac

  ASSET_NAME="soma-${OS_NAME}-${ARCH}"
  TARGET="${ARCH}-${OS}"  # Keep original for display

  # GitHub repository details
  GITHUB_REPO="trysoma/soma"

  # Determine download URL based on version
  if [ "$VERSION" = "latest" ]; then
    URL_PREFIX="https://github.com/${GITHUB_REPO}/releases/latest/download"
    VERSION_DISPLAY="latest"
  else
    URL_PREFIX="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}"
    VERSION_DISPLAY="v${VERSION}"
  fi

  # Remove existing soma binary if it exists
  if [ -f "$INSTALL_DIRECTORY/soma" ]; then
    printf "${bright_blue}Removing existing Soma installation...${reset}\n"
    rm -f "$INSTALL_DIRECTORY/soma"
  fi

  printf "${bright_blue}Downloading ${reset}Soma ${VERSION_DISPLAY} for $TARGET ...\n"

  DOWNLOAD_FILE=$(mktemp -t soma.XXXXXXXXXX)

  # Try primary asset naming convention (soma-linux-x86_64, soma-macos-aarch64, etc.)
  BINARY_NAME="${ASSET_NAME}"
  URL="${URL_PREFIX}/${BINARY_NAME}"

  if ! curl --fail --progress-bar -L "$URL" -o "$DOWNLOAD_FILE" 2>/dev/null; then
    # Try with .tar.gz extension
    BINARY_NAME="${ASSET_NAME}.tar.gz"
    URL="${URL_PREFIX}/${BINARY_NAME}"

    if ! curl --fail --progress-bar -L "$URL" -o "$DOWNLOAD_FILE" 2>/dev/null; then
      # Try Rust target triple format
      BINARY_NAME="soma-${TARGET}"
      URL="${URL_PREFIX}/${BINARY_NAME}"

      if ! curl --fail --progress-bar -L "$URL" -o "$DOWNLOAD_FILE" 2>/dev/null; then
        # Try Rust target triple with .tar.gz
        BINARY_NAME="soma-${TARGET}.tar.gz"
        URL="${URL_PREFIX}/${BINARY_NAME}"

        if ! curl --fail --progress-bar -L "$URL" -o "$DOWNLOAD_FILE" 2>/dev/null; then
          printf "\n${bright_yellow}Error: Could not download Soma binary for your platform.${reset}\n"
          printf "Tried the following URLs:\n"
          printf "  - ${URL_PREFIX}/${ASSET_NAME}\n"
          printf "  - ${URL_PREFIX}/${ASSET_NAME}.tar.gz\n"
          printf "  - ${URL_PREFIX}/soma-${TARGET}\n"
          printf "  - ${URL_PREFIX}/soma-${TARGET}.tar.gz\n"
          printf "\nPlease visit https://github.com/${GITHUB_REPO}/releases\n"
          printf "to download and install manually, or build from source.\n"
          rm -f "$DOWNLOAD_FILE"
          exit 1
        fi
      fi
    fi
  fi

  printf "\n${bright_blue}Installing to ${reset}$INSTALL_DIRECTORY\n"
  mkdir -p $INSTALL_DIRECTORY

  # Check if the download is a tar.gz archive or a raw binary
  if file "$DOWNLOAD_FILE" | grep -q "gzip compressed"; then
    tar -C $INSTALL_DIRECTORY -zxf $DOWNLOAD_FILE
    # Find the soma binary in the extracted files
    if [ ! -f "$INSTALL_DIRECTORY/soma" ]; then
      # Try to find it in a subdirectory
      SOMA_BIN=$(find $INSTALL_DIRECTORY -name "soma" -type f 2>/dev/null | head -n 1)
      if [ -n "$SOMA_BIN" ]; then
        mv "$SOMA_BIN" "$INSTALL_DIRECTORY/soma"
        # Clean up any extracted directories
        find $INSTALL_DIRECTORY -mindepth 1 -maxdepth 1 -type d -exec rm -rf {} \; 2>/dev/null || true
      fi
    fi
  else
    # Assume it's a raw binary
    mv "$DOWNLOAD_FILE" "$INSTALL_DIRECTORY/soma"
  fi

  # Make sure the binary is executable
  chmod +x "$INSTALL_DIRECTORY/soma"

  rm -f "$DOWNLOAD_FILE"

  # Verify installation
  if [ ! -f "$INSTALL_DIRECTORY/soma" ]; then
    printf "\n${bright_yellow}Error: Installation failed. Binary not found at $INSTALL_DIRECTORY/soma${reset}\n"
    exit 1
  fi

  # Verify the binary works
  if ! "$INSTALL_DIRECTORY/soma" --version >/dev/null 2>&1; then
    printf "\n${bright_yellow}Warning: Installed binary may not be compatible with your system.${reset}\n"
  fi
}

# do everything in main, so that partial downloads of this file don't mess up the installation
main() {
  printf "\nWelcome to the Soma installer!\n"

  if [ "$VERSION" != "latest" ]; then
    printf "Installing version: ${bright_blue}${VERSION}${reset}\n"
  fi
  printf "\n"

  probe_arch
  probe_os

  INSTALL_DIRECTORY="$HOME/.soma/bin"
  install_soma
  install_completions
  update_profile

  # Get installed version
  INSTALLED_VERSION=$("$INSTALL_DIRECTORY/soma" --version 2>/dev/null | head -n 1 || echo "unknown")

  printf "\n${bright_green}✓ Soma installed successfully!${reset}\n"
  printf "  Version: ${INSTALLED_VERSION}\n"
  printf "  Location: $INSTALL_DIRECTORY/soma\n\n"
  printf "Get started by running: ${bright_blue}soma --help${reset}\n"
  printf "To start a development server, run: ${bright_blue}soma dev${reset}\n\n"
}

main
