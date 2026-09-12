# HostDeck Vault

HostDeck Vault is a secure, cross-platform infrastructure management application designed to eliminate friction in handling SSH configurations, identities, and remote server access. Engineered with Rust and React via the Tauri framework, it provides a native, high-performance control surface for managing a fleet of servers without relying on cloud-hosted intermediaries or third-party credential stores.

## Core Capabilities

- **Secure Configuration Management**: HostDeck Vault natively interfaces with your operating system's local credential mechanisms, ensuring that sensitive data such as SSH keys, passphrases, and GitHub tokens remain exclusively on your device.
- **Identity Deployment**: Directly distribute public keys to remote infrastructure with automated passwordless setup sequences, bypassing manual configuration steps.
- **Git Synchronization**: Maintain complete revision history and cross-device consistency by securely syncing your infrastructure definitions to a private repository.
- **Native Performance**: Built on a memory-safe Rust backend with a minimal memory footprint, HostDeck Vault launches instantly and performs operations directly against standard TCP streams rather than abstracting through slow sub-processes.
- **Jump Host Routing**: Define and orchestrate complex SSH jumps through bastion hosts intuitively from the user interface.

## Architecture

HostDeck Vault adheres to a strict client-side security model. The application functions exclusively as an interface between your local configuration files and the remote servers you manage. No telemetry is collected, and no data is transmitted to centralized servers. All data synchronization is executed exclusively through standard Git protocols to a remote repository of your choosing.

## Installation Guide

HostDeck Vault is distributed as pre-compiled, native executables for Windows, macOS, and Linux.

### Requirements
- Administrator or elevated privileges to install system-wide applications.
- (Optional) A pre-configured Git environment if utilizing the remote synchronization feature.

### Standard Installation

1. Navigate to the **Releases** section of this repository.
2. Download the appropriate installer for your operating system:
   - **Windows**: Download the `.msi` or `.exe` installer.
   - **macOS (Apple Silicon)**: Download the `aarch64.dmg` installer.
   - **macOS (Intel)**: Download the `x86_64.dmg` installer.
   - **Linux**: Download the `.AppImage` or `.deb` package.
3. Execute the installer and follow the standard operating system prompts.
4. Launch HostDeck Vault from your application directory or start menu.

### Security Notice

This repository operates under a proprietary license. The source code is provided for educational and audit purposes only. Redistribution, modification, or commercial usage without explicit written permission is strictly prohibited. Refer to the LICENSE file for complete terms and conditions.
