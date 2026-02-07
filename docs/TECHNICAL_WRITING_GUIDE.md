# Technical Writing Guide for Specifications

## Purpose

This guide provides best practices for writing precise, verifiable technical specifications, particularly when discussing compatibility and use cases.

## Problem: Vague Assertions

### Example of Problematic Phrasing

**BAD:**
```
This protocol change preserves all known monetary use cases.
```

### Issues with This Approach

1. **Absolute claim without evidence**: "all known" is a broad assertion that's difficult to verify and may not be accurate
2. **Vague terminology**: "monetary use cases" is imprecise—what exactly qualifies as monetary vs. non-monetary?
3. **No accountability**: "known" by whom? When? No reference or justification provided
4. **Lack of specificity**: Readers can't verify the claim without examples
5. **Potential liability**: If an unforeseen monetary use case breaks, this statement becomes problematic

## Solution: Precise Specifications

### Option 1: Conservative with Examples

**GOOD:**
```
This protocol change preserves standard monetary use cases such as payments, 
multi-signature transactions, and time-locked transfers.
```

**Benefits:**
- Provides concrete examples
- Limits scope appropriately
- Verifiable claims

### Option 2: Qualified Statement

**GOOD:**
```
This protocol change preserves all currently documented monetary use cases 
to the best of the authors' knowledge.
```

**Benefits:**
- Acknowledges limitations
- Provides temporal context
- Shows intellectual honesty

### Option 3: Comprehensive Enumeration

**BEST:**
```
This protocol change preserves monetary use cases including:
- Standard payments
- Multi-signature wallets
- Payment channels
- Time-locked transactions
- Atomic swaps
```

**Benefits:**
- Most precise and verifiable
- Enables systematic testing
- Clear success criteria

### Option 4: With Reference

**GOOD:**
```
This protocol change preserves known monetary use cases 
(see Section X for compatibility analysis).
```

**Benefits:**
- Points to detailed analysis
- Maintains conciseness
- Provides evidence trail

## Best Practices for Technical Specifications

### 1. Enumerate Specific Cases

Instead of using "all" or "every", list specific cases that have been tested or considered.

**Example:**
- **Poor:** "Supports all file formats"
- **Better:** "Supports PNG, JPEG, GIF, and WebP image formats"

### 2. Provide References

Link to compatibility testing, analysis documents, or standards.

**Example:**
```
This implementation follows the Bitcoin Core payment standards 
(version 0.21+) and has been tested against the test cases in 
[compatibility-tests.md](./compatibility-tests.md).
```

### 3. Use Qualifying Language

Use words like "documented," "tested," or "identified" rather than absolute terms.

**Examples:**
- "currently documented use cases"
- "tested scenarios"
- "identified patterns"

### 4. Acknowledge Limitations

Be honest about what hasn't been tested or considered.

**Example:**
```
This specification addresses standard payment scenarios. 
Complex smart contract interactions have not been fully analyzed 
and may require additional testing.
```

### 5. Make Claims Verifiable

Ensure that every claim can be tested or proven with available documentation.

**Example:**
```
Compatibility verified through:
- 247 unit tests covering payment scenarios
- Integration tests with Bitcoin Core 0.21.0
- Manual testing on testnet
```

## Application to gnostr

When writing specifications for gnostr's Bitcoin and Nostr integrations:

1. **Be specific about supported features:**
   - List exact NIP (Nostr Implementation Possibility) numbers
   - Specify Bitcoin Core versions tested
   - Enumerate supported payment types

2. **Provide test evidence:**
   - Link to test files
   - Reference CI/CD validation
   - Show testnet verification

3. **Acknowledge scope:**
   - Clear about what's supported
   - Honest about limitations
   - Plan for future capabilities

## Checklist for Specification Review

Before publishing a technical specification, verify:

- [ ] All absolute claims ("all", "every", "never") are justified or removed
- [ ] Vague terms are replaced with specific examples
- [ ] References to tests or documentation are provided
- [ ] Limitations and assumptions are acknowledged
- [ ] Claims are verifiable by readers
- [ ] Success criteria are clear and measurable

## References

- [RFC 2119: Key words for use in RFCs](https://www.ietf.org/rfc/rfc2119.txt)
- [Bitcoin Improvement Proposals (BIPs)](https://github.com/bitcoin/bips)
- [Nostr Implementation Possibilities (NIPs)](https://github.com/nostr-protocol/nips)

## Version History

- v1.0.0 (2026-02-07): Initial guide based on best practices for technical specification writing
