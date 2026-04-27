"use client";

import { useState } from "react";
import { TokenPairSelector, type TokenPair } from "@swyft/ui";
import { useTokens, useRecentTokens, usePoolExists } from "@/hooks/useTokens";

export function SwapWidget() {
  const { tokens, loading } = useTokens();
  const { recentIds, pushRecent } = useRecentTokens();
  const [pair, setPair] = useState<TokenPair>({ tokenIn: null, tokenOut: null });

  const poolExists = usePoolExists(pair.tokenIn?.id ?? null, pair.tokenOut?.id ?? null);

  function handleChange(next: TokenPair) {
    if (next.tokenIn) pushRecent(next.tokenIn.id);
    if (next.tokenOut) pushRecent(next.tokenOut.id);
    setPair(next);
  }

  return (
    <div className="rounded-2xl border border-zinc-200 bg-white p-6 shadow-sm dark:border-zinc-800 dark:bg-zinc-900 w-full max-w-md">
      <h2 className="mb-4 text-base font-semibold text-zinc-900 dark:text-white">Swap</h2>
      <TokenPairSelector
        pair={pair}
        tokens={tokens}
        recentIds={recentIds}
        loading={loading}
        poolExists={poolExists}
        onChange={handleChange}
      />
    </div>
  );
}
