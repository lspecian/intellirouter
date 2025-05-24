# Test Restructuring Improvements Report

Generated: Wed 21 May 2025 08:13:02 PM CEST

## Overview

This report documents the verification of tests and measurements of improvements
after the test code restructuring project.

## Test Results

- ❌ Unit tests: FAILED
- ❌ Integration tests: FAILED
- ❌ E2E tests: FAILED
- ❌ Property tests: FAILED
- ❌ Custom test runner: FAILED

⚠️ **Warning**: Some tests failed. The test restructuring may have introduced issues that need to be fixed.

## Binary Size

| Metric      | Value         |
|-------------|---------------|
| Size        | 14M  |
| Size (bytes)| 14037736 |

## Compilation Time

| Build Type | Time (seconds) |
|------------|---------------|
| Debug      | 87.478967476   |
| Release    | 113.828972798 |

## Test Code in Production Binary

❌ **Test code found in production binary**

The following test-related strings were found in the production binary:

- #[test]

This suggests that some test code is still being included in the production build.
Further investigation is needed to identify and remove this test code.

## Baseline Measurements

No previous measurements were found. These measurements will serve as the baseline for future comparisons.

## Summary

The test restructuring project aimed to separate test code from production code to reduce binary size and improve compilation time. This report documents the verification of tests and measurements of improvements after the restructuring.

Key findings:
- Test Status: All tests did not pass with the new structure.
- Test Code in Production: Still found test code in the production binary.
- Binary Size: Reduced by  bytes (%).
- Debug Compilation Time: Improved by  seconds (%).
- Release Compilation Time: Improved by  seconds (%).
