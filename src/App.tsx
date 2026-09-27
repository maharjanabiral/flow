import { useEffect, useState, useRef } from "react";
import { listen } from "@tauri-apps/api/event";

type Status = "idle" | "recording" | "transcribing";
const BAR_COUNT = 24;

function App() {
  const [status, setStatus] = useState<Status>("idle");
  const [levels, setLevels] = useState<number[]>(Array(BAR_COUNT).fill(2));
  const levelsRef = useRef(levels);
  levelsRef.current = levels;

  useEffect(() => {
    const unlistenStarted = listen("recording-started", () => setStatus("recording"));
    const unlistenStopped = listen("recording-stopped", () => setStatus("transcribing"));
    const unlistenDone = listen("transcription-done", () => setStatus("idle"));
    const unlistenEmpty = listen("transcription-empty", () => setStatus("idle"));

    const unlistenLevel = listen<number>("audio-level", (event) => {
      const normalized = Math.min(100, Math.max(4, event.payload / 8));
      setLevels((prev) => [...prev.slice(1), normalized]);
    });

    return () => {
      unlistenStarted.then((f) => f());
      unlistenStopped.then((f) => f());
      unlistenDone.then((f) => f());
      unlistenEmpty.then((f) => f());
      unlistenLevel.then((f) => f());
    };
  }, []);

  if (status === "idle") {
    return <div className="w-full h-full" />; // fully transparent, nothing visible
  }

  return (
    <div className="flex h-full w-full items-center justify-center">
      <div className="flex items-center gap-2 rounded-full bg-black/85 px-5 py-3 shadow-2xl backdrop-blur-md">
        {status === "recording" ? (
          <div className="flex h-6 items-center gap-[3px]">
            {levels.map((level, i) => (
              <span
                key={i}
                className="w-[3px] rounded-full bg-white transition-all duration-75"
                style={{ height: `${level}%` }}
              />
            ))}
          </div>
        ) : (
          <span className="text-xs text-white/80">Transcribing…</span>
        )}
      </div>
    </div>
  );
}

export default App;
