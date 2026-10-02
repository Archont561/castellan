/* tslint:disable */
/* eslint-disable */

/**
 * Whether an entry saved at `candidate` should be offered at `origin`.
 *
 * The app uses the identical function when answering `get_entries`, so the
 * extension's local pre-filter and the app's authoritative answer can never
 * disagree about whether github.com matches accounts.github.com.
 */
export function origin_matches(origin: string, candidate: string): boolean;

/**
 * Errors surface as `JsError` so TypeScript `catch` blocks get a readable
 * message rather than a Rust panic.
 */
export function parse_otpauth(uri: string): any;

/**
 * The protocol version of the shared logic this build was cut from.
 *
 * Lets a packed extension answer `hello` checks without waking the app.
 */
export function protocol_version(): number;
