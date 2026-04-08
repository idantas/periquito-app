import SessionSprite from "./SessionSprite";
import grassImg from "../assets/sprites/GrassIsland.png";

interface PeriquitoState {
  task: string;
  emotion: string;
}

interface GrassIslandProps {
  state: PeriquitoState;
  width: number;
  height: number;
}

const PATCH_WIDTH = 80;

export default function GrassIsland({ state, width, height }: GrassIslandProps) {
  const patchCount = Math.ceil(width / PATCH_WIDTH) + 1;

  return (
    <div
      style={{
        position: "relative",
        width,
        height,
        overflow: "hidden",
        pointerEvents: "none",
      }}
    >
      {/* Grass patches */}
      <div style={{ display: "flex", position: "absolute", bottom: 0, width: "100%", height: "100%" }}>
        {Array.from({ length: patchCount }).map((_, i) => (
          <img
            key={i}
            src={grassImg}
            alt=""
            style={{
              width: PATCH_WIDTH,
              height: "100%",
              objectFit: "cover",
              flexShrink: 0,
            }}
          />
        ))}
      </div>

      {/* Sprite centered on the grass */}
      <div
        style={{
          position: "absolute",
          bottom: 0,
          left: "50%",
          transform: "translateX(-50%)",
        }}
      >
        <SessionSprite state={state} containerWidth={width} />
      </div>
    </div>
  );
}
