# Source video provenance

This skill was authored from a public YouTube tutorial and then strengthened
with deterministic visual-TDD and safe-baseline practices. This file preserves
the available source metadata and transcription extraction so that the research
trail is inspectable. It is not required to run the skill.

## Capture record

- **Retrieved:** 2026-10-05 (Europe/Warsaw)
- **Primary URL:** <https://www.youtube.com/watch?v=qaOHkqFVLm4>
- **Video ID:** `qaOHkqFVLm4`
- **Title:** *Visual Regression Testing with Playwright: Catch Every UI Change!*
- **Visibility:** Public
- **Creator:** I Code It
- **Channel handle:** `@icodeit.juntao`
- **Channel URL:** <https://www.youtube.com/channel/UCncUOcMo7oSiXRVhVH2lF1w>
- **Creator links exposed on the watch page:**
  [Twitter/X](https://twitter.com/JuntaoQiu) and
  [LinkedIn](https://www.linkedin.com/in/juntao-qiu-b865501b/)
- **Channel subscribers shown by embed:** 15.5K
- **Uploaded at:** 2025-02-09
- **Published-at field:** blank in the page extraction
- **Duration:** 10:03
- **Category:** Howto & Style
- **Views shown:** 6,513 (page summary rounds this to 6.5K)
- **Likes shown:** 140
- **Thumbnail:** <https://i.ytimg.com/vi_webp/qaOHkqFVLm4/maxresdefault.webp>
- **Audio note:** the page says “Auto-dubbed: Audio tracks for some languages
  were automatically generated.”

The watch/embedded pages did not expose comment count, tags, license,
recording location, captions-language list, a nonempty published-at value, or
other Studio-only data. Those fields are therefore not invented here. Direct
caption endpoint/API retrieval was unavailable in the execution environment
(TLS termination from direct YouTube HTTPS; alternate public APIs did not
return the video). The text below is exactly the public watch-page auto-
transcription that was accessible, normalized only from its visual line breaks
into paragraphs. It ends mid-sentence in the returned page source, so it must
be treated as an **available transcription extract**, not a claim of a complete
10:03 transcript.

## Description (verbatim)

> In this 10-minute tutorial, I’ll show you how to set up and run Visual
> Regression Tests with Playwright. We’ll walk through installing Playwright,
> configuring the test environment, writing snapshot tests, and debugging
> unexpected layout or styling changes. Whether you’re a seasoned developer or
> just getting started, this guide will help you ensure your web app always
> looks the way it should.
>
> What You’ll Learn
> • Why Visual Regression Testing matters for modern web apps.
> • Step-by-step setup of Playwright for screenshot comparisons.
> • How to handle different viewports (desktop and mobile).
> • How to update and manage snapshots over time.
>
> If you find this helpful, give it a thumbs-up and subscribe for more coding
> tips. Have questions? Drop them in the comments. Let’s keep our UIs bug-free
> together!
>
> Subscribe to my newsletter for more articles and videos on clean code and
> refactoring: <https://juntao.substack.com>
>
> **Additional Resources:**
> - React Anti-Patterns on Amazon: <https://www.amazon.com/dp/1805123971>
> - Maintainable React Book on Leanpub: <https://leanpub.com/maintainable-react>
> - React Clean Code Book on Leanpub: <https://leanpub.com/react-clean-code>
> - Mastering Maintainable React Course on Udemy:
>   <https://udemy.com/course/mastering-maintainable-react/>

## Chapters

| Start | Title |
| ---: | --- |
| 0:00 | Introduction to visual testing |
| 1:30 | Installing Playwright |
| 3:05 | Writing a visual test |
| 4:49 | Running tests and baselines |
| 6:05 | Verifying snapshot results |
| 6:57 | Debugging and updating tests |
| 9:40 | Conclusion |

## Available auto-transcription extract

> So in this video I will show you how to catch UI issues in your web
> application before it reaches your users. We will use Playwright for visual
> regression testing so that we can see it when the UI is changed unexpectedly.
> Let's get started.
>
> Unit and integration tests are great for functionalities, but they don't
> always catch when layout has changed or your dock mode doesn't work as
> expected. With visual regression testing we take screenshots of our pages and
> compare them over time so we can quickly detect and fix unexpected change.
>
> I have a simple gallery application that displays images in different
> viewports. We'll use this application to showcase how we do the visual
> regression test. As you can see, the application is very simple. It's showing
> the images in a grid and if we resize the page, you can see the layout
> changes. Things like this are very hard to detect in unit or integration
> tests, or sometimes it's even impossible. But with a visual regression test,
> we can detect the pixel difference, so we can detect the error much earlier
> before it reaches our users.
>
> Installing Playwright is very easy. We just add the `npm init` command to
> initialize the latest version. It will download all the necessary packages and
> set up the configurations, and then you're ready to go. So we simply type
> `npm init playwright@latest`; it will ask whether you want to proceed, where
> you want to put the tests, whether to add a GitHub Actions workflow, and
> whether to install the browsers. The example puts visual regression tests in a
> `vr-tests` folder and installs browsers because Playwright uses a headless
> browser underneath.
>
> It gives you a list of commands you can use. To run the tests, use `npx
> playwright test`; it will run all the tests in the configured folder. If you
> want to see the UI, there is a GUI you can use to select the test you want to
> run.
>
> The gallery test looks similar to a Jest test: a test description and a test
> body. It sets the viewport size—desktop first, with a width and height—then
> goes to the application, assumed to be running on port 5173. It waits for a
> few images and ensures the images are loaded completely. The last step checks
> the page against a screenshot of the desktop gallery. The example tolerates
> fewer than 100 pixels of difference and raises an issue for anything bigger.
>
> The first test run fails because the application is not running on port 5173:
> connection refused. After launching the application, the next test fails
> because a baseline image is not defined. That is expected: there is no
> `gallery-desktop.png` baseline yet. Running the test with the
> `--update-snapshots` parameter generates a baseline from the browser
> screenshot. A `gallery.spec.ts-snapshots` folder is generated, with baselines
> for Chrome, Firefox, and WebKit. The following normal test run passes, and
> the report shows all three browsers passing.
>
> The video then deliberately breaks the gallery UI by changing one item in the
> image array. Running the test catches the difference in all three browsers.
> In the report there are several ways to inspect exactly what differs: a diff
> view, side-by-side comparison, and a slider view. The presenter finds the
> slider especially useful: drag the bar left or right to identify exactly what
> changed. This is particularly useful for subtle differences.
>
> The page extraction ends mid-sentence after: “sometimes you can see the very
> subtle difference i”.

## How the skill extends the video

The tutorial establishes the essential loop—capture a deterministic page,
compare it to a baseline, inspect the report, and update intentional
snapshots. The skill adds safeguards needed for durable frontend visual TDD:
state/data determinism, semantic preconditions, font/image readiness, narrowly
scoped masking and tolerances, canonical CI environment, responsive test
selection, artifact retention, and explicit human baseline approval.
