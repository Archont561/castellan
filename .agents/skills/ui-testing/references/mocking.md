# Deterministic API, HAR, and external-system testing

Select the control method for each dependency rather than letting every test
call the network:

| Need | Preferred evidence |
| --- | --- |
| Component/page state from an HTTP response | MSW handler with a small deterministic payload |
| Reproducible app-level exchange with an external system | Sanitized Playwright HAR/cassette replay |
| Important schema/protocol compatibility | Contract/integration test against a controlled sandbox or real service |
| A tiny operational confidence check | Explicit, isolated real-service smoke test |

Normal component, page, and CI UI suites should not rely on provider uptime.
Neither an MSW handler nor a HAR proves that an external API still accepts the
application's requests, so retain the smallest useful contract/smoke coverage
for important dependencies.

## MSW for component, Storybook, and page scenarios

Use MSW handlers to represent UI-relevant outcomes, not to reimplement a
backend. Start with the normal successful payload and add only failure modes
that change the UI: empty, loading/delayed, validation 400, unauthenticated
401, forbidden 403, absent 404, conflict 409, rate limit 429, server 500,
timeout/network error, and malformed response. Model the API shape faithfully
from a schema/fixture/contract; do not make mocks friendlier than production.

```ts
import { http, HttpResponse, delay } from "msw";

export const invoiceHandlers = [
  http.get("/api/invoices/:id", ({ params }) =>
    HttpResponse.json({ id: params.id, status: "open", total: 42 })
  )
];

export const unavailableInvoiceHandlers = [
  http.get("/api/invoices/:id", async () => {
    await delay(250);
    return HttpResponse.json({ message: "Try again" }, { status: 503 });
  })
];
```

Register handlers through the repository's MSW setup—often Storybook
parameters for a story and the test server for component/page tests. Reset
handlers and request state between tests. Fix a test clock and request order
when it changes behavior.

## Async, cancellation, and races

A generic delay is insufficient for race coverage. Use controllable/deferred
handlers or routes to hold response A, issue response B, resolve B, then resolve
A. Assert the newer result remains visible. Exercise double submit, retry,
route change, cancellation, and unmount only if the UI can make those outcomes
observable. The test should prove the policy (ignore stale request, abort,
disable resubmit, etc.), not merely that a promise happened to settle.

## Playwright HAR/cassette replay

Use `page.routeFromHAR()` or the repository's equivalent when an application-
level test benefits from replaying a representative external conversation.
Replay must be deterministic and fail on unexpected traffic; an illustrative
shape is:

```ts
await page.routeFromHAR("tests/har/provider-sandbox.har", {
  notFound: "abort",
  update: false
});
```

Record only from a disposable test/sandbox identity with synthetic data. Before
committing, inspect every request/response/body/header; remove authorization,
cookies, API keys, PII, production identifiers, and signed URLs; replace with
safe fixture values; document origin and refresh policy. Do not record live
production traffic. A HAR remains a historical fixture, so pair important
external dependencies with schema/contract or narrow real-service verification.

## Auth and third parties

For identity, payment, CRM, analytics, messaging, GraphQL/REST, or widget
integrations, decide explicitly whether the scenario needs MSW, HAR replay,
contract evidence, or an isolated sandbox smoke test. Test meaningful permission
boundaries (for example admin can edit, read-only cannot, anonymous cannot
access), expired/refresh/logout behavior, and provider error handling without
duplicating every UI scenario for every role.
