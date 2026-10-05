# Critical E2E journeys and CI

E2E tests prove a small number of high-value journeys whose value comes from
real composition: routing, session lifecycle, browser behavior, multiple pages,
or a business outcome. They are not the default test for every component.

## Select journeys by risk

Choose journeys a component/page suite cannot honestly prove, such as sign in →
protected deep link → sign out; create/edit/submit a business record across
pages; pay/confirm in a sandbox; install/use a browser integration; or recover
from an expired session. Define the start state, user goal, dependency control,
and final observable outcome. Keep the test focused enough that a failure has a
clear owner.

For authenticated apps, cover only meaningful boundaries: anonymous access,
successful login, authenticated behavior, expired/refresh/logout policy,
unauthorized and forbidden routes. A role matrix should exercise distinct
permissions—not clone every suite for admin, editor, and reader.

## Determinism and isolation

Each journey provisions its own synthetic identity, data, storage, feature
flags, and external mocks/cassettes. Use test-specific sandbox accounts and
clean them through an API/database fixture when required. Never make test B
depend on test A's record, test order, browser storage, or a shared mutable
provider account. Avoid normal CI reliance on live vendor availability.

Use page-level tests for route composition first. Promote only the cross-page or
browser-critical path to E2E, then preserve the component tests as the detailed
behavior diagnosis.

## CI and browser matrix

Run focused E2E locally while implementing; run the selected critical journeys
in CI. Read the project's support policy and existing projects before expanding
browsers. A practical matrix may use Chromium for fast component feedback,
Chromium plus a required mobile/WebKit target for visual checks, and the
supported browser matrix for critical E2E. The project's promised support—not
test-count optics—sets the matrix.

Retain screenshots, traces, video, request logs, and visual diff/report
artifacts on CI failure under ignored artifact directories. Classify failures
before retrying or changing timeouts. Retries can expose intermittent
infrastructure behavior; they do not convert a race or product regression into
a passing test.
