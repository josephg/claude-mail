# RFC Assumptions and Ambiguities in the JMAP Test Suite

## Part 1: Assumptions Made

This lists every place where the test suite makes a judgment call about RFC 8620/8621 interpretation. These are grouped by theme.

---

### A. Tests that treat SHOULD as MUST

These tests assert behavior the RFC only recommends (SHOULD), not requires (MUST).

1. **`core/error-not-request`** (RFC 8620 S3.6.1) — Asserts HTTP 400 for JSON that isn't a valid JMAP Request. RFC says the server "SHOULD" return 400, not MUST.

2. **`email/filter-text-search`** (RFC 8621 S4.4.1) — Asserts that `text` filter finds content in email body. RFC says server "MUST look up text in from, to, cc, bcc, and subject" but only "SHOULD look inside the plain and HTML body parts." Test treats body search as required.

3. **`search-snippet/snippet-mark-tags`** (RFC 8621 S5) — Asserts matching text is enclosed in `<mark></mark>` tags. RFC says "SHOULD" use these tags.

---

### B. Tests that treat MUST as MAY (overly lenient)

These tests accept behaviors the RFC actually forbids. They pass even when the server violates a MUST.

4. **`core/error-empty-using`** (RFC 8620 S3.2) — Test always passes regardless of server response. RFC says `using` MUST contain at least the core capability, so an empty array should be an error.

5. **`core/error-state-mismatch`** (RFC 8620 S5.3) — Test passes silently if the server ignores an invalid `ifInState`. RFC says server "MUST reject the entire method call with a stateMismatch error."

6. **`mailbox/get-properties-filter`** (RFC 8620 S5.1) — The `properties` filter is treated as purely advisory and the assertion always passes. RFC says "only the properties listed in the array are returned" (MUST).

7. **`mailbox/set-duplicate-name-same-parent`** (RFC 8621 S2) — Test accepts both rejection and success for two sibling mailboxes with the same name. RFC says "There MUST NOT be two sibling Mailboxes with the same name."

8. **`mailbox/set-cannot-destroy-with-children`** (RFC 8621 S2.5) — Test passes even if the server successfully destroys a mailbox with children. RFC says this MUST fail with `mailboxHasChild`.

9. **`email/paging-anchor-not-found`** (RFC 8620 S5.5) — Test accepts empty results if server doesn't return `anchorNotFound` error. RFC says server "MUST return an anchorNotFound error."

10. **`email/import-invalid-blob`** (RFC 8621 S4.8) — Test passes if server imports non-email blob data without error. RFC says "the server MUST parse the RFC 5322 message" implying invalid content should be rejected.

11. **`email/parse-not-parsable`** (RFC 8621 S4.9) — Test passes if server parses non-email content. RFC says non-parsable blobs should go in `notParsable`.

12. **`submission/set-no-recipients-error`** (RFC 8621 S7.5) — Test passes if server accepts email with no recipients. RFC defines `noRecipients` SetError for this case.

13. **`push-subscription/push-subscription-receives-notification`** (RFC 8620 S7.2) — Test passes even if no push notification arrives within 3 seconds. RFC says server MUST send notifications on state change.

---

### C. HTTP status code choices

14. **`core/error-not-json`** and **`core/error-wrong-content-type`** (RFC 8620 S3.6.1) — Tests accept either 400 or 415. RFC only mentions 400 (with `urn:ietf:params:jmap:error:notJSON`). 415 is a reasonable HTTP-level response but isn't in the RFC.

15. **`core/error-unknown-capability`** (RFC 8620 S3.6.1) — Test accepts both HTTP-level 400 errors and JMAP method-level `unknownCapability` errors. RFC describes this as an HTTP-level error only.

---

### D. Error type assumptions

16. **`core/error-invalid-arguments-missing-account`** (RFC 8620 S3.6.2) — Accepts both `invalidArguments` and `accountNotFound` when `accountId` is omitted from a method call. Also accepts the server defaulting to primary account (no error at all).

