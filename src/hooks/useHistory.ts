import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { HistoryRecord } from "../types";

export function useHistory() {
  const [records, setRecords] = useState<HistoryRecord[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const data = await invoke<HistoryRecord[]>("list_compression_history");
      setRecords(data);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const clearHistory = useCallback(async () => {
    await invoke("clear_compression_history");
    setRecords([]);
  }, []);

  return { records, loading, refresh, clearHistory };
}
