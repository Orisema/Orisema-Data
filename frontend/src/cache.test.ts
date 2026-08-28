import { QueryClient } from "@tanstack/react-query";
import { it, expect } from "vitest";
import { invalidateConnection } from "./cache";

it("invalidates edited connection metadata without invalidating another target", async () => {
  const client = new QueryClient();
  const keys = [
    ["connections"],
    ["schemas", "edited"],
    ["tables", "edited", "public"],
    ["columns", "edited", "public", "products"],
  ];
  for (const key of keys) client.setQueryData(key, []);
  client.setQueryData(["schemas", "other"], []);
  await invalidateConnection(client, "edited");
  for (const key of keys)
    expect(client.getQueryState(key)?.isInvalidated).toBe(true);
  expect(client.getQueryState(["schemas", "other"])?.isInvalidated).toBe(false);
  client.clear();
});
