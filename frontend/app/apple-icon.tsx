import { ImageResponse } from "next/og";

export const size = { width: 180, height: 180 };
export const contentType = "image/png";
export const alt = "PrinciPL mark";

export default function AppleIcon() {
  return new ImageResponse(
    (
      <div
        style={{
          alignItems: "center",
          background: "#202b27",
          borderRadius: 38,
          color: "#f4efe3",
          display: "flex",
          fontSize: 118,
          fontWeight: 700,
          height: "100%",
          justifyContent: "center",
          position: "relative",
          width: "100%",
        }}
      >
        P
        <div
          style={{
            background: "#c44b35",
            borderRadius: 999,
            bottom: 32,
            height: 20,
            position: "absolute",
            right: 31,
            width: 20,
          }}
        />
      </div>
    ),
    size,
  );
}
