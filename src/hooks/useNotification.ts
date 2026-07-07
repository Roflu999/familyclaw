import { useCallback } from "react";
import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";

export function useNotification() {
  const ensurePermission = useCallback(async () => {
    let granted = await isPermissionGranted();
    if (!granted) {
      const perm = await requestPermission();
      granted = perm === "granted";
    }
    return granted;
  }, []);

  const notify = useCallback(async (title: string, body: string) => {
    const granted = await ensurePermission();
    if (granted) {
      sendNotification({ title, body });
    }
  }, [ensurePermission]);

  return { notify, ensurePermission };
}
