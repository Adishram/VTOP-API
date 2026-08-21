# Security Policy 🛡️

This project acts as a bridge between the VIT Chennai VTOP student portal and custom frontend applications. By design, it handles highly sensitive data (student passwords, academic records, and session cookies). 

**Please read this carefully before deploying or running this service.**

## Core Principles
1. **Self-Host Only**: This backend is designed to be self-hosted locally (e.g., on a student's laptop via Docker) or deployed as a single-user instance. **Do not** host a shared public instance of this API for multiple users. You will likely trigger VTOP's rate limits and security flags, and hosting other people's passwords—even ephemerally—is a massive liability.
2. **Ephemeral Memory Only**: This application does not use a database. 
   - Passwords are held in memory for less than 1 second (only during the `POST /api/login/verify` request) and are immediately dropped.
   - VTOP Session Cookies are kept in an in-memory `reqwest::cookie::Jar`. If the server restarts, all sessions are wiped.
3. **No Logging of Credentials**: We enforce strict logging policies. No passwords, session tokens, or sensitive user data are ever written to stdout or stderr.

## .env File
- **Never commit your `.env` file**. A `.gitignore` is provided to ensure it stays out of version control.
- If you deploy this service, inject the environment variables natively through your hosting provider rather than copying the `.env` file into the container.

## Reporting a Vulnerability
Since this is an educational workshop project, there is no formal bug bounty. However, if you discover a flaw where the system inadvertently logs credentials or persists sensitive data to disk, please open an Issue immediately or inform the workshop instructors.
