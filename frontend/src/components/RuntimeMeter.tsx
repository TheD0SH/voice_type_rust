import { useMemo, type CSSProperties } from "react";

function clampUnit(value: number) {
  return Math.max(0, Math.min(1, value));
}

export function RuntimeMeter(props: {
  level: number;
  peak: number;
  active: boolean;
  /** When true, level/peak/active are read from CSS custom properties
   *  (--meter-level, --meter-peak, --meter-active) on the nearest ancestor
   *  instead of from props.  Used by the HUD to avoid 60fps React re-renders. */
  cssDriven?: boolean;
}) {
  const { active, level, peak, cssDriven = false } = props;

  const bars = useMemo(() => {
    const count = 20;
    const liveLevel = clampUnit(level);
    const livePeak = clampUnit(Math.max(level, peak));
    const intensity = active ? Math.max(liveLevel, livePeak * 0.82) : livePeak * 0.28;

    return Array.from({ length: count }, (_, index) => {
      const mid = (count - 1) / 2;
      const offset = Math.abs(index - mid) / mid;
      const ridge = 1 - Math.pow(offset, 1.2) * 0.72;
      const pattern = [1, 0.88, 0.96, 0.82, 0.91][index % 5];
      const floor = active ? 0.14 : 0.08;
      const height = clampUnit(
        floor + intensity * ridge * pattern + livePeak * 0.12 * (1 - offset * 0.55),
      );

      return {
        height: Math.max(active ? 0.15 : 0.08, height),
        opacity: active ? 0.42 + height * 0.5 : 0.2 + height * 0.24,
        delay: `${(index - count / 2) * 0.08}s`,
        bounce: String(1 + (0.08 + intensity * 0.16) * ridge),
      };
    });
  }, [active, level, peak]);

  // When cssDriven, each bar gets its static geometry (floor, ridge, pattern,
  // offset) as inline CSS vars at mount.  The dynamic height/opacity is then
  // computed by CSS calc() from --meter-level/--meter-peak that the animation
  // loop updates via a DOM ref — no React re-render needed per frame.
  if (cssDriven) {
    return (
      <div
        className="meter-shell meter-shell-bars meter-shell-css-driven"
        aria-label="Audio level meter"
      >
        <div className="meter-bars">
          {bars.map((bar, index) => {
            const mid = (bars.length - 1) / 2;
            const offset = Math.abs(index - mid) / mid;
            const ridge = 1 - Math.pow(offset, 1.2) * 0.72;
            const pattern = [1, 0.88, 0.96, 0.82, 0.91][index % 5];

            return (
              <span
                key={index}
                className="meter-bar meter-bar-css-driven"
                style={
                  {
                    "--bar-delay": bar.delay,
                    "--bar-floor": "0.14",
                    "--bar-ridge": String(ridge),
                    "--bar-pattern": String(pattern),
                    "--bar-offset": String(offset),
                  } as CSSProperties
                }
              />
            );
          })}
        </div>
      </div>
    );
  }

  return (
    <div
      className={`meter-shell meter-shell-bars ${active ? "meter-shell-active" : ""}`}
      aria-label="Audio level meter"
    >
      <div className="meter-bars">
        {bars.map((bar, index) => (
          <span
            key={index}
            className="meter-bar"
            style={
              {
                "--bar-height": `${Math.round(bar.height * 100)}%`,
                "--bar-opacity": String(bar.opacity),
                "--bar-delay": bar.delay,
                "--bar-bounce": bar.bounce,
              } as CSSProperties
            }
          />
        ))}
      </div>
    </div>
  );
}
