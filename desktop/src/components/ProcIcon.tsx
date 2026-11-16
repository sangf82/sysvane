import { invoke, isTauri } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

/** Icon per exe path, as a data URL, or null when the file has none. Read once per path for the window's life. */
const cache = new Map<string, Promise<string | null>>();

async function load(path: string): Promise<string | null> {
  const buf = new Uint8Array(await invoke<ArrayBuffer>("exe_icon", { path }));
  const side = Math.sqrt(buf.length / 4);
  if (!side || !Number.isInteger(side)) return null;
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = side;
  canvas.getContext("2d")!.putImageData(new ImageData(new Uint8ClampedArray(buf), side, side), 0, 0);
  return canvas.toDataURL();
}

function iconOf(path: string | null): Promise<string | null> {
  if (!path || !isTauri()) return Promise.resolve(null);
  let p = cache.get(path);
  if (!p) {
    p = load(path).catch(() => null);
    cache.set(path, p);
  }
  return p;
}

/** The program's own icon, or a blank tile of the same size so names stay aligned. */
export function ProcIcon({ exe }: { exe: string | null }) {
  const [url, setUrl] = useState<string | null>(null);
  useEffect(() => {
    let live = true;
    setUrl(null);
    iconOf(exe).then((u) => live && setUrl(u));
    return () => {
      live = false;
    };
  }, [exe]);
  return url ? <img className="picon" src={url} alt="" /> : <span className="picon blank" aria-hidden />;
}
