<script lang="ts">
import TotpCode from "@/src/components/TotpCode.svelte";

interface Totp {
  code: string;
  secondsRemaining: number;
}

async function getCode(): Promise<Totp> {
  throw new Error("the vault is locked");
}
</script>

<TotpCode {getCode} />
