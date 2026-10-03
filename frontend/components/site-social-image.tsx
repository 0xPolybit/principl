import { ImageResponse } from "next/og";

export const socialImageSize = { width: 1200, height: 630 };
export const socialImageContentType = "image/png";

export function createPrinciSocialImage() {
  return new ImageResponse(
    (
      <div
        style={{
          alignItems: "center",
          background: "#202b27",
          color: "#f4efe3",
          display: "flex",
          height: "100%",
          padding: "72px 88px",
          position: "relative",
          width: "100%",
        }}
      >
        <div
          style={{
            border: "2px solid #53635b",
            borderRadius: 26,
            display: "flex",
            flexDirection: "column",
            gap: 28,
            height: "100%",
            justifyContent: "center",
            padding: "55px 66px",
            width: "100%",
          }}
        >
          <div style={{ alignItems: "center", display: "flex", gap: 22 }}>
            <div
              style={{
                alignItems: "center",
                background: "#f4efe3",
                borderRadius: 14,
                color: "#202b27",
                display: "flex",
                fontSize: 52,
                fontWeight: 700,
                height: 78,
                justifyContent: "center",
                width: 78,
              }}
            >
              P
            </div>
            <span style={{ color: "#d6dfd7", fontSize: 25, letterSpacing: 4 }}>PRINCIPL / V0.1</span>
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
            <strong style={{ fontSize: 70, letterSpacing: -3 }}>Structure when needed.</strong>
            <strong style={{ color: "#c9d4cb", fontSize: 70, letterSpacing: -3 }}>Simplicity by default.</strong>
          </div>
          <span style={{ color: "#d1a08b", fontSize: 26 }}>Statically typed · Native Windows x86-64</span>
        </div>
        <div
          style={{
            background: "#c44b35",
            borderRadius: 999,
            bottom: 94,
            height: 13,
            position: "absolute",
            right: 120,
            width: 13,
          }}
        />
      </div>
    ),
    socialImageSize,
  );
}
