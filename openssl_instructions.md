# OpenSSL Build Error on Windows: Missing Perl Module

The error message:
`Can't locate Locale/Maketext/Simple.pm in @INC`

This indicates that the Perl environment used to configure OpenSSL is missing a required module. OpenSSL's build process on Windows often relies on a Perl installation to run its `Configure` script.

## Solution

You need to install the `Locale::Maketext::Simple` Perl module.

### How to Install

**1. Using `cpan` (Recommended if Perl is set up):**

If you have `cpan` (the Comprehensive Perl Archive Network client) installed and configured, you can typically install the module by opening a command prompt or PowerShell and running:

```bash
cpan -i Locale::Maketext::Simple
```

You might be prompted to configure `cpan` if it's your first time using it. Follow the on-screen instructions.

**2. Using Strawberry Perl:**

If you don't have a robust Perl environment or `cpan` isn't working, consider installing [Strawberry Perl](http://strawberryperl.com/). It's a complete Perl distribution for Windows that includes `cpan` and many common modules pre-installed. After installing Strawberry Perl, you can try the `cpan -i` command again.

**3. Checking Git for Windows Perl:**

If you are using the Perl included with Git for Windows, ensure it's properly configured and that its `bin` directory is in your system's `PATH`. You might need to open a Git Bash terminal to use its `cpan` client.

## Next Steps

After successfully installing the `Locale::Maketext::Simple` Perl module, please retry your build command (e.g., `cargo build -j8`).

**Have you already tried these steps? If so, please describe what happened when you attempted to install `Locale::Maketext::Simple` or run your build again.**