17. **`email/import-not-found-blob`** (RFC 8621 S4.8) — Asserts error type `blobNotFound`. RFC says "the server MUST reject with a SetError of type `blobNotFound`." This is correct but worth confirming.

---

### E. Collation and sort ordering

18. **`mailbox/query-sort-by-name`** and **`email/sort-subject`** (RFC 8621 S2.3, S4.4.2) — Tests verify sort order using JavaScript's `localeCompare`. The server's collation algorithm may differ (RFC 8620 S5.5 mentions server-advertised collation algorithms).

19. **`thread/get-thread-email-ids-order`** (RFC 8621 S3) — Test checks `emailIds` are sorted by `receivedAt` but doesn't verify alphabetical tiebreaking when dates are equal. RFC specifies both criteria.

---

### F. Content-Type handling in uploads/downloads

20. **`binary/upload-basic`** and **`binary/upload-preserves-content-type`** (RFC 8620 S6.1) — Tests assert the returned `type` field exactly matches the uploaded Content-Type (e.g., `text/plain`). Server might normalize to `text/plain; charset=UTF-8`.

21. **`binary/download-respects-type-param`** (RFC 8620 S6.2) — Test asserts the Content-Type of the download response matches the requested type parameter. RFC says the server "MAY" use this to set the Content-Type header, not MUST.

---

### G. Default values and edge cases

22. **`core/echo-nested`** (RFC 8620 S4) — Tests floating-point round-trip (3.14). RFC says server returns "the same object" but JSON float precision is not addressed.

23. **`email/get-received-at-is-utc-date`** (RFC 8621 S4.1.1) — Uses JavaScript `new Date()` parsing which is more lenient than RFC 3339. Doesn't verify the "Z" suffix or strict UTC format.

24. **`email/body-max-body-value-bytes`** (RFC 8621 S4.2) — Tests `maxBodyValueBytes: 100` but allows up to 200 characters in the response. RFC says the value MUST be truncated to the maximum number of octets. The generous limit may mask truncation bugs.

25. **`email/header-subject-empty`** (RFC 8621 S4.1.2.4) — Accepts both `""` and `null` for an email with no Subject header. RFC says subject is `null` if the header is not present. If the header IS present but empty, it should be `""`. These are different cases.

26. **`email/header-case-insensitive`** (RFC 8621 S4.1.2) — Only asserts "at least one" of `header:subject:asText` or `header:SUBJECT:asText` returns a value. Should assert BOTH return the same value since header names are case-insensitive per the RFC.

---

### H. Singleton and scope assumptions

27. **`core/session-has-mail-capability`** (RFC 8621 S2) — Asserts `urn:ietf:params:jmap:mail` MUST be in session capabilities. A pure RFC 8620 server (no mail) wouldn't have this. This is a test scope assumption.

28. **`identity/get-all-identities`** (RFC 8621 S6.1) — Asserts at least one identity exists. RFC doesn't explicitly require this, though it's implied for submission-capable accounts.

---

### I. Same-account Email/copy

29. **`email/copy-same-account`** (RFC 8621 S4.7) — Tests Email/copy within the same account. RFC defines /copy as cross-account. Same-account use isn't prohibited but also isn't the intended use case.

---

### J. Asynchronous processing assumptions

30. **`submission/set-on-success-update-email`** (RFC 8621 S7.5) — Inserts a 500ms sleep after submission, implying `onSuccessUpdateEmail` might be processed asynchronously. RFC describes it as part of the same method call response (synchronous). The test should see the implicit Email/set response in `methodResponses` without waiting.

---

### K. Weak tests (assertions don't actually verify behavior)

31. **`email/sort-from`** and **`email/sort-to`** (RFC 8621 S4.4.2) — Only assert results are returned (length > 0), don't verify sort order at all.

32. **`push-eventsource/eventsource-types-filter`** and **`push-eventsource/eventsource-closeafter`** (RFC 8620 S7.3) — Only verify HTTP 200, don't verify filtering behavior or connection closure.

33. **`push-subscription/push-subscription-verification`** (RFC 8620 S7.2.2) — Skips the entire verification flow. Passes regardless.

---
