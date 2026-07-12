import { buildHudCardStyle, hudHeadline, normalizeHudBackgroundMode } from "../lib/hud";
import { runtimeBadge } from "../lib/appearance";
import { APP_DISPLAY_NAME } from "../lib/options";
import type { Config, RuntimeSnapshot } from "../types";
import { RuntimeMeter } from "./RuntimeMeter";

export function HudCard(props: {
  config: Config;
  snapshot: RuntimeSnapshot;
  backgroundUrl: string | null;
  active: boolean;
  level: number;
  peak: number;
  preview?: boolean;
}) {
  const { active, backgroundUrl, config, level, peak, preview = false, snapshot } = props;

  const headline = hudHeadline(snapshot.appState);
  const stateMeta = runtimeBadge(snapshot.appState);
  const backgroundMode = normalizeHudBackgroundMode(config.hud_background_mode);
  const showTopline = config.hud_show_state || config.hud_show_app_name;
  const showMeter = config.hud_show_meter;
  const cardStyle = buildHudCardStyle(config, backgroundUrl);
  const mediaClass = backgroundMode === "image" && backgroundUrl ? " hud-card-media" : "";

  return (
    <div
      className={`hud-card${preview ? " hud-card-preview" : ""}${mediaClass}`}
      data-state={snapshot.appState}
      style={cardStyle}
    >
      <div className="hud-card-content">
        <div className="hud-copy">
          {showTopline ? (
            <div className="hud-topline-row">
              {config.hud_show_state ? (
                <span className={stateMeta.tone}>
                  {stateMeta.label}
                </span>
              ) : null}
              {config.hud_show_app_name ? <span className="hud-chip">{APP_DISPLAY_NAME}</span> : null}
            </div>
          ) : null}
          <strong className="hud-title">{headline}</strong>
        </div>

        {showMeter ? (
          <RuntimeMeter
            active={active}
            level={level}
            peak={peak}
            cssDriven={!preview}
          />
        ) : null}
      </div>
    </div>
  );
}
