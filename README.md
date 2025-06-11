Data Sanitizer Pro MVP
A data sanitization tool for Windows laptops, built with Tauri and Rust. This Minimum Viable Product (MVP) aims to securely erase data from external drives, with ongoing development to meet industry standards.
Project Overview
This MVP implements drive detection, safety checks, and a basic sanitization process for USB drives. Phase 2 focuses on enhancing core hardware integration, including detailed drive info, safety validation, and initial sanitization logic. Future phases will target full data destruction compliance (e.g., NIST 800-88).
Setup Instructions
Prerequisites

Operating System: Windows 10 or 11
Rust: Install via rustup by running:
powershell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update

Node.js: Download and install the LTS version from nodejs.org
Git: Install from git-scm.com
Tauri CLI: Install with:
powershell
cargo install tauri-cli --version "^1.5"


Installation

Clone the repository to your local machine:
powershell
git clone https://github.com/rhoque1/data-san.git
cd data-san

Install project dependencies:
powershell
npm install
cd src-tauri
cargo check
cd ..


Running the App

Open PowerShell as Administrator:
powershell
Start-Process powershell -Verb RunAs
cd data-san

Build and run the application:
powershell
npm run build
npm run tauri dev


Usage

Test Backend Connection: Click to verify system and backend communication.
Detect Drives: Lists all connected drives (e.g., C:, D:) with details like size, file system, and serial number.
Check Safety: Select a drive and click to confirm it's safe to sanitize (blocks system drives like C:).
Sanitize Drive: After safety check, click to overwrite the selected drive with a 3-pass method (limited to 1GB).

Testing

Connect a USB drive (e.g., D:) to your computer.
Open the app, click "Detect Drives" to list drives.
Select D:, click "Check Safety" (expect "✅ Safe to proceed with confirmation").
Click "Sanitize Drive" and expect: "✅ Sanitized D:\ with 3-pass overwrite and verified (limited to 1000MB)".

Known Limitations

Sanitization Scope: Current method overwrites only 1GB of the drive, not the full capacity.
Secure Erase: Lacks kernel-level Secure Erase (e.g., ATA commands), requiring third-party tools or future API integration.
Verification: Limited to the overwritten area; full drive verification isn't implemented.

Future Enhancements

Full Wipe: Increase overwrite to cover the entire drive.
Secure Erase: Integrate kernel-level commands or third-party tools (e.g., Eraser) for hardware-level wipes.
Compliance: Target NIST 800-88 or DoD 5220.22-M standards with multi-pass and verification.

Contributing

Follow the setup steps to get started.
Report issues or suggest improvements via GitHub Issues.
Collaborate by forking the repository and submitting pull requests.

License
[Add license here, e.g., MIT] (optional, consult with your partner)
