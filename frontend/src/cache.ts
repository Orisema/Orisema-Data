import type { QueryClient } from "@tanstack/react-query";

export async function invalidateConnection(client: QueryClient, id: string) {
  await Promise.all([
    client.invalidateQueries({ queryKey: ["connections"] }),
    ...["schemas", "tables", "columns"].map((kind) =>
      client.invalidateQueries({ queryKey: [kind, id] }),
    ),
  ]);
}
