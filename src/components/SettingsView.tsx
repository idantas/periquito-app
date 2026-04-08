import { useEffect, useState } from "react";
import {
  getSettings,
  updateSettings,
  previewSound,
  getAvailableSounds,
  installHooks,
  uninstallHooks,
  isHooksInstalled,
  type AppSettings,
} from "../lib/ipc";

const FONT_SIZES = [
  { label: "S", value: "small" },
  { label: "M", value: "regular" },
  { label: "L", value: "large" },
];

export default function SettingsView() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [sounds, setSounds] = useState<string[]>([]);
  const [hooksStatus, setHooksStatus] = useState<boolean | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    getSettings().then(setSettings);
    getAvailableSounds().then(setSounds);
    isHooksInstalled().then(setHooksStatus);
  }, []);

  const save = async (patch: Partial<AppSettings>) => {
    if (!settings) return;
    const updated = { ...settings, ...patch };
    setSettings(updated);
    setSaving(true);
    await updateSettings(updated);
    setSaving(false);
  };

  if (!settings) {
    return (
      <div style={{ textAlign: "center", opacity: 0.3, padding: "20px 0" }}>
        Loading settings...
      </div>
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
      {/* Hooks */}
      <SettingsSection label="Hooks">
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
          }}
        >
          <div>
            <div style={{ fontSize: 12 }}>Claude Code Hooks</div>
            <div style={{ fontSize: 10, opacity: 0.4, marginTop: 2 }}>
              {hooksStatus ? "Installed and active" : "Not installed"}
            </div>
          </div>
          <button
            onClick={async () => {
              if (hooksStatus) {
                await uninstallHooks();
                setHooksStatus(false);
              } else {
                await installHooks();
                setHooksStatus(true);
              }
            }}
            style={{
              background: hooksStatus
                ? "rgba(248, 113, 113, 0.15)"
                : "rgba(74, 222, 128, 0.15)",
              color: hooksStatus ? "#f87171" : "#4ade80",
              border: "none",
              borderRadius: 6,
              padding: "5px 12px",
              fontSize: 11,
              fontWeight: 600,
              cursor: "pointer",
            }}
          >
            {hooksStatus ? "Uninstall" : "Install"}
          </button>
        </div>
      </SettingsSection>

      {/* Sound */}
      <SettingsSection label="Sound">
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            marginBottom: 10,
          }}
        >
          <span style={{ fontSize: 12 }}>Mute</span>
          <Toggle
            checked={settings.is_muted}
            onChange={(v) => save({ is_muted: v })}
          />
        </div>

        <div style={{ opacity: settings.is_muted ? 0.3 : 1, transition: "opacity 0.2s" }}>
          <div style={{ fontSize: 10, opacity: 0.5, marginBottom: 6 }}>
            Notification sound
          </div>
          <div
            style={{
              display: "flex",
              flexWrap: "wrap",
              gap: 4,
            }}
          >
            {sounds.map((s) => (
              <button
                key={s}
                onClick={() => {
                  save({ notification_sound: s });
                  if (!settings.is_muted) previewSound(s);
                }}
                style={{
                  background:
                    settings.notification_sound === s
                      ? "rgba(167, 139, 250, 0.2)"
                      : "rgba(255,255,255,0.05)",
                  color:
                    settings.notification_sound === s
                      ? "#a78bfa"
                      : "rgba(255,255,255,0.5)",
                  border:
                    settings.notification_sound === s
                      ? "1px solid rgba(167, 139, 250, 0.3)"
                      : "1px solid transparent",
                  borderRadius: 5,
                  padding: "3px 8px",
                  fontSize: 10,
                  cursor: settings.is_muted ? "default" : "pointer",
                  pointerEvents: settings.is_muted ? "none" : "auto",
                }}
              >
                {s}
              </button>
            ))}
          </div>
        </div>
      </SettingsSection>

      {/* Font size */}
      <SettingsSection label="Display">
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
          }}
        >
          <span style={{ fontSize: 12 }}>Font size</span>
          <div style={{ display: "flex", gap: 4 }}>
            {FONT_SIZES.map((f) => (
              <button
                key={f.value}
                onClick={() => save({ font_size: f.value })}
                style={{
                  width: 32,
                  height: 28,
                  background:
                    settings.font_size === f.value
                      ? "rgba(167, 139, 250, 0.2)"
                      : "rgba(255,255,255,0.05)",
                  color:
                    settings.font_size === f.value
                      ? "#a78bfa"
                      : "rgba(255,255,255,0.5)",
                  border:
                    settings.font_size === f.value
                      ? "1px solid rgba(167, 139, 250, 0.3)"
                      : "1px solid transparent",
                  borderRadius: 6,
                  fontSize: f.value === "small" ? 10 : f.value === "regular" ? 12 : 14,
                  fontWeight: 600,
                  cursor: "pointer",
                }}
              >
                {f.label}
              </button>
            ))}
          </div>
        </div>
      </SettingsSection>

      {/* Save indicator */}
      {saving && (
        <div style={{ fontSize: 10, opacity: 0.4, textAlign: "center" }}>
          Saving...
        </div>
      )}
    </div>
  );
}

function SettingsSection({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div
      style={{
        background: "rgba(255,255,255,0.05)",
        borderRadius: 12,
        padding: "10px 14px",
      }}
    >
      <div
        style={{
          fontSize: 10,
          fontWeight: 600,
          opacity: 0.4,
          textTransform: "uppercase",
          letterSpacing: "0.05em",
          marginBottom: 8,
        }}
      >
        {label}
      </div>
      {children}
    </div>
  );
}

function Toggle({
  checked,
  onChange,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <button
      onClick={() => onChange(!checked)}
      style={{
        width: 36,
        height: 20,
        borderRadius: 10,
        border: "none",
        background: checked ? "#a78bfa" : "rgba(255,255,255,0.15)",
        position: "relative",
        cursor: "pointer",
        transition: "background 0.2s",
      }}
    >
      <div
        style={{
          width: 16,
          height: 16,
          borderRadius: 8,
          background: "white",
          position: "absolute",
          top: 2,
          left: checked ? 18 : 2,
          transition: "left 0.2s",
        }}
      />
    </button>
  );
}
