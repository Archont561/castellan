import type { OtpImportCandidate, OtpImportSelection } from "@castellan/protocol";
import { expect, test } from "@playwright/experimental-ct-svelte";
import OtpImport from "@/src/components/OtpImport.svelte";

const candidates: OtpImportCandidate[] = [
  {
    issuer: "GitHub",
    account: "octocat",
    otpauth: "otpauth://totp/GitHub:octocat?secret=JBSWY3DPEHPK3PXP",
    problem: null
  },
  {
    issuer: "Example",
    account: "alice",
    otpauth: "otpauth://totp/Example:alice?secret=JBSWY3DPEHPK3PXP",
    problem: null
  },
  {
    issuer: "Legacy",
    account: "bob",
    otpauth: null,
    problem: "HOTP accounts need counter support (task-42)"
  }
];

test("lets a user review a batch and import only the accepted accounts", async ({ mount }) => {
  const imports: OtpImportSelection[][] = [];
  const panel = await mount(OtpImport, {
    props: {
      preview: async () => candidates,
      importAccounts: async (selection) => {
        imports.push(selection);
        return selection.length;
      }
    }
  });

  await panel.getByLabel("Import payload").fill("otpauth://totp/anything");
  await panel.getByRole("button", { name: "Preview" }).press("Enter");

  const review = panel.getByRole("list", { name: "Accounts to import" });
  await expect(review.getByRole("listitem")).toHaveCount(3);
  await expect(review).toContainText("HOTP accounts need counter support");
  await panel.getByRole("checkbox", { name: "Import Example" }).uncheck();
  await panel.getByRole("button", { name: "Import 1 account" }).click();

  await expect(panel.getByRole("status")).toHaveText("Imported 1 account.");
  expect(imports).toEqual([
    [
      {
        title: "GitHub",
        username: "octocat",
        otpauth: "otpauth://totp/GitHub:octocat?secret=JBSWY3DPEHPK3PXP"
      }
    ]
  ]);
});

test("previews a decoded QR image through the face decoder", async ({ mount }) => {
  const decoded: File[] = [];
  const panel = await mount(OtpImport, {
    props: {
      preview: async () => candidates,
      importAccounts: async () => 0,
      decodeImage: async (file) => {
        decoded.push(file);
        return "decoded-from-qr";
      }
    }
  });

  await panel.getByLabel("Import a file or QR image").setInputFiles({
    name: "qr.png",
    mimeType: "image/png",
    buffer: Buffer.from("synthetic QR fixture")
  });

  await expect(panel.getByLabel("Import payload")).toHaveValue("decoded-from-qr");
  await expect(panel.getByRole("list", { name: "Accounts to import" })).toBeVisible();
  expect(decoded).toHaveLength(1);
});

test("announces an invalid import payload", async ({ mount }) => {
  const panel = await mount(OtpImport, {
    props: {
      preview: async () => {
        throw new Error("not a recognized import");
      },
      importAccounts: async () => 0
    }
  });

  await panel.getByLabel("Import payload").fill("garbage");
  await panel.getByRole("button", { name: "Preview" }).click();

  await expect(panel.getByRole("alert")).toContainText("not a recognized import");
  await expect(panel.getByRole("list", { name: "Accounts to import" })).toHaveCount(0);
});

test("keeps the most recent preview when requests resolve out of order", async ({ mount }) => {
  const pending = new Map<string, (value: OtpImportCandidate[]) => void>();
  const panel = await mount(OtpImport, {
    props: {
      preview: (payload) =>
        new Promise<OtpImportCandidate[]>((resolve) => {
          pending.set(payload, resolve);
        }),
      importAccounts: async () => 0
    }
  });

  await panel.getByLabel("Import payload").fill("first");
  await panel.getByRole("button", { name: "Preview" }).click();
  await panel.getByLabel("Import payload").fill("second");
  await panel.getByRole("button", { name: "Preview" }).click();

  const resolveCurrent = pending.get("second");
  if (!resolveCurrent) throw new Error("the current preview was not requested");
  resolveCurrent([
    { issuer: "Current", account: "current", otpauth: "otpauth://totp/Current", problem: null }
  ]);
  await expect(panel.getByRole("list", { name: "Accounts to import" })).toContainText("Current");

  const resolveStale = pending.get("first");
  if (!resolveStale) throw new Error("the stale preview was not requested");
  resolveStale([
    { issuer: "Stale", account: "stale", otpauth: "otpauth://totp/Stale", problem: null }
  ]);
  await expect(panel.getByRole("list", { name: "Accounts to import" })).toContainText("Current");
  await expect(panel.getByText("Stale", { exact: true })).toHaveCount(0);
});

test("prevents a second import while the first is still pending", async ({ mount }) => {
  let imports = 0;
  let completeImport: ((count: number) => void) | undefined;
  const panel = await mount(OtpImport, {
    props: {
      preview: async () => candidates.slice(0, 1),
      importAccounts: async () => {
        imports += 1;
        return new Promise<number>((resolve) => {
          completeImport = resolve;
        });
      }
    }
  });

  await panel.getByLabel("Import payload").fill("one account");
  await panel.getByRole("button", { name: "Preview" }).click();
  await panel.getByRole("button", { name: "Import 1 account" }).click();

  await expect(panel).toHaveAttribute("aria-busy", "true");
  await expect(panel.getByRole("button", { name: "Importing…" })).toBeDisabled();
  expect(imports).toBe(1);

  if (!completeImport) throw new Error("the import did not start");
  completeImport(1);
  await expect(panel.getByRole("status")).toHaveText("Imported 1 account.");
  expect(imports).toBe(1);
});
