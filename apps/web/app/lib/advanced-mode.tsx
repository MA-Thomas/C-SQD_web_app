"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";

const STORAGE_KEY = "csqd_advanced_mode";

type AdvancedModeValue = {
  advanced: boolean;
  toggle: () => void;
};

const AdvancedModeContext = createContext<AdvancedModeValue>({
  advanced: false,
  toggle: () => {},
});

export function AdvancedModeProvider({ children }: { children: ReactNode }) {
  const [advanced, setAdvanced] = useState(false);

  useEffect(() => {
    try {
      setAdvanced(window.localStorage.getItem(STORAGE_KEY) === "true");
    } catch {
      // localStorage unavailable; default to friendly labels.
    }
  }, []);

  const toggle = useCallback(() => {
    setAdvanced((current) => {
      const next = !current;

      try {
        window.localStorage.setItem(STORAGE_KEY, String(next));
      } catch {
        // Persistence is best-effort.
      }

      return next;
    });
  }, []);

  const value = useMemo(() => ({ advanced, toggle }), [advanced, toggle]);

  return (
    <AdvancedModeContext.Provider value={value}>{children}</AdvancedModeContext.Provider>
  );
}

export function useAdvancedMode() {
  return useContext(AdvancedModeContext);
}